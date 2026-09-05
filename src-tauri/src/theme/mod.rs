//! Temalar — `.muitema` biçimi, doğrulama ve yerleşik temalar.
//!
//! **Karar #3: tema veri, kod değil** (`docs/Roadmap.md`). Tema paketi
//! JavaScript çalıştırmıyor, rastgele CSS de çalıştırmıyor. Tema =
//! doğrulanmış jeton kümesi + doğrulanmış varlık dosyaları.
//!
//! ```text
//!   jeton.rs   renk/ölçü doğrulama + kontrast   ← güvenlik çekirdeği, saf
//!   paket.rs   .muitema okuma, doğrulama, kurma
//!   mod.rs     tipler + yerleşik üç tema
//! ```
//!
//! Yerleşik temalar da **aynı** doğrulamadan geçiyor. İstisna yok; istisna
//! olan yol, bir gün doğrulamayı atlayan yol oluyor (`docs/Temalar.md`).

pub mod jeton;
pub mod paket;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// `.muitema` arşivinin içindeki `tema.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemaDosyasi {
    /// Biçim sürümü. Bilinmeyen sürüm reddediliyor: ileride alanların anlamı
    /// değişirse eski Muiren yeni bir temayı yanlış yorumlamamalı.
    pub bicim: u32,
    pub ad: String,
    #[serde(default)]
    pub yazar: String,
    #[serde(default)]
    pub surum: String,
    #[serde(default)]
    pub jetonlar: BTreeMap<String, String>,
    #[serde(default)]
    pub arkaplan: Option<ArkaplanDosyasi>,
}

/// Desteklenen `bicim` değeri. Yeni alan eklendiğinde artıyor.
pub const BICIM: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArkaplanDosyasi {
    /// Arşivin **içindeki** dosya adı. `http(s)://`, `file://`, mutlak yol ve
    /// `..` reddediliyor (`paket::yol_guvenli`).
    pub dosya: String,
    #[serde(default = "yerlesim_varsayilan")]
    pub yerlesim: String,
    #[serde(default = "karartma_varsayilan")]
    pub karartma: f32,
    #[serde(default)]
    pub bulanik: u32,
}

fn yerlesim_varsayilan() -> String {
    "kapla".into()
}

/// Varsayılan 0.5: çoğu fotoğrafın üstünde metin okunmuyor ve bunu tema
/// yazarına bırakmak, kullanıcıyı okunmaz bir arayüzle baş başa bırakıyor
/// (`docs/Temalar.md`).
fn karartma_varsayilan() -> f32 {
    0.5
}

/// Arayüze giden, **doğrulanmış** tema.
///
/// `jetonlar` doğrudan `documentElement.style` üzerine yazılıyor. Buraya
/// giren her değer `jeton::dogrula`dan geçti; arayüz tarafında ikinci bir
/// süzgeç yok ve olmamalı — iki süzgeç, biri gevşediğinde fark edilmeyen bir
/// açık demek.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemaOzeti {
    pub ad: String,
    pub yazar: String,
    pub surum: String,
    /// Yerleşik temalar silinemiyor.
    pub yerlesik: bool,
    /// `(--jeton, deger)` çiftleri; hepsi doğrulanmış.
    pub jetonlar: Vec<(String, String)>,
    /// Arka plan görselinin **kabuk tarafından okunabilir** yolu. `None` ise
    /// arka plan yok.
    pub arkaplan: Option<Arkaplan>,
    /// Beyaz listede olmayan ya da doğrulamadan geçemeyen anahtarlar.
    /// Sessizce yok saymak, tema yazarının saatlerce "neden çalışmıyor" diye
    /// bakması demek.
    pub atilan_jetonlar: Vec<String>,
    /// Kontrast uyarıları. Tema **reddedilmiyor** ama kullanıcı görüyor
    /// (`docs/Temalar.md`).
    pub uyarilar: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arkaplan {
    /// Dosyanın diskteki tam yolu.
    pub yol: String,
    pub yerlesim: String,
    pub karartma: f32,
    pub bulanik: u32,
}

/// Arka plan görselinin üst sınırı.
///
/// Duvar kâğıtları büyük olabiliyor ve bu bayt dizisi `data:` adresi olarak
/// kabuğa gidiyor — base64 boyutu 4/3'e çıkarıyor. 8 MB'lık bir görsel
/// 10,6 MB'lık bir dizeye dönüşüyor; bunun üstü, "az RAM harcayan tarayıcı"
/// iddiasıyla çelişmeye başlıyor.
const AZAMI_ARKAPLAN: u64 = 8 * 1024 * 1024;

/// Arka plan görselini `data:` adresi olarak okur.
///
/// **Neden dosya yolu değil `data:`** — `favicon/mod.rs` ile birebir aynı
/// gerekçe: kabuğa bir dosya sistemi kanalı (Tauri `asset` protokolü)
/// açmamak için. O protokolü açmak, kabukta çalışan her satırın kullanıcının
/// diskinden okuyabilmesi demek ve bunun karşılığı bir duvar kâğıdı değil.
///
/// Yol `paket::arkaplan_dogrula` süzgecinden geçmiş: tema klasörünün içinde,
/// dizin taşması yok, uzantısı bilinen bir görsel biçimi. Burada ikinci bir
/// yol doğrulaması **yok** — iki süzgeç, biri gevşediğinde fark edilmeyen bir
/// açık demek (`docs/Temalar.md`, aynı kural jetonlar için de geçerli).
pub fn arkaplan_veri(arkaplan: &Arkaplan) -> Option<String> {
    let yol = std::path::Path::new(&arkaplan.yol);
    let boyut = std::fs::metadata(yol).ok()?.len();
    if boyut == 0 || boyut > AZAMI_ARKAPLAN {
        log::warn!("muiren: arka plan görseli atlandı ({boyut} bayt, üst sınır {AZAMI_ARKAPLAN})");
        return None;
    }
    let veri = std::fs::read(yol).ok()?;
    let tur = mime_turu(yol)?;
    Some(format!(
        "data:{tur};base64,{}",
        crate::favicon::base64_kodla(&veri)
    ))
}

/// Uzantıdan MIME türü.
///
/// Dosyanın içine bakılmıyor: uzantı zaten `paket::arkaplan_dogrula`
/// tarafından beyaz listeden geçirildi ve bu adres bir `<img>` değil bir CSS
/// `background-image`; tarayıcı türü yanlışsa görseli çizmiyor, kod
/// çalıştırmıyor.
fn mime_turu(yol: &std::path::Path) -> Option<&'static str> {
    match yol
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase()
        .as_str()
    {
        // Liste `paket::GORSEL_UZANTILARI` ile aynı olmak zorunda: orada
        // kabul edilip burada tanınmayan bir uzantı, doğrulamayı geçmiş ama
        // sessizce çizilmeyen bir arka plan demek.
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "avif" => Some("image/avif"),
        _ => None,
    }
}

/// Yerleşik üç tema (`docs/Temalar.md`).
///
/// Jetonlar burada **ham dize** olarak duruyor ve `paket::dogrula_jetonlar`
/// üzerinden geçiyor — yerleşik olmaları doğrulamayı atlamalarına sebep
/// değil.
pub fn yerlesikler() -> Vec<(&'static str, &'static [(&'static str, &'static str)])> {
    vec![("Mui", MUI), ("Mürekkep", MUREKKEP), ("Kağıt", KAGIT)]
}

/// Varsayılan tema — kardeş projelerin birebir jetonları.
const MUI: &[(&str, &str)] = &[
    ("bg", "#0f1115"),
    ("bg-panel", "#181b22"),
    ("bg-elevated", "#20242d"),
    ("bg-sunken", "#0b0d11"),
    ("border", "#262b35"),
    ("border-strong", "#333a47"),
    ("text", "#e8eaed"),
    ("text-muted", "#8b93a3"),
    ("accent", "#2dd4bf"),
    ("accent-strong", "#5eead4"),
    // Teal AÇIK bir renk: üstündeki yazı koyu olmak zorunda.
    ("on-accent", "#04211d"),
    ("radius", "12px"),
    ("radius-lg", "16px"),
];

/// Daha koyu, düşük kontrastlı, gece için.
const MUREKKEP: &[(&str, &str)] = &[
    ("bg", "#080a0f"),
    ("bg-panel", "#0f1219"),
    ("bg-elevated", "#151923"),
    ("bg-sunken", "#05070a"),
    ("border", "#1b2029"),
    ("border-strong", "#272d3a"),
    ("text", "#c9cfda"),
    // Kontrast testi 3.98 gösterdiği için açıldı: eşik altında kalan bir
    // yerleşik tema, kuralın kendisini anlamsız yapardı.
    ("text-muted", "#828c9d"),
    ("accent", "#3aa9a0"),
    ("accent-strong", "#4fc4ba"),
    ("on-accent", "#02120f"),
    ("radius", "10px"),
    ("radius-lg", "14px"),
];

/// Açık tema. Mui ailesinde açık tema yok; burada var çünkü tarayıcı gündüz
/// de kullanılıyor ve sayfaların çoğu beyaz — koyu kabuk + beyaz sayfa göz
/// yoruyor (`docs/Temalar.md`).
const KAGIT: &[(&str, &str)] = &[
    ("bg", "#eef1f5"),
    ("bg-panel", "#ffffff"),
    ("bg-elevated", "#f2f4f8"),
    ("bg-sunken", "#e7eaf0"),
    ("border", "#dde1e9"),
    ("border-strong", "#c3c9d4"),
    ("text", "#12151a"),
    ("text-muted", "#5d6675"),
    // Açık temada `on-accent` beyaz olmak zorunda, dolayısıyla `accent`
    // aile teal'inden daha koyu: `#0d9488` üstünde beyaz yazı 3.74:1 ile
    // eşiğin altında kalıyordu (kontrast testi yakaladı). `docs/Temalar.md`
    // içindeki `on-accent` tuzağının açık tema hâli.
    ("accent", "#0a6b62"),
    ("accent-strong", "#085850"),
    ("on-accent", "#ffffff"),
    ("radius", "12px"),
    ("radius-lg", "16px"),
];

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn yerlesik_temalarin_hepsi_kendi_dogrulamamizdan_geciyor() {
        // Kendi temamız kendi kuralımızdan geçmezse kural yanlış demektir.
        for (ad, jetonlar) in yerlesikler() {
            let harita: BTreeMap<String, String> = jetonlar
                .iter()
                .map(|(a, d)| (a.to_string(), d.to_string()))
                .collect();
            let (kabul, atilan) = jeton::dogrula(&harita);
            assert!(atilan.is_empty(), "{ad}: atılan jeton {atilan:?}");
            assert_eq!(kabul.len(), jetonlar.len(), "{ad}");
        }
    }

    #[test]
    fn yerlesik_temalar_kontrast_esigini_geciyor() {
        for (ad, jetonlar) in yerlesikler() {
            let bul = |k: &str| {
                jeton::renk_dogrula(jetonlar.iter().find(|(a, _)| *a == k).unwrap().1).unwrap()
            };
            let cift = [
                ("text/bg", bul("text"), bul("bg")),
                ("text-muted/bg-panel", bul("text-muted"), bul("bg-panel")),
                ("on-accent/accent", bul("on-accent"), bul("accent")),
            ];
            for (etiket, a, b) in cift {
                assert!(
                    jeton::kontrast(a, b) >= jeton::KONTRAST_ESIGI,
                    "{ad} · {etiket}: {:.2}",
                    jeton::kontrast(a, b)
                );
            }
        }
    }

    #[test]
    fn yerlesik_temalarin_adlari_benzersiz() {
        let mut adlar: Vec<&str> = yerlesikler().iter().map(|(a, _)| *a).collect();
        adlar.sort_unstable();
        let sayi = adlar.len();
        adlar.dedup();
        assert_eq!(adlar.len(), sayi);
    }

    #[test]
    fn her_yerlesik_tema_ayni_jeton_kumesini_tanimliyor() {
        // Bir temada eksik jeton, o jetonun `styles.css` varsayılanında
        // kalması demek: tema yarım uygulanmış görünüyor.
        let ilk: Vec<&str> = MUI.iter().map(|(a, _)| *a).collect();
        for (ad, jetonlar) in yerlesikler() {
            let bu: Vec<&str> = jetonlar.iter().map(|(a, _)| *a).collect();
            assert_eq!(bu, ilk, "{ad}");
        }
    }
}

#[cfg(test)]
mod arkaplan_testleri {
    use super::*;

    fn ornek(yol: &std::path::Path) -> Arkaplan {
        Arkaplan {
            yol: yol.to_string_lossy().into_owned(),
            yerlesim: "kapla".into(),
            karartma: 0.0,
            bulanik: 0,
        }
    }

    #[test]
    fn gorsel_data_adresine_ceviriliyor() {
        let d = tempfile::tempdir().unwrap();
        let yol = d.path().join("arkaplan.png");
        std::fs::write(&yol, b"sahte-png-baytlari").unwrap();

        let adres = arkaplan_veri(&ornek(&yol)).unwrap();
        assert!(adres.starts_with("data:image/png;base64,"), "{adres}");
    }

    #[test]
    fn jpeg_dogru_mime_aliyor() {
        let d = tempfile::tempdir().unwrap();
        let yol = d.path().join("a.JPG");
        std::fs::write(&yol, b"x").unwrap();
        assert!(arkaplan_veri(&ornek(&yol))
            .unwrap()
            .starts_with("data:image/jpeg;"));
    }

    #[test]
    fn cok_buyuk_gorsel_atlaniyor() {
        let d = tempfile::tempdir().unwrap();
        let yol = d.path().join("dev.png");
        std::fs::write(&yol, vec![0u8; (AZAMI_ARKAPLAN + 1) as usize]).unwrap();
        assert!(arkaplan_veri(&ornek(&yol)).is_none());
    }

    #[test]
    fn bos_dosya_atlaniyor() {
        let d = tempfile::tempdir().unwrap();
        let yol = d.path().join("bos.png");
        std::fs::write(&yol, b"").unwrap();
        assert!(arkaplan_veri(&ornek(&yol)).is_none());
    }

    #[test]
    fn olmayan_dosya_patlamiyor() {
        assert!(arkaplan_veri(&ornek(std::path::Path::new("yok.png"))).is_none());
    }

    #[test]
    fn taninmayan_uzanti_reddediliyor() {
        // `paket::GORSEL_UZANTILARI` bunu zaten geçirmiyor; ikinci kapı
        // burada olsun ki liste ayrıştığında sessiz kalmasın.
        let d = tempfile::tempdir().unwrap();
        let yol = d.path().join("a.svg");
        std::fs::write(&yol, b"<svg/>").unwrap();
        assert!(arkaplan_veri(&ornek(&yol)).is_none());
    }
}
