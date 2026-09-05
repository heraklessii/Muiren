//! Bellek ölçümü — Win32'ye özel.
//!
//! **İki ayrı şey ölçülüyor ve karıştırılmıyor** (`docs/Bellek.md`):
//!
//! 1. **Sistem baskısı** — `GlobalMemoryStatusEx`. Ucuz, sık çağrılabilir,
//!    eşik kararının girdisi.
//! 2. **Süreç bellekleri** — WebView2 süreçlerinin toplamı. Yalnız kullanıcıya
//!    gösterilen bilgi; **karar buna dayanmıyor.** Bu ayrım kasıtlı: süreç →
//!    sekme eşlemesi güvenilmez çıkarsa panel "yaklaşık" diyor ama politika
//!    çalışmaya devam ediyor.
//!
//! Seviyeye çevirme (`BellekBaskisi::yuzdeden`) burada **değil**, `esik.rs`
//! içinde: eşiği Win32 çağrısının yanında hesaplasaydık motorsuz derlemede
//! test edilemezdi.
//!
//! > **Kural (CLAUDE.md #5):** buradaki hiçbir ölçüm uyuyan bir sekmeye
//! > dokunmuyor. Süreç tablosunu okumak sayfayı uyandırmıyor; bu dosyanın
//! > sayfaya hiç erişimi yok.

/// Sistemin o anki fiziksel bellek durumu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SistemBellek {
    pub toplam_mb: u64,
    pub bos_mb: u64,
}

impl SistemBellek {
    pub fn bos_yuzde(&self) -> f64 {
        if self.toplam_mb == 0 {
            // Ölçüm alınamadı. %100 boş varsayılıyor: bilinmeyen bir durumda
            // bütün sekmeleri atmak, hiçbirini atmamaktan çok daha kötü.
            return 100.0;
        }
        (self.bos_mb as f64 / self.toplam_mb as f64) * 100.0
    }
}

/// WebView2 süreçlerinin toplamı ve kabuğun kendi payı.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SurecBellegi {
    /// Muiren'in doğurduğu WebView2 süreçlerinin toplamı (MB).
    pub webview_mb: u64,
    /// Kabuğun kendi süreci (MB).
    pub kabuk_mb: u64,
    /// Sayılan WebView2 süreci adedi. Faz 0/R4'ün ölçüm aracı: aynı siteden
    /// 10 sekme açıp buraya bakınca bayrağın geçip geçmediği görünüyor.
    pub surec_sayisi: u32,
    /// Süreç başına özel bellek (PID → MB).
    ///
    /// Toplamın ayrıntısı: `webview_mb` bu tablonun toplamı. Ayrı durmasının
    /// sebebi sekme başına ölçüm — motorun verdiği süreç → sekme eşlemesiyle
    /// birleşen yer `memory/esleme.rs` ve o dosya bir PID'e bakmadan çalışmak
    /// zorunda (saf). Tablo aynı taramadan çıkıyor; ikinci bir Toolhelp32
    /// anlık görüntüsü alınmıyor.
    pub pid_mb: std::collections::HashMap<u32, u64>,
}

#[cfg(feature = "motor")]
mod win {
    use std::collections::HashMap;

    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows::Win32::System::Threading::{
        GetCurrentProcessId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    use super::{SistemBellek, SurecBellegi};

    const MB: u64 = 1024 * 1024;

    pub fn sistem() -> SistemBellek {
        let mut durum = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        // SAFETY: `dwLength` doldurulmuş; API sözleşmesinin istediği tek şey bu.
        let ok = unsafe { GlobalMemoryStatusEx(&mut durum) }.is_ok();
        if !ok {
            return SistemBellek {
                toplam_mb: 0,
                bos_mb: 0,
            };
        }
        SistemBellek {
            toplam_mb: durum.ullTotalPhys / MB,
            bos_mb: durum.ullAvailPhys / MB,
        }
    }

    /// Bir sürecin özel (private) bellek kullanımı, MB.
    ///
    /// `PrivateUsage` seçildi, `WorkingSetSize` değil: çalışma kümesi diğer
    /// süreçlerle paylaşılan sayfaları da içeriyor ve süreçler toplandığında
    /// aynı sayfa defalarca sayılıyor. "Bu sekmeyi kapatırsam ne kazanırım"
    /// sorusunun cevabı özel bellek.
    fn surec_mb(pid: u32) -> Option<u64> {
        // SAFETY: elde edilen tanıtıcı hata durumunda kapatılıyor; başarıda da
        // fonksiyondan çıkmadan önce.
        unsafe {
            let h: HANDLE = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
                false,
                pid,
            )
            .ok()?;
            let mut sayac = PROCESS_MEMORY_COUNTERS_EX::default();
            let sonuc = GetProcessMemoryInfo(
                h,
                &mut sayac as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
                std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            );
            let _ = CloseHandle(h);
            if sonuc.is_err() {
                return None;
            }
            Some(sayac.PrivateUsage as u64 / MB)
        }
    }

    /// Süreç tablosunu bir kez okuyup (pid → (ebeveyn, ad)) haritası çıkarır.
    fn surec_tablosu() -> HashMap<u32, (u32, String)> {
        let mut tablo = HashMap::new();
        // SAFETY: snapshot tanıtıcısı fonksiyondan çıkmadan kapatılıyor;
        // `PROCESSENTRY32W.dwSize` API sözleşmesine göre dolduruluyor.
        unsafe {
            let Ok(anlik) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
                return tablo;
            };
            let mut kayit = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            if Process32FirstW(anlik, &mut kayit).is_ok() {
                loop {
                    let ad = String::from_utf16_lossy(
                        &kayit.szExeFile[..kayit
                            .szExeFile
                            .iter()
                            .position(|c| *c == 0)
                            .unwrap_or(kayit.szExeFile.len())],
                    );
                    tablo.insert(
                        kayit.th32ProcessID,
                        (kayit.th32ParentProcessID, ad.to_ascii_lowercase()),
                    );
                    if Process32NextW(anlik, &mut kayit).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(anlik);
        }
        tablo
    }

    /// Muiren'in kendi WebView2 süreçleri.
    ///
    /// Ada göre değil **soy ağacına göre** filtreleniyor: `msedgewebview2.exe`
    /// başka uygulamalarda da koşuyor (Teams, Office, Windows'un kendi
    /// parçaları) ve hepsini toplamak kullanıcıya kendi tarayıcısının üç katı
    /// bellek yediğini gösterirdi.
    pub fn surecler() -> SurecBellegi {
        let tablo = surec_tablosu();
        // SAFETY: parametresiz, her zaman başarılı.
        let benim = unsafe { GetCurrentProcessId() };

        let bizden_mi = |mut pid: u32| -> bool {
            // Zincir uzun değil (kabuk → browser → renderer) ama bozuk bir
            // tabloda döngü olabilir; adım sayısı sınırlı.
            for _ in 0..16 {
                let Some((ebeveyn, _)) = tablo.get(&pid) else {
                    return false;
                };
                if *ebeveyn == benim {
                    return true;
                }
                if *ebeveyn == 0 || *ebeveyn == pid {
                    return false;
                }
                pid = *ebeveyn;
            }
            false
        };

        let mut toplam = 0u64;
        let mut sayi = 0u32;
        let mut pid_mb = HashMap::new();
        for (pid, (_, ad)) in &tablo {
            if ad == "msedgewebview2.exe" && bizden_mi(*pid) {
                if let Some(mb) = surec_mb(*pid) {
                    toplam += mb;
                    sayi += 1;
                    pid_mb.insert(*pid, mb);
                }
            }
        }

        SurecBellegi {
            webview_mb: toplam,
            kabuk_mb: surec_mb(benim).unwrap_or(0),
            surec_sayisi: sayi,
            pid_mb,
        }
    }
}

#[cfg(not(feature = "motor"))]
mod win {
    use super::{SistemBellek, SurecBellegi};

    /// Motorsuz derlemede ölçüm yok. `toplam_mb = 0` "bilinmiyor" demek ve
    /// [`SistemBellek::bos_yuzde`] onu %100 boş sayıyor — yani politika
    /// hiçbir sekmeye dokunmuyor. Doğru yön bu: ölçemediğimiz bir makinede
    /// sekme atmak, atmamaktan kötü.
    pub fn sistem() -> SistemBellek {
        SistemBellek {
            toplam_mb: 0,
            bos_mb: 0,
        }
    }

    pub fn surecler() -> SurecBellegi {
        SurecBellegi::default()
    }
}

pub use win::{sistem, surecler};

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn olculemeyen_makine_bos_sayiliyor() {
        // Ölçüm alınamadığında baskı `Dusuk` çıkmalı: bilinmeyen bir durumda
        // bütün sekmeleri atmak, hiçbirini atmamaktan çok daha kötü.
        let s = SistemBellek {
            toplam_mb: 0,
            bos_mb: 0,
        };
        assert_eq!(s.bos_yuzde(), 100.0);
        assert_eq!(
            crate::memory::esik::BellekBaskisi::yuzdeden(s.bos_yuzde()),
            crate::memory::esik::BellekBaskisi::Dusuk
        );
    }

    #[test]
    fn yuzde_hesabi() {
        let s = SistemBellek {
            toplam_mb: 16_000,
            bos_mb: 4_000,
        };
        assert_eq!(s.bos_yuzde(), 25.0);
    }

    #[cfg(feature = "motor")]
    #[test]
    fn sistem_olcumu_makul_deger_veriyor() {
        // Bu test yalnız Windows'ta koşuyor. Kesin bir sayı beklemiyor —
        // makineden makineye değişir — ama "toplam RAM sıfır" ya da "boş
        // bellek toplamdan büyük" gibi bir sonuç ölçümün bozuk olduğunu
        // gösterir.
        let s = sistem();
        assert!(s.toplam_mb > 512, "toplam: {}", s.toplam_mb);
        assert!(s.bos_mb <= s.toplam_mb, "{:?}", s);
    }

    #[cfg(feature = "motor")]
    #[test]
    fn kendi_surecimizi_olcebiliyoruz() {
        // Test süreci WebView2 doğurmuyor, dolayısıyla `webview_mb` sıfır
        // olmalı; ama kendi belleğimiz okunabilmeli.
        let s = surecler();
        assert!(s.kabuk_mb > 0, "kabuk: {}", s.kabuk_mb);
    }
}
