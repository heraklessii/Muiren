//! Kalıcı tercihler.
//!
//! Üç fonksiyon, **üçü birlikte değişiyor** (CLAUDE.md #13): [`duzelt`],
//! [`oku`], [`yaz`]. Yeni bir anahtar eklenip [`duzelt`] unutulduğunda geçersiz
//! bir değer doğrudan motora gidiyor ve hata çalışma zamanına kadar görünmüyor.
//!
//! **Bellek eşikleri burada, kodda değil** (CLAUDE.md #6): 8 GB'lık bir
//! dizüstü ile 32 GB'lık bir masaüstü aynı politikayı kaldırmıyor. Eşiği koda
//! gömmek, kullanıcının makinesine göre ayarlanamayan bir tarayıcı demek.
//!
//! [`oku`] **asla hata döndürmüyor**: bozuk bir ayar dosyası yüzünden
//! tarayıcının açılmaması kabul edilemez. Bozuk dosya yedekleniyor ve
//! varsayılana dönülüyor (`docs/Depolama.md`).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::hata::Sonuc;

/// Makinenin RAM'ine göre seçilen eşik profili (`docs/Bellek.md` tablosu).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Profil {
    /// ≤ 8 GB — uyku 2 dk, atma 20 dk, uyanık üst sınır 6.
    Siki,
    /// 8–24 GB — uyku 8 dk, atma 60 dk, uyanık üst sınır 14.
    Dengeli,
    /// > 24 GB — uyku 20 dk, atma 3 sa, uyanık üst sınır 30.
    Rahat,
}

/// Chromium süreç modeli. Değişince **yeniden başlatma** gerekiyor: bu ayar
/// `EK_BAYRAKLAR`a dönüşüyor ve bayraklar ancak WebView2 ortamı yeniden
/// kurulduğunda etkili oluyor (`docs/Depolama.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SurecPolitikasi {
    /// `--process-per-site` açık. Bir sitenin sekmesi çökerse o sitenin tüm
    /// sekmeleri gidiyor — URL'den geri geliyorlar.
    Birlesik,
    /// Chromium'un kendi modeli. Daha çok RAM, daha iyi çökme yalıtımı.
    Varsayilan,
}

/// İndirmeyi kim yapacak (`docs/Kopruler.md`, Muiget köprüsü).
///
/// Muiget kurulu **değilse** üçü de aynı sonucu veriyor: motorun kendi
/// indirmesi. Karar #4'ün ikinci yarısı bu — "Muiget kurulu değilse motorun
/// kendi indirmesi devrede kalıyor. Kullanıcı indirme yapamaz duruma
/// düşmüyor."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IndirmePolitikasi {
    /// Her indirme doğrudan Muiget'e.
    Muiget,
    /// Her seferinde sorulsun (`muiren://indirme-onerisi`).
    Sor,
    /// Köprü hiç kullanılmasın; WebView2 kendi indirmesini yapsın.
    Motorda,
}

/// **`#[serde(default)]` kasıtlı.** Eksik bir anahtar dosyayı bozuk yapmıyor,
/// yalnız o alan varsayılana düşüyor. Olmasaydı sürüm yükseltmede eklenen her
/// yeni alan, kullanıcının bütün ayar dosyasını `.bozuk` yapıp sıfırlardı —
/// eşiklerini elle ayarlamış birinin gözünde bu bir veri kaybı
/// (`docs/Depolama.md`). Oturum dosyası aynı korumayı alan alan alıyor
/// (`tabs/oturum.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub profil: Profil,

    // --- Bellek politikası (Faz 2'de gözcü bunları okuyor) ---
    pub uyku_esigi_sn: u64,
    pub atma_esigi_sn: u64,
    pub uyanik_ust_sinir: u32,
    pub gozcu_periyodu_sn: u64,
    /// Bu alan adlarındaki sekmeler uyutulmuyor/atılmıyor (koruma kuralı #6:
    /// web posta, izleme panelleri).
    pub istisna_alanlari: Vec<String>,

    // --- Süreç modeli (yeniden başlatma gerektiriyor) ---
    pub surec_politikasi: SurecPolitikasi,
    /// `--renderer-process-limit=N`. `0` ise bayrak hiç geçirilmiyor.
    pub renderer_tavani: u32,

    // --- Kabuk ---
    /// Adres çubuğuna URL olmayan bir şey yazıldığında kullanılan şablon.
    /// `%s` sorgunun yerini tutuyor.
    ///
    /// Varsayılan Google. Muiren'in kendisi arama motoruna **hiçbir şey**
    /// göndermiyor: canlı öneri API'si kullanılmıyor, her tuş vuruşu bir
    /// sunucuya gitmiyor (`docs/Frontend.md`). Sorgu yalnız kullanıcı Enter'a
    /// bastığında ve sayfanın kendi isteği olarak gidiyor. Şablon ayarlardan
    /// değiştirilebiliyor.
    pub arama_url: String,
    pub tema: String,
    /// `-1` geçmiş tutma, `0` sınırsız, `1..=3650` gün.
    pub gecmis_saklama_gun: i32,

    // --- köprüler (Faz 3–5) ---
    pub indirme_politikasi: IndirmePolitikasi,

    // --- engelleme (Faz 4) ---
    /// Pop-up ve kullanıcı hareketi olmadan başlayan yönlendirme engelleme.
    /// **Varsayılan açık** (`docs/Roadmap.md` Faz 4): bir politika, liste
    /// değil; kullanıcının bir şey yazmasını gerektirmiyor.
    pub engelleme_acik: bool,
    /// Kullanıcının filtre listeleri. **Varsayılan kapalı** ve kutudan çıkan
    /// liste yok (karar #5: "Filtre listeleri bile kullanıcı tarafından
    /// konuyor").
    pub filtre_acik: bool,
    /// Filtre kuralları, satır satır (`engel/liste.rs` biçimi).
    pub filtre_kurallari: Vec<String>,
    /// Tam ekran oyun algılandığında sekmeler uyutulsun mu.
    ///
    /// **Varsayılan kapalı** (`docs/Kopruler.md`): algılama kabaca bir sezgi
    /// ve yanlış pozitifin bedeli, kullanıcının baktığı sekmelerin sebepsiz
    /// uyuması. Kapalıyken oyun modu yalnız elle açılıyor.
    pub oyun_algilama: bool,
    /// Oyun modu açıkken kabuk penceresi simge durumuna alınsın mı
    /// (`docs/Bellek.md`, "Oyun modu" 4. adım).
    ///
    /// **Varsayılan kapalı.** Kullanıcının penceresini kendiliğinden
    /// oynatmak geri alınması zor bir davranış; istemeyen biri özelliğin
    /// tamamını kapatır. Açıkken bile `hide()` değil `minimize()`
    /// kullanılıyor — gizlenen pencerenin görev çubuğu düğmesi de kaybolur
    /// ve tam ekran bir oyun ekranı kaplarken kullanıcının geri dönecek
    /// görünür bir yolu kalmazdı.
    pub oyun_pencere_gizle: bool,
}

impl Default for Settings {
    fn default() -> Self {
        varsayilan(Profil::Dengeli)
    }
}

/// Toplam RAM'e göre profil. Saf — ölçümü `memory/olcum.rs` yapıyor (Faz 2).
pub fn varsayilan_profil(toplam_ram_mb: u64) -> Profil {
    match toplam_ram_mb {
        0..=8192 => Profil::Siki,
        8193..=24576 => Profil::Dengeli,
        _ => Profil::Rahat,
    }
}

/// Profilin `docs/Bellek.md` tablosundaki eşikleri.
pub fn varsayilan(profil: Profil) -> Settings {
    let (uyku, atma, ust_sinir) = match profil {
        Profil::Siki => (120, 20 * 60, 6),
        Profil::Dengeli => (8 * 60, 60 * 60, 14),
        Profil::Rahat => (20 * 60, 3 * 60 * 60, 30),
    };
    Settings {
        profil,
        uyku_esigi_sn: uyku,
        atma_esigi_sn: atma,
        uyanik_ust_sinir: ust_sinir,
        gozcu_periyodu_sn: 10,
        istisna_alanlari: Vec::new(),
        surec_politikasi: SurecPolitikasi::Birlesik,
        renderer_tavani: 0,
        arama_url: "https://www.google.com/search?q=%s".into(),
        tema: "Mui".into(),
        gecmis_saklama_gun: 90,
        // Muiget kurulu değilse bu seçim arayüzde görünmüyor bile; kuruluysa
        // kullanıcının ilk indirmede karar vermesi, sessizce başka bir
        // programa devretmekten dürüst.
        indirme_politikasi: IndirmePolitikasi::Sor,
        engelleme_acik: true,
        filtre_acik: false,
        filtre_kurallari: Vec::new(),
        oyun_algilama: false,
        oyun_pencere_gizle: false,
    }
}

/// Aralık dışı değerleri sınıra çeker.
///
/// `ayarlar_yaz` **düzeltilmiş** ayarları geri döndürüyor ve arayüz düzeltilmiş
/// değeri gösteriyor: kullanıcının yazdığından sessizce farklı bir değerle
/// çalışmak yok (`docs/IPC.md`).
pub fn duzelt(ham: Settings) -> Settings {
    let uyku = ham.uyku_esigi_sn.clamp(30, 86_400);

    Settings {
        profil: ham.profil,
        uyku_esigi_sn: uyku,
        // Atma eşiği uyku eşiğinin ALTINA inemez. İnseydi sekmeler uyumadan
        // atılırdı; kullanıcı her sekme değişiminde sayfanın yeniden
        // yüklendiğini görür ve sebebini bulamazdı. Sessiz ve çok sinir bozucu
        // bir hata (`docs/Depolama.md`).
        atma_esigi_sn: ham.atma_esigi_sn.clamp(uyku, 604_800),
        uyanik_ust_sinir: ham.uyanik_ust_sinir.clamp(1, 200),
        gozcu_periyodu_sn: ham.gozcu_periyodu_sn.clamp(2, 120),
        istisna_alanlari: ham
            .istisna_alanlari
            .into_iter()
            .map(|a| a.trim().to_lowercase())
            .filter(|a| !a.is_empty())
            .collect(),
        surec_politikasi: ham.surec_politikasi,
        renderer_tavani: match ham.renderer_tavani {
            0 => 0,
            n => n.clamp(4, 64),
        },
        arama_url: if ham.arama_url.contains("%s") && ham.arama_url.starts_with("https://") {
            ham.arama_url
        } else {
            // Şablonda `%s` yoksa arama kutusu sessizce hep aynı sayfayı
            // açardı. `http://` reddediliyor: arama sorgusu düz metin gider.
            varsayilan(ham.profil).arama_url
        },
        tema: if ham.tema.trim().is_empty() {
            "Mui".into()
        } else {
            ham.tema
        },
        gecmis_saklama_gun: match ham.gecmis_saklama_gun {
            -1 | 0 => ham.gecmis_saklama_gun,
            n if (1..=3650).contains(&n) => n,
            _ => 90,
        },
        // İkisinin de düzeltilecek bir aralığı yok (bir enum ve bir bool);
        // yine de **açıkça** taşınıyorlar. `..ham` ile geçmek, bir sonraki
        // alanın `duzelt` içinde unutulmasını sessizleştirirdi
        // (CLAUDE.md #13).
        indirme_politikasi: ham.indirme_politikasi,
        engelleme_acik: ham.engelleme_acik,
        filtre_acik: ham.filtre_acik,
        // Kural metni burada AYRIŞTIRILMIYOR, yalnız boş satırlar
        // atılıyor: ayrıştırma `engel/liste.rs` içinde ve orası
        // anlaşılmayan satırları kullanıcıya geri gösteriyor. Burada
        // sessizce elemek, kullanıcının yazdığı kuralın kaybolması
        // demek olurdu.
        filtre_kurallari: ham
            .filtre_kurallari
            .into_iter()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect(),
        oyun_algilama: ham.oyun_algilama,
        oyun_pencere_gizle: ham.oyun_pencere_gizle,
    }
}

/// **Asla hata döndürmüyor.** Bozuk dosya `.bozuk` uzantısıyla saklanıyor ki
/// kullanıcı ne kaybettiğini görebilsin.
pub fn oku(yol: &Path) -> Settings {
    let Ok(veri) = fs::read(yol) else {
        return Settings::default();
    };
    match serde_json::from_slice::<Settings>(crate::bom_kirp(&veri)) {
        Ok(s) => duzelt(s),
        Err(_) => {
            let _ = fs::rename(yol, yol.with_extension("json.bozuk"));
            Settings::default()
        }
    }
}

pub fn yaz(yol: &Path, s: &Settings) -> Sonuc<()> {
    if let Some(dizin) = yol.parent() {
        fs::create_dir_all(dizin)?;
    }
    fs::write(yol, serde_json::to_vec_pretty(s)?)?;
    Ok(())
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn profil_esikleri_bellek_belgesiyle_ayni() {
        assert_eq!(varsayilan(Profil::Siki).uyku_esigi_sn, 120);
        assert_eq!(varsayilan(Profil::Dengeli).uyku_esigi_sn, 480);
        assert_eq!(varsayilan(Profil::Rahat).atma_esigi_sn, 10_800);
        assert_eq!(varsayilan(Profil::Siki).uyanik_ust_sinir, 6);
    }

    #[test]
    fn ram_profili_seciyor() {
        assert_eq!(varsayilan_profil(8192), Profil::Siki);
        assert_eq!(varsayilan_profil(16384), Profil::Dengeli);
        assert_eq!(varsayilan_profil(32768), Profil::Rahat);
    }

    #[test]
    fn sifir_uyku_esigi_alt_sinira_cekiliyor() {
        let s = duzelt(Settings {
            uyku_esigi_sn: 0,
            ..Default::default()
        });
        assert_eq!(s.uyku_esigi_sn, 30);
    }

    #[test]
    fn atma_esigi_uyku_esiginin_altina_inemiyor() {
        let s = duzelt(Settings {
            uyku_esigi_sn: 600,
            atma_esigi_sn: 60,
            ..Default::default()
        });
        assert_eq!(s.atma_esigi_sn, 600);
    }

    #[test]
    fn renderer_tavani_ya_sifir_ya_dort_ustu() {
        let tavan = |n| {
            duzelt(Settings {
                renderer_tavani: n,
                ..Default::default()
            })
            .renderer_tavani
        };
        assert_eq!(tavan(0), 0, "0 = sınırsız, bayrak geçirilmiyor");
        assert_eq!(tavan(2), 4);
        assert_eq!(tavan(128), 64);
    }

    #[test]
    fn sablonsuz_arama_adresi_reddediliyor() {
        let s = duzelt(Settings {
            arama_url: "https://ornek.com/ara".into(),
            ..Default::default()
        });
        assert!(s.arama_url.contains("%s"));
    }

    #[test]
    fn duz_http_arama_adresi_reddediliyor() {
        let s = duzelt(Settings {
            arama_url: "http://ornek.com/?q=%s".into(),
            ..Default::default()
        });
        assert!(s.arama_url.starts_with("https://"));
    }

    #[test]
    fn gecmis_saklama_ozel_degerleri_koruyor() {
        let gun = |n| {
            duzelt(Settings {
                gecmis_saklama_gun: n,
                ..Default::default()
            })
            .gecmis_saklama_gun
        };
        assert_eq!(gun(-1), -1, "-1: geçmiş tutma");
        assert_eq!(gun(0), 0, "0: sınırsız");
        assert_eq!(gun(45), 45);
        assert_eq!(gun(-9), 90, "geçersiz → varsayılan");
        assert_eq!(gun(99_999), 90);
    }

    #[test]
    fn istisna_alanlari_normalize_ediliyor() {
        let s = duzelt(Settings {
            istisna_alanlari: vec!["  Mail.Example.COM ".into(), "   ".into()],
            ..Default::default()
        });
        assert_eq!(s.istisna_alanlari, vec!["mail.example.com"]);
    }

    #[test]
    fn engelleme_varsayilan_acik_filtre_kapali() {
        // `docs/Roadmap.md` Faz 4 ve karar #5: politika açık, liste kapalı.
        let s = Settings::default();
        assert!(s.engelleme_acik, "pop-up engelleme varsayılan kapalı");
        assert!(!s.filtre_acik, "filtre varsayılan açık");
        assert!(s.filtre_kurallari.is_empty(), "kutudan liste çıkıyor");
    }

    #[test]
    fn filtre_kurallarindan_bos_satirlar_atiliyor() {
        let s = duzelt(Settings {
            filtre_kurallari: vec!["  reklam.example.com ".into(), "   ".into(), "".into()],
            ..Default::default()
        });
        assert_eq!(s.filtre_kurallari, vec!["reklam.example.com"]);
    }

    #[test]
    fn anlasilmayan_kural_duzeltmede_silinmiyor() {
        // Ayrıştırma `engel/liste.rs` içinde ve orası anlaşılmayan satırları
        // kullanıcıya geri gösteriyor. Burada sessizce elemek, kullanıcının
        // yazdığı kuralın kaybolması demek olurdu.
        let s = duzelt(Settings {
            filtre_kurallari: vec!["bu bir kural değil".into()],
            ..Default::default()
        });
        assert_eq!(s.filtre_kurallari, vec!["bu bir kural değil"]);
    }

    #[test]
    fn oyun_algilama_varsayilan_kapali() {
        // `docs/Kopruler.md`: sezgi kaba, yanlış pozitifin bedeli yüksek.
        assert!(!Settings::default().oyun_algilama);
    }

    #[test]
    fn oyun_pencere_gizle_varsayilan_kapali() {
        // Kullanıcının penceresini kendiliğinden oynatmak geri alınması zor
        // (`docs/Bellek.md`, "Oyun modu" 4. adım).
        assert!(!Settings::default().oyun_pencere_gizle);
    }

    #[test]
    fn eski_ayar_dosyasi_eksik_anahtarla_okunuyor() {
        // Sürüm yükseltmede eklenen yeni alan, kullanıcının bütün ayar
        // dosyasını `.bozuk` yapıp sıfırlamamalı: eşiklerini elle ayarlamış
        // biri için bu bir veri kaybı. `#[serde(default)]` bu testin konusu.
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("ayarlar.json");
        fs::write(
            &yol,
            br#"{"profil":"siki","uykuEsigiSn":300,"atmaEsigiSn":1200}"#,
        )
        .unwrap();

        let s = oku(&yol);
        assert_eq!(s.uyku_esigi_sn, 300, "yazılmış değer korunuyor");
        assert_eq!(s.profil, Profil::Siki);
        assert!(!s.oyun_pencere_gizle, "yeni alan varsayılana düşüyor");
        assert_eq!(
            s.arama_url,
            Settings::default().arama_url,
            "eksik alanlar varsayılandan geliyor"
        );
        assert!(
            !yol.with_extension("json.bozuk").exists(),
            "eksik anahtar dosyayı bozuk yapmıyor"
        );
    }

    #[test]
    fn duzelt_kararli() {
        // İki kez düzeltmek bir kez düzeltmekle aynı sonucu vermeli; yoksa
        // her kaydetmede değer kayıyor.
        let bir = duzelt(Settings {
            uyku_esigi_sn: 5,
            atma_esigi_sn: 1,
            renderer_tavani: 2,
            ..Default::default()
        });
        assert_eq!(duzelt(bir.clone()), bir);
    }

    #[test]
    fn bozuk_dosya_varsayilana_donuyor() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("ayarlar.json");
        fs::write(&yol, b"{ bozuk").unwrap();

        assert_eq!(oku(&yol), Settings::default());
        assert!(
            yol.with_extension("json.bozuk").exists(),
            "bozuk dosya saklanmadı"
        );
    }

    #[test]
    fn bom_ile_yazilmis_dosya_okunuyor() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("ayarlar.json");
        let mut veri = vec![0xEF, 0xBB, 0xBF];
        veri.extend_from_slice(&serde_json::to_vec(&Settings::default()).unwrap());
        fs::write(&yol, veri).unwrap();

        assert_eq!(oku(&yol), Settings::default());
        assert!(
            !yol.with_extension("json.bozuk").exists(),
            "geçerli dosya bozuk sayıldı"
        );
    }

    #[test]
    fn yaz_oku_gidis_donus() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("ayarlar.json");
        let s = duzelt(Settings {
            uyku_esigi_sn: 45,
            ..Default::default()
        });
        yaz(&yol, &s).unwrap();
        assert_eq!(oku(&yol), s);
    }
}
