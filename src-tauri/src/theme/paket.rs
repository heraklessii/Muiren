//! `.muitema` paketi — okuma, **doğrulama** ve kurma.
//!
//! > **Kural (CLAUDE.md #14):** `tema.json` içine yeni bir alan eklendiğinde
//! > buraya karşılığı ve testi eklenmeden birleştirilmiyor. Doğrulanmayan
//! > alan, dışarıdan indirilen bir dosyanın arayüzü ele geçirmesi demek.
//!
//! Akış (`docs/Temalar.md`):
//!
//! ```text
//! kullanıcı .muitema seçiyor
//!   → ac()        ZIP aç, boyut sınırlarını uygula
//!   → dogrula()   şema + her jeton + arka plan yolu + kontrast
//!      ├─ hata → tema uygulanmıyor, sebep kullanıcıya gösteriliyor
//!      └─ tamam ↓
//!   → kur()       veri klasörüne kopyalanıyor (orijinal dosyaya bağlı kalmıyor)
//! ```

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::jeton::{self, KONTRAST_ESIGI};
use super::{Arkaplan, ArkaplanDosyasi, TemaDosyasi, TemaOzeti, BICIM};
use crate::hata::{MuirenHata, Sonuc};

/// Arşivin tamamı için üst sınır. Sıkıştırılmış bir arşivin açıldığında
/// diski doldurması ("zip bomb") klasik saldırı; sınır hem sıkıştırılmış hem
/// açılmış boyuta uygulanıyor.
const ARSIV_SINIRI: u64 = 32 * 1024 * 1024;
/// Tek bir varlık dosyası için üst sınır (`docs/Temalar.md`: görsel ≤ 8 MB).
const VARLIK_SINIRI: u64 = 8 * 1024 * 1024;
/// `tema.json` için üst sınır. Birkaç KB'lık bir dosya; 1 MB fazlasıyla bol
/// ve ayrıştırıcıyı devasa bir girdiyle boğmayı engelliyor.
const JSON_SINIRI: u64 = 1024 * 1024;

/// Arka planda kabul edilen uzantılar. Kod çözme motorun işi; kendi
/// çözücümüzü yazmıyoruz (`docs/Temalar.md`).
const GORSEL_UZANTILARI: &[&str] = &["jpg", "jpeg", "png", "webp", "avif"];

fn bicim_hatasi(m: impl std::fmt::Display) -> MuirenHata {
    MuirenHata::Bicim(m.to_string())
}

/// Arşiv içi yolun güvenli olup olmadığı.
///
/// **Dizin taşmasının kapandığı yer.** Reddedilenler:
///
/// - `..` içeren yollar — `../../../Windows/System32/...`
/// - mutlak yollar (`/x`, `C:\x`)
/// - şema içerenler (`http://`, `file://`) — uzak varlık yok, hem gizlilik
///   (her yeni sekmede sunucuya sinyal) hem "bulut yok" kuralı
/// - alt dizinler — arşiv düz, tek seviye
pub fn yol_guvenli(ad: &str) -> bool {
    let a = ad.trim();
    !a.is_empty()
        && !a.contains("..")
        && !a.contains('/')
        && !a.contains('\\')
        && !a.contains(':')
        && !a.starts_with('.')
}

/// Ham jeton haritasını doğrular ve kontrast uyarılarını üretir.
///
/// Ayrı fonksiyon çünkü **yerleşik temalar da buradan geçiyor**: istisna olan
/// yol, bir gün doğrulamayı atlayan yol oluyor.
pub fn dogrula_jetonlar(
    ham: &BTreeMap<String, String>,
) -> (Vec<(String, String)>, Vec<String>, Vec<String>) {
    let (kabul, atilan) = jeton::dogrula(ham);

    // Kontrast: tema reddedilmiyor ama kullanıcı uyarılıyor ve tek tıkla geri
    // alabiliyor (`docs/Temalar.md`).
    let mut uyarilar = Vec::new();
    let oku = |k: &str| ham.get(k).and_then(|v| jeton::renk_dogrula(v));
    let ciftler = [
        (
            "Metin arka planın üstünde okunmuyor",
            oku("text"),
            oku("bg"),
        ),
        (
            "Soluk metin panel üstünde okunmuyor",
            oku("text-muted"),
            oku("bg-panel"),
        ),
        (
            "Birincil düğmelerin yazısı okunmuyor",
            oku("on-accent"),
            oku("accent"),
        ),
    ];
    for (mesaj, a, b) in ciftler {
        if let (Some(x), Some(y)) = (a, b) {
            let oran = jeton::kontrast(x, y);
            if oran < KONTRAST_ESIGI {
                uyarilar.push(format!("{mesaj} (kontrast {oran:.1}:1, en az 4.5 olmalı)"));
            }
        }
    }

    (kabul, atilan, uyarilar)
}

/// Yerleşik bir temayı özet hâline getirir.
pub fn yerlesik_ozet(ad: &str, jetonlar: &[(&str, &str)]) -> TemaOzeti {
    let harita: BTreeMap<String, String> = jetonlar
        .iter()
        .map(|(a, d)| (a.to_string(), d.to_string()))
        .collect();
    let (kabul, atilan, uyarilar) = dogrula_jetonlar(&harita);
    TemaOzeti {
        ad: ad.to_string(),
        yazar: "Mui".into(),
        surum: env!("CARGO_PKG_VERSION").to_string(),
        yerlesik: true,
        jetonlar: kabul,
        arkaplan: None,
        atilan_jetonlar: atilan,
        uyarilar,
    }
}

/// `tema.json` metnini doğrulanmış özete çevirir.
///
/// `arkaplan_kok` verilirse arka plan dosyasının tam yolu oradan kuruluyor.
pub fn dogrula(json: &str, arkaplan_kok: Option<&Path>) -> Sonuc<TemaOzeti> {
    let d: TemaDosyasi = serde_json::from_str(json).map_err(bicim_hatasi)?;

    // Bilinmeyen biçim sürümü reddediliyor: ileride alanların anlamı
    // değişirse eski Muiren yeni bir temayı yanlış yorumlamamalı.
    if d.bicim != BICIM {
        return Err(bicim_hatasi(format!(
            "desteklenmeyen tema biçimi: {} (beklenen {BICIM})",
            d.bicim
        )));
    }

    let ad = d.ad.trim();
    if ad.is_empty() || ad.chars().count() > 60 {
        return Err(bicim_hatasi("tema adı boş ya da çok uzun"));
    }
    // Ad dosya sistemine klasör adı olarak gidiyor: dizin taşmasını burada da
    // kapatıyoruz.
    if !ad_guvenli(ad) {
        return Err(bicim_hatasi("tema adında kullanılamayan karakter var"));
    }

    let (kabul, atilan, uyarilar) = dogrula_jetonlar(&d.jetonlar);
    if kabul.is_empty() {
        return Err(bicim_hatasi("temada geçerli hiçbir jeton yok"));
    }

    let arkaplan = match (&d.arkaplan, arkaplan_kok) {
        (Some(a), Some(kok)) => Some(arkaplan_dogrula(a, kok)?),
        // Arka plan bildirilmiş ama kök verilmemiş: doğrulama aşamasında
        // (dosya henüz kurulmadan) normal.
        _ => None,
    };

    Ok(TemaOzeti {
        ad: ad.to_string(),
        yazar: d.yazar.trim().chars().take(60).collect(),
        surum: d.surum.trim().chars().take(20).collect(),
        yerlesik: false,
        jetonlar: kabul,
        arkaplan,
        atilan_jetonlar: atilan,
        uyarilar,
    })
}

/// Tema adı klasör adı olarak kullanılabilir mi.
///
/// Ayrı bir kontrol çünkü `yol_guvenli` arşiv içi dosya adları için; burada
/// kullanıcıya gösterilen bir ad var ve boşluk/Türkçe harf serbest olmalı.
fn ad_guvenli(ad: &str) -> bool {
    !ad.contains("..")
        && !ad.contains('/')
        && !ad.contains('\\')
        && !ad.contains(':')
        && !ad.starts_with('.')
        && !ad.chars().any(|c| c.is_control() || "<>\"|?*".contains(c))
}

fn arkaplan_dogrula(a: &ArkaplanDosyasi, kok: &Path) -> Sonuc<Arkaplan> {
    if !yol_guvenli(&a.dosya) {
        return Err(bicim_hatasi("arka plan dosya yolu güvenli değil"));
    }
    let uzanti = Path::new(&a.dosya)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if !GORSEL_UZANTILARI.contains(&uzanti.as_str()) {
        return Err(bicim_hatasi(format!(
            "arka plan biçimi desteklenmiyor: {uzanti}"
        )));
    }

    let yol = kok.join(&a.dosya);
    Ok(Arkaplan {
        yol: yol.to_string_lossy().into_owned(),
        yerlesim: match a.yerlesim.as_str() {
            "kapla" | "sigdir" | "dose" | "ortala" => a.yerlesim.clone(),
            // Tanınmayan yerleşim varsayılana düşüyor; tema reddedilmiyor.
            _ => "kapla".into(),
        },
        // Üst sınır 0.9: tam karartma arka planı görünmez yapardı ve
        // kullanıcı temayı uygulamış ama hiçbir şey değişmemiş sanırdı.
        karartma: a.karartma.clamp(0.0, 0.9),
        bulanik: a.bulanik.min(40),
    })
}

/// `.muitema` arşivini açar, doğrular ve veri klasörüne kurar.
///
/// Kurulum **kopyalıyor**: tema uygulandıktan sonra kullanıcı indirdiği
/// dosyayı silebilmeli (`docs/Temalar.md`).
pub fn kur(arsiv_yolu: &Path, temalar_dizini: &Path) -> Sonuc<TemaOzeti> {
    let dosya = std::fs::File::open(arsiv_yolu)?;
    let boyut = dosya.metadata()?.len();
    if boyut > ARSIV_SINIRI {
        return Err(bicim_hatasi("tema arşivi çok büyük"));
    }

    let mut arsiv = zip::ZipArchive::new(dosya).map_err(bicim_hatasi)?;

    // Açılmış toplam boyut da sınırlı: sıkıştırılmış küçük ama açıldığında
    // diski dolduran arşiv ("zip bomb") klasik saldırı.
    let acilmis: u64 = (0..arsiv.len())
        .filter_map(|i| arsiv.by_index(i).ok().map(|f| f.size()))
        .sum();
    if acilmis > ARSIV_SINIRI {
        return Err(bicim_hatasi("tema arşivi açıldığında çok büyük"));
    }

    let json = {
        let mut f = arsiv
            .by_name("tema.json")
            .map_err(|_| bicim_hatasi("arşivde tema.json yok"))?;
        if f.size() > JSON_SINIRI {
            return Err(bicim_hatasi("tema.json çok büyük"));
        }
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        s
    };

    // Önce jetonları doğrula: geçersiz bir tema için diske hiçbir şey
    // yazmıyoruz.
    let on_ozet = dogrula(&json, None)?;

    let hedef = temalar_dizini.join(&on_ozet.ad);
    std::fs::create_dir_all(&hedef)?;
    std::fs::write(hedef.join("tema.json"), &json)?;

    // Varlıkları kopyala. **Yalnız güvenli adlar ve bilinen uzantılar**;
    // arşivdeki her şeyi açmak, `..\..\Startup\kotu.exe` yazmak demek olurdu.
    for i in 0..arsiv.len() {
        let mut f = match arsiv.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        // `enclosed_name` zaten taşma yapan adları eliyor; kendi
        // kontrolümüz de üstüne biniyor (iki süzgeç yerine iki kilit).
        let Some(ad) = f.enclosed_name().and_then(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        }) else {
            continue;
        };
        if ad == "tema.json" || !yol_guvenli(&ad) {
            continue;
        }
        let uzanti = Path::new(&ad)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();
        if !GORSEL_UZANTILARI.contains(&uzanti.as_str()) {
            continue;
        }
        if f.size() > VARLIK_SINIRI {
            return Err(bicim_hatasi(format!("{ad} çok büyük")));
        }
        let mut veri = Vec::new();
        f.read_to_end(&mut veri)?;
        std::fs::write(hedef.join(&ad), veri)?;
    }

    // Şimdi arka plan yolunu da kurabiliriz.
    dogrula(&json, Some(&hedef))
}

/// Kurulu temaları okur. Bozuk bir tema klasörü **atlanıyor**: bir bozuk
/// dosya yüzünden tema listesinin hiç açılmaması kabul edilemez.
pub fn kurulular(temalar_dizini: &Path) -> Vec<TemaOzeti> {
    let mut liste = Vec::new();
    let Ok(girdiler) = std::fs::read_dir(temalar_dizini) else {
        return liste;
    };
    for g in girdiler.flatten() {
        let yol = g.path();
        if !yol.is_dir() {
            continue;
        }
        let Ok(json) = std::fs::read_to_string(yol.join("tema.json")) else {
            continue;
        };
        match dogrula(&json, Some(&yol)) {
            Ok(t) => liste.push(t),
            Err(e) => log::warn!("muiren: tema atlandı ({}): {e}", yol.display()),
        }
    }
    liste
}

/// Kurulu bir temayı siler. Yerleşik temalar silinemiyor.
pub fn sil(ad: &str, temalar_dizini: &Path) -> Sonuc<()> {
    if !ad_guvenli(ad) {
        return Err(bicim_hatasi("geçersiz tema adı"));
    }
    if super::yerlesikler().iter().any(|(a, _)| *a == ad) {
        return Err(bicim_hatasi("yerleşik tema silinemiyor"));
    }
    let yol = temalar_dizini.join(ad);
    if yol.is_dir() {
        std::fs::remove_dir_all(yol)?;
    }
    Ok(())
}

/// Kurulu bir temayı `.muitema` olarak dışa aktarır.
///
/// Uygulama içi tema mağazası **yok** (`docs/Temalar.md`): sunucu yok, hesap
/// yok, moderasyon yükü yok. Dosya paylaşımı kullanıcının kendi işi.
pub fn disa_aktar(ad: &str, temalar_dizini: &Path, hedef: &Path) -> Sonuc<()> {
    if !ad_guvenli(ad) {
        return Err(bicim_hatasi("geçersiz tema adı"));
    }
    let kaynak = temalar_dizini.join(ad);
    let cikti = std::fs::File::create(hedef)?;
    let mut zip = zip::ZipWriter::new(cikti);
    let secenek: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for g in std::fs::read_dir(&kaynak)?.flatten() {
        let yol = g.path();
        if !yol.is_file() {
            continue;
        }
        let Some(dosya_adi) = yol.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        zip.start_file(dosya_adi, secenek).map_err(bicim_hatasi)?;
        let veri = std::fs::read(&yol)?;
        use std::io::Write;
        zip.write_all(&veri)?;
    }
    zip.finish().map_err(bicim_hatasi)?;
    Ok(())
}

/// Temalar dizini. `settings` içindeki tema adı buradaki bir klasöre denk
/// geliyor.
pub fn dizin(veri_dizini: &Path) -> PathBuf {
    veri_dizini.join("temalar")
}

#[cfg(test)]
mod testler {
    use super::*;

    fn json(govde: &str) -> String {
        format!(r##"{{"bicim":1,"ad":"Deneme",{govde}}}"##)
    }

    // ------------------------------------------------------- yol güvenliği
    //
    // Bu blok dizin taşmasının kapandığı yer; her satır gerçek bir saldırı.

    #[test]
    fn dizin_tasmasi_reddediliyor() {
        for kotu in [
            "../kotu.png",
            "..\\kotu.png",
            "/etc/passwd",
            "C:\\Windows\\System32\\kotu.dll",
            "alt/dizin.png",
            "alt\\dizin.png",
            "http://saldirgan.net/x.png",
            "file:///C:/x.png",
            ".gizli",
            "",
            "   ",
        ] {
            assert!(!yol_guvenli(kotu), "{kotu:?} geçmemeliydi");
        }
    }

    #[test]
    fn duz_dosya_adlari_gecerli() {
        for iyi in ["arkaplan.jpg", "onizleme.png", "arka plan.webp"] {
            assert!(yol_guvenli(iyi), "{iyi}");
        }
    }

    #[test]
    fn tema_adi_klasor_adi_olarak_guvenli() {
        assert!(ad_guvenli("Gece Sakura"));
        assert!(ad_guvenli("Kağıt"));
        for kotu in ["../kotu", "a/b", "a\\b", "C:x", ".gizli", "a<b", "a|b"] {
            assert!(!ad_guvenli(kotu), "{kotu}");
        }
    }

    // ------------------------------------------------------------ doğrulama

    #[test]
    fn gecerli_tema_dogrulaniyor() {
        let t = dogrula(
            &json(r##""jetonlar":{"bg":"#101014","text":"#ffffff"}"##),
            None,
        )
        .unwrap();
        assert_eq!(t.ad, "Deneme");
        assert_eq!(t.jetonlar.len(), 2);
        assert!(!t.yerlesik);
    }

    #[test]
    fn bilinmeyen_bicim_reddediliyor() {
        let g = r##"{"bicim":99,"ad":"X","jetonlar":{"bg":"#101014"}}"##;
        assert!(dogrula(g, None).is_err());
    }

    #[test]
    fn bos_ad_reddediliyor() {
        let g = r##"{"bicim":1,"ad":"  ","jetonlar":{"bg":"#101014"}}"##;
        assert!(dogrula(g, None).is_err());
    }

    #[test]
    fn hicbir_gecerli_jeton_yoksa_reddediliyor() {
        // Tamamı atılan bir tema uygulanmış görünüp hiçbir şey değiştirmezdi.
        let g = json(r##""jetonlar":{"bg":"url(https://x/)","bilinmeyen":"1"}"##);
        assert!(dogrula(&g, None).is_err());
    }

    #[test]
    fn kotu_jeton_atiliyor_tema_yasiyor() {
        let t = dogrula(
            &json(r##""jetonlar":{"bg":"#101014","text":"url(https://x/)"}"##),
            None,
        )
        .unwrap();
        assert_eq!(t.jetonlar.len(), 1);
        assert_eq!(t.atilan_jetonlar, vec!["text"]);
    }

    #[test]
    fn kotu_kontrast_uyari_uretiyor_ama_reddetmiyor() {
        // Tema reddedilmiyor; kullanıcı görüyor ve geri alabiliyor.
        let t = dogrula(
            &json(r##""jetonlar":{"bg":"#101014","text":"#151519"}"##),
            None,
        )
        .unwrap();
        assert_eq!(t.jetonlar.len(), 2);
        assert!(!t.uyarilar.is_empty());
        assert!(t.uyarilar[0].contains("okunmuyor"));
    }

    #[test]
    fn iyi_kontrast_uyari_uretmiyor() {
        let t = dogrula(
            &json(r##""jetonlar":{"bg":"#0f1115","text":"#e8eaed"}"##),
            None,
        )
        .unwrap();
        assert!(t.uyarilar.is_empty());
    }

    #[test]
    fn on_accent_uyarisi_ciziliyor() {
        // `docs/Temalar.md` tuzağı.
        let t = dogrula(
            &json(r##""jetonlar":{"accent":"#2dd4bf","on-accent":"#ffffff"}"##),
            None,
        )
        .unwrap();
        assert!(t.uyarilar.iter().any(|u| u.contains("Birincil düğme")));
    }

    // --------------------------------------------------------- arka plan

    #[test]
    fn arkaplan_yolu_dogrulaniyor() {
        let kok = std::path::Path::new("C:\\temalar\\Deneme");
        let iyi = ArkaplanDosyasi {
            dosya: "arkaplan.jpg".into(),
            yerlesim: "kapla".into(),
            karartma: 0.5,
            bulanik: 0,
        };
        assert!(arkaplan_dogrula(&iyi, kok).is_ok());

        let kotu = ArkaplanDosyasi {
            dosya: "../../gizli.jpg".into(),
            ..iyi.clone()
        };
        assert!(arkaplan_dogrula(&kotu, kok).is_err());
    }

    #[test]
    fn uzak_arkaplan_reddediliyor() {
        // Her yeni sekmede tema yazarının sunucusuna sinyal gitmemeli.
        let kok = std::path::Path::new("C:\\t");
        for url in [
            "https://saldirgan.net/x.jpg",
            "http://x/y.png",
            "file:///C:/x.png",
        ] {
            let a = ArkaplanDosyasi {
                dosya: url.into(),
                yerlesim: "kapla".into(),
                karartma: 0.5,
                bulanik: 0,
            };
            assert!(arkaplan_dogrula(&a, kok).is_err(), "{url}");
        }
    }

    #[test]
    fn desteklenmeyen_gorsel_bicimi_reddediliyor() {
        let kok = std::path::Path::new("C:\\t");
        for ad in ["kotu.svg", "kotu.exe", "kotu.html", "kotu"] {
            let a = ArkaplanDosyasi {
                dosya: ad.into(),
                yerlesim: "kapla".into(),
                karartma: 0.5,
                bulanik: 0,
            };
            assert!(arkaplan_dogrula(&a, kok).is_err(), "{ad}");
        }
    }

    #[test]
    fn karartma_ve_bulanik_sinirlaniyor() {
        let kok = std::path::Path::new("C:\\t");
        let a = ArkaplanDosyasi {
            dosya: "x.jpg".into(),
            yerlesim: "bilinmeyen".into(),
            karartma: 5.0,
            bulanik: 9999,
        };
        let s = arkaplan_dogrula(&a, kok).unwrap();
        // Tam karartma arka planı görünmez yapardı.
        assert_eq!(s.karartma, 0.9);
        assert_eq!(s.bulanik, 40);
        // Tanınmayan yerleşim varsayılana düşüyor, tema reddedilmiyor.
        assert_eq!(s.yerlesim, "kapla");
    }

    #[test]
    fn negatif_karartma_sifira_cekiliyor() {
        let kok = std::path::Path::new("C:\\t");
        let a = ArkaplanDosyasi {
            dosya: "x.jpg".into(),
            yerlesim: "ortala".into(),
            karartma: -1.0,
            bulanik: 0,
        };
        assert_eq!(arkaplan_dogrula(&a, kok).unwrap().karartma, 0.0);
    }

    // ------------------------------------------------------------ yerleşik

    #[test]
    fn yerlesik_ozet_dogrulanmis_jeton_veriyor() {
        let (ad, jetonlar) = super::super::yerlesikler()[0];
        let o = yerlesik_ozet(ad, jetonlar);
        assert!(o.yerlesik);
        assert!(o.atilan_jetonlar.is_empty());
        assert!(o.uyarilar.is_empty());
        // Çıktı normalleşmiş: her renk `#rrggbbaa`.
        assert!(o
            .jetonlar
            .iter()
            .filter(|(a, _)| a != "--radius" && a != "--radius-lg")
            .all(|(_, d)| d.len() == 9 && d.starts_with('#')));
    }

    #[test]
    fn yerlesik_tema_silinemiyor() {
        let d = std::path::Path::new("C:\\t");
        assert!(sil("Mui", d).is_err());
    }

    #[test]
    fn gecersiz_adla_silme_reddediliyor() {
        let d = std::path::Path::new("C:\\t");
        assert!(sil("../../Windows", d).is_err());
    }

    // ------------------------------------------------------- gerçek arşiv

    #[test]
    fn arsivden_kurulum_calisiyor() {
        let gecici = tempfile::tempdir().unwrap();
        let arsiv_yolu = gecici.path().join("deneme.muitema");
        {
            let f = std::fs::File::create(&arsiv_yolu).unwrap();
            let mut z = zip::ZipWriter::new(f);
            let s: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();
            use std::io::Write;
            z.start_file("tema.json", s).unwrap();
            z.write_all(
                br##"{"bicim":1,"ad":"Deneme","yazar":"ben","jetonlar":{"bg":"#101014","text":"#ffffff"}}"##,
            )
            .unwrap();
            // Görsel olmayan bir dosya: kopyalanmamalı.
            z.start_file("kotu.exe", s).unwrap();
            z.write_all(b"MZ").unwrap();
            z.finish().unwrap();
        }

        let temalar = gecici.path().join("temalar");
        let o = kur(&arsiv_yolu, &temalar).unwrap();
        assert_eq!(o.ad, "Deneme");
        assert_eq!(o.yazar, "ben");
        assert!(temalar.join("Deneme").join("tema.json").exists());
        // Görsel olmayan dosya kopyalanmadı.
        assert!(!temalar.join("Deneme").join("kotu.exe").exists());

        // Kurulu liste onu görüyor.
        let liste = kurulular(&temalar);
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].ad, "Deneme");
    }

    #[test]
    fn temasiz_arsiv_reddediliyor() {
        let gecici = tempfile::tempdir().unwrap();
        let arsiv_yolu = gecici.path().join("bos.muitema");
        {
            let f = std::fs::File::create(&arsiv_yolu).unwrap();
            let mut z = zip::ZipWriter::new(f);
            let s: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();
            use std::io::Write;
            z.start_file("okuma.txt", s).unwrap();
            z.write_all(b"merhaba").unwrap();
            z.finish().unwrap();
        }
        assert!(kur(&arsiv_yolu, gecici.path()).is_err());
    }

    #[test]
    fn bozuk_tema_klasoru_listeyi_dusurmuyor() {
        // Bir bozuk dosya yüzünden tema listesinin hiç açılmaması kabul
        // edilemez.
        let gecici = tempfile::tempdir().unwrap();
        let temalar = gecici.path();
        std::fs::create_dir_all(temalar.join("Bozuk")).unwrap();
        std::fs::write(temalar.join("Bozuk").join("tema.json"), "{ bozuk").unwrap();
        std::fs::create_dir_all(temalar.join("Iyi")).unwrap();
        std::fs::write(
            temalar.join("Iyi").join("tema.json"),
            r##"{"bicim":1,"ad":"Iyi","jetonlar":{"bg":"#101014"}}"##,
        )
        .unwrap();

        let liste = kurulular(temalar);
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].ad, "Iyi");
    }

    #[test]
    fn disa_aktarim_gidis_donus() {
        let gecici = tempfile::tempdir().unwrap();
        let temalar = gecici.path().join("temalar");
        std::fs::create_dir_all(temalar.join("Deneme")).unwrap();
        std::fs::write(
            temalar.join("Deneme").join("tema.json"),
            r##"{"bicim":1,"ad":"Deneme","jetonlar":{"bg":"#101014"}}"##,
        )
        .unwrap();

        let cikti = gecici.path().join("cikti.muitema");
        disa_aktar("Deneme", &temalar, &cikti).unwrap();

        let geri = gecici.path().join("geri");
        let o = kur(&cikti, &geri).unwrap();
        assert_eq!(o.ad, "Deneme");
    }
}
