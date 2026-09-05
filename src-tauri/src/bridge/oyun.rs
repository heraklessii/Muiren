//! Tam ekran oyun algılama — **kendi sezgimiz**, Win32'ye özel.
//!
//! `docs/Kopruler.md`: "Muifly kurulu değilse Muiren kendi başına tam ekran bir
//! oyun süreci algılamayı deniyor. Bu kabaca bir sezgi; yanlış pozitif ihtimali
//! var, o yüzden **varsayılan kapalı**."
//!
//! Faz 4'te Muifly tarafına bakıldı ve köprünün olmadığı görüldü
//! ([`super::muifly`]), yani bu dosya artık yedek değil **tek** yol.
//!
//! ## Karar saf, ölçüm değil
//!
//! `memory/olcum.rs` ile aynı ayrım: Win32 çağrıları [`win`] içinde, "bu
//! pencere bir oyun mu" kararı [`oyun_mu`] içinde ve saf. Kararı Win32
//! çağrısının yanında verseydik motorsuz derlemede test edilemezdi ve bu tam
//! olarak yanlış pozitiflerin yaşadığı yer.
//!
//! ## Sezginin kendisi
//!
//! Dört koşul birlikte:
//!
//! 1. Ön plandaki pencere **bir monitörün tamamını** kaplıyor.
//! 2. Pencere **bizim değil** — Muiren tam ekrana geçtiğinde kendini oyun
//!    sanmasın.
//! 3. Pencere **kabuğun değil** — masaüstü ve görev çubuğu her zaman ekran
//!    boyunda; bu iki süreç elenmezse oyun modu hiç kapanmazdı.
//! 4. Süreç **yüksek öncelikli** ya da pencere **kenarlıksız**. Oyunlar
//!    ikisinden birini yapıyor; tam ekran bir tarayıcı ya da video oynatıcı
//!    genelde ikisini de yapmıyor.
//!
//! Dördü de sağlanmadan oyun modu açılmıyor. Yanlış pozitifin bedeli
//! sekmelerin gereksiz uyuması, yani kullanıcının fark edeceği bir şey; bu
//! yüzden sezgi cömert değil cimri.

/// Ön plandaki pencerenin kararı verilebilir hâli.
///
/// Saf: içinde tutamak (`HWND`) yok, yalnız ölçülmüş sayılar. [`win`] bunu
/// dolduruyor, [`oyun_mu`] okuyor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OnPlan {
    /// Pencere bulunduğu monitörün tamamını kaplıyor mu.
    pub tam_ekran: bool,
    /// Pencerenin süreci Muiren mi (kendi tam ekranımız).
    pub bizim: bool,
    /// Masaüstü / görev çubuğu (`explorer.exe`).
    pub kabuk: bool,
    /// `HIGH_PRIORITY_CLASS` ya da `REALTIME_PRIORITY_CLASS`.
    pub yuksek_oncelik: bool,
    /// `WS_CAPTION` ve `WS_THICKFRAME` yok — kenarlıksız tam ekran.
    pub kenarliksiz: bool,
}

/// Ön plandaki pencere bir oyun mu.
///
/// **Saf ve testli.** Yanlış pozitiflerin yaşadığı yer burası; testler tam
/// ekran tarayıcıyı, video oynatıcıyı ve masaüstünü ayrı ayrı sabitliyor.
pub fn oyun_mu(p: OnPlan) -> bool {
    if !p.tam_ekran || p.bizim || p.kabuk {
        return false;
    }
    p.yuksek_oncelik || p.kenarliksiz
}

#[cfg(feature = "motor")]
mod win {
    use windows::Win32::Foundation::{CloseHandle, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::System::Threading::{
        GetCurrentProcessId, GetPriorityClass, OpenProcess, HIGH_PRIORITY_CLASS,
        PROCESS_QUERY_LIMITED_INFORMATION, REALTIME_PRIORITY_CLASS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId,
        GWL_STYLE, WS_CAPTION, WS_THICKFRAME,
    };

    use super::OnPlan;

    /// Kabuk sayılan süreçler. `explorer.exe` masaüstünü ve görev çubuğunu
    /// çiziyor; ikisi de her zaman ekran boyunda ve elenmezse oyun modu hiç
    /// kapanmıyor.
    const KABUK_SURECLERI: &[&str] = &["explorer.exe", "ApplicationFrameHost.exe"];

    pub fn on_plan() -> OnPlan {
        // SAFETY: hepsi salt okuma pencere/süreç sorgusu; verilen tutamaklar
        // ya API'nin döndürdüğü ya da bizim açıp kapattığımız tutamaklar.
        unsafe {
            let pencere = GetForegroundWindow();
            if pencere.0.is_null() {
                return OnPlan::default();
            }

            let mut alan = RECT::default();
            if GetWindowRect(pencere, &mut alan).is_err() {
                return OnPlan::default();
            }

            let monitor = MonitorFromWindow(pencere, MONITOR_DEFAULTTONEAREST);
            let mut bilgi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let tam_ekran = if GetMonitorInfoW(monitor, &mut bilgi).as_bool() {
                let m = bilgi.rcMonitor;
                // `<=` değil `==` değil: bazı oyunlar pencereyi bir piksel
                // taşırıyor. Kapsıyorsa yeter.
                alan.left <= m.left
                    && alan.top <= m.top
                    && alan.right >= m.right
                    && alan.bottom >= m.bottom
            } else {
                false
            };

            let mut pid = 0u32;
            GetWindowThreadProcessId(pencere, Some(&mut pid));
            let bizim = pid == GetCurrentProcessId();

            let stil = GetWindowLongPtrW(pencere, GWL_STYLE) as u32;
            let kenarliksiz = (stil & WS_CAPTION.0) == 0 && (stil & WS_THICKFRAME.0) == 0;

            OnPlan {
                tam_ekran,
                bizim,
                kabuk: kabuk_mu(pid),
                yuksek_oncelik: yuksek_oncelik(pid),
                kenarliksiz,
            }
        }
    }

    /// Süreç yüksek öncelikli mi.
    ///
    /// Açılamayan süreç `false` sayılıyor: yönetici olarak koşan bir sürecin
    /// tutamağını alamıyoruz ve "bilmiyorum"u "oyun" saymak yanlış pozitif
    /// üretirdi.
    unsafe fn yuksek_oncelik(pid: u32) -> bool {
        let Ok(tutamak) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return false;
        };
        let sinif = GetPriorityClass(tutamak);
        let _ = CloseHandle(tutamak);
        sinif == HIGH_PRIORITY_CLASS.0 || sinif == REALTIME_PRIORITY_CLASS.0
    }

    /// Süreç Windows kabuğu mu.
    ///
    /// Ada göre bakılıyor ve bu bilinçli bir istisna: `memory/olcum.rs`
    /// WebView2 süreçlerini **soy ağacına göre** süzüyor çünkü orada yanlış
    /// sayım kullanıcıya üç katı bellek gösteriyordu. Burada risk ters yönde
    /// ve küçük: `explorer.exe` adlı bir oyun oyun sayılmıyor, o kadar.
    unsafe fn kabuk_mu(pid: u32) -> bool {
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };

        let Ok(anlik) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return false;
        };
        let mut giris = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut sonuc = false;
        if Process32FirstW(anlik, &mut giris).is_ok() {
            loop {
                if giris.th32ProcessID == pid {
                    let uzunluk = giris
                        .szExeFile
                        .iter()
                        .position(|c| *c == 0)
                        .unwrap_or(giris.szExeFile.len());
                    let ad = String::from_utf16_lossy(&giris.szExeFile[..uzunluk]);
                    sonuc = KABUK_SURECLERI.iter().any(|k| ad.eq_ignore_ascii_case(k));
                    break;
                }
                if Process32NextW(anlik, &mut giris).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(anlik);
        sonuc
    }
}

/// Motorsuz derlemede pencere yok; algılama hep "oyun yok" diyor.
#[cfg(not(feature = "motor"))]
mod win {
    use super::OnPlan;

    pub fn on_plan() -> OnPlan {
        OnPlan::default()
    }
}

/// Ön planda tam ekran bir oyun var mı.
pub fn oyun_calisiyor() -> bool {
    oyun_mu(win::on_plan())
}

#[cfg(test)]
mod testler {
    use super::*;

    /// Kenarlıksız tam ekran oyun — en yaygın hâli.
    #[test]
    fn kenarliksiz_tam_ekran_oyun_sayiliyor() {
        assert!(oyun_mu(OnPlan {
            tam_ekran: true,
            kenarliksiz: true,
            ..Default::default()
        }));
    }

    #[test]
    fn yuksek_oncelikli_tam_ekran_oyun_sayiliyor() {
        assert!(oyun_mu(OnPlan {
            tam_ekran: true,
            yuksek_oncelik: true,
            ..Default::default()
        }));
    }

    #[test]
    fn kendi_tam_ekranimiz_oyun_degil() {
        // Muiren `F11` ile tam ekrana geçtiğinde kendini oyun sanıp bütün
        // sekmelerini uyutsaydı, kullanıcı videoyu tam ekran yaptığı anda
        // tarayıcısını kaybederdi.
        assert!(!oyun_mu(OnPlan {
            tam_ekran: true,
            bizim: true,
            kenarliksiz: true,
            ..Default::default()
        }));
    }

    #[test]
    fn masaustu_oyun_degil() {
        // Masaüstü her zaman ekran boyunda ve kenarlıksız. Elenmeseydi oyun
        // modu hiç kapanmazdı.
        assert!(!oyun_mu(OnPlan {
            tam_ekran: true,
            kabuk: true,
            kenarliksiz: true,
            ..Default::default()
        }));
    }

    #[test]
    fn pencereli_oyun_algilanmiyor() {
        // Bilinen sınır: pencereli kipte oynayan oyun görünmüyor. Sezgiyi
        // gevşetmek yerine kabul ediliyor — `docs/Kopruler.md` bunu "kabaca
        // bir sezgi" diye yazıyor ve yanlış pozitifin bedeli daha yüksek.
        assert!(!oyun_mu(OnPlan {
            tam_ekran: false,
            yuksek_oncelik: true,
            kenarliksiz: true,
            ..Default::default()
        }));
    }

    #[test]
    fn tam_ekran_normal_uygulama_oyun_degil() {
        // Tam ekran ama başlık çubuğu olan ve normal öncelikli bir pencere:
        // maksimize edilmiş bir metin düzenleyici.
        assert!(!oyun_mu(OnPlan {
            tam_ekran: true,
            ..Default::default()
        }));
    }
}
