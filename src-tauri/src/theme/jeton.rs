//! Jeton doğrulama — **temanın güvenlik çekirdeği**.
//!
//! `docs/Roadmap.md` karar #3: *tema veri, kod değil.* Sebep somut — bir
//! tarayıcının arayüzünde çalışan kod, o tarayıcının açtığı her sayfayı
//! görebilir. "Şu forumdan indirdiğim güzel tema" bankacılık sekmesini
//! okuyabilir hâle gelirdi.
//!
//! Rastgele CSS bile yetiyor: `url()` ile uzak sunucuya sinyal göndermek,
//! `var()` ile başka bir jetonu kendine çekmek, `expression()` ile eski
//! motorlarda kod çalıştırmak. **CSS enjeksiyonunun tamamı bu dosyada
//! kapanıyor** ve buradan geçmeyen hiçbir değer `documentElement.style`
//! üzerine yazılmıyor.
//!
//! Yaklaşım kara liste değil **beyaz liste**: "şunlar yasak" demek yerine
//! "yalnız şu biçim serbest" deniyor. Kara liste her yeni CSS özelliğinde
//! güncellenmesi gereken bir borç; beyaz liste değil.

use serde::{Deserialize, Serialize};

/// Temanın değiştirebileceği jetonlar (`docs/Temalar.md` tablosu).
///
/// Listede olmayan anahtar **hata değil, sessizce yok sayılıyor**: ileri
/// sürüm temaları eski Muiren'de de açılabilsin diye. Ama uygulanmıyor.
///
/// Burada **olmayanlar** da bilinçli: Mui logosu ve ikonu, hakkında ekranı,
/// izin/uyarı diyaloglarının rengi, adres çubuğundaki güvenlik göstergesi.
/// Uyarı diyaloğunu yeşile boyayabilen tema, uyarıyı yok edebilir demektir.
pub const RENK_JETONLARI: &[&str] = &[
    "bg",
    "bg-panel",
    "bg-elevated",
    "bg-sunken",
    "border",
    "border-strong",
    "text",
    "text-muted",
    "accent",
    "accent-strong",
    "on-accent",
];

/// Ölçü jetonları. Aralık dar: 32 pikselden büyük bir köşe yarıçapı düğmeyi
/// daireye çeviriyor ve arayüzü kullanılamaz hâle getiriyor.
pub const OLCU_JETONLARI: &[&str] = &["radius", "radius-lg"];

/// Ölçü jetonlarının üst sınırı, piksel.
pub const OLCU_UST_SINIR: u32 = 32;

/// Doğrulanmış renk. Yapıcısı yok: yalnız [`renk_dogrula`] üretebiliyor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Renk {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Renk {
    /// CSS'e yazılacak biçim. **Her zaman `#rrggbbaa`** — girdi ne biçimde
    /// gelirse gelsin çıktı normalleştiriliyor, böylece arayüze giden dizenin
    /// biçimi tek ve öngörülebilir oluyor.
    pub fn css(&self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }

    /// WCAG bağıl parlaklık. Kontrast hesabının girdisi.
    fn parlaklik(&self) -> f64 {
        fn kanal(v: u8) -> f64 {
            let s = v as f64 / 255.0;
            if s <= 0.039_28 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * kanal(self.r) + 0.7152 * kanal(self.g) + 0.0722 * kanal(self.b)
    }
}

/// İki rengin WCAG kontrast oranı (1.0–21.0).
///
/// Saydamlık **yok sayılıyor**: alfa kanalı hesaba katılsaydı arkasındaki
/// rengi de bilmek gerekirdi ve tema doğrulama sırasında o bilgi yok. Yarı
/// saydam bir metin rengi zaten kötü fikir.
pub fn kontrast(a: Renk, b: Renk) -> f64 {
    let (x, y) = (a.parlaklik(), b.parlaklik());
    let (parlak, koyu) = if x > y { (x, y) } else { (y, x) };
    (parlak + 0.05) / (koyu + 0.05)
}

/// WCAG AA eşiği. Altında kalan tema **reddedilmiyor**, uyarı çıkıyor
/// (`docs/Temalar.md`).
pub const KONTRAST_ESIGI: f64 = 4.5;

/// Renk doğrulama — `#rgb`, `#rrggbb`, `#rrggbbaa`. **Başka hiçbir şey.**
///
/// Adlandırılmış renkler (`red`), `rgb()`, `hsl()`, `color-mix()` ve
/// hepsinden önemlisi `url(`, `var(`, `expression(`, `image-set(` yok.
/// Sebep: bunların hepsi ya uzak sunucuya sinyal ya da başka bir değeri
/// kendine çekme yolu.
///
/// Ayrıştırma `#` sonrasındaki karakterlerin **hepsinin** onaltılık olmasını
/// istiyor; boşluk, ters bölü ve yorum dizisi doğal olarak eleniyor.
pub fn renk_dogrula(ham: &str) -> Option<Renk> {
    let s = ham.trim();
    let g = s.strip_prefix('#')?;
    if !g.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    let onalti = |i: usize, n: usize| u8::from_str_radix(&g[i..i + n], 16).ok();

    match g.len() {
        // `#rgb` → her basamak iki katına çıkıyor (`#abc` = `#aabbcc`).
        3 => {
            let d: Vec<u8> = g
                .chars()
                .map(|c| c.to_digit(16).unwrap() as u8)
                .map(|v| v * 17)
                .collect();
            Some(Renk {
                r: d[0],
                g: d[1],
                b: d[2],
                a: 255,
            })
        }
        6 => Some(Renk {
            r: onalti(0, 2)?,
            g: onalti(2, 2)?,
            b: onalti(4, 2)?,
            a: 255,
        }),
        8 => Some(Renk {
            r: onalti(0, 2)?,
            g: onalti(2, 2)?,
            b: onalti(4, 2)?,
            a: onalti(6, 2)?,
        }),
        _ => None,
    }
}

/// Doğrulanmış ölçü. Yalnız `0..=32` tam sayı + `px`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Olcu(pub u32);

impl Olcu {
    pub fn css(&self) -> String {
        format!("{}px", self.0)
    }
}

/// Ölçü doğrulama — `"12px"`. Birimsiz sayı, `em`, `rem`, `calc()` yok.
///
/// Birim zorunlu çünkü birimsiz bir değer CSS'te `0` dışında geçersiz ve
/// sessizce yok sayılıyor: tema uygulanmış görünüp uygulanmamış olurdu.
pub fn olcu_dogrula(ham: &str) -> Option<Olcu> {
    let s = ham.trim();
    let sayi = s.strip_suffix("px")?;
    if sayi.is_empty() || !sayi.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: u32 = sayi.parse().ok()?;
    (n <= OLCU_UST_SINIR).then_some(Olcu(n))
}

/// Ham jeton haritasını doğrulanmış hâline çevirir.
///
/// Dönüş: CSS'e yazılabilir `(--jeton, deger)` çiftleri ve **atılan**
/// anahtarlar. Atılanlar kullanıcıya gösteriliyor: sessizce yok sayılan bir
/// jeton, tema yazarının "neden çalışmıyor" diye saatlerce bakması demek.
pub fn dogrula(
    ham: &std::collections::BTreeMap<String, String>,
) -> (Vec<(String, String)>, Vec<String>) {
    let mut kabul = Vec::new();
    let mut atilan = Vec::new();

    for (anahtar, deger) in ham {
        let ad = anahtar.trim();
        if RENK_JETONLARI.contains(&ad) {
            match renk_dogrula(deger) {
                Some(r) => kabul.push((format!("--{ad}"), r.css())),
                None => atilan.push(ad.to_string()),
            }
        } else if OLCU_JETONLARI.contains(&ad) {
            match olcu_dogrula(deger) {
                Some(o) => kabul.push((format!("--{ad}"), o.css())),
                None => atilan.push(ad.to_string()),
            }
        } else {
            // Beyaz listede yok: sessizce atılıyor ama rapor ediliyor.
            atilan.push(ad.to_string());
        }
    }

    (kabul, atilan)
}

#[cfg(test)]
mod testler {
    use super::*;

    // ------------------------------------------------------- renk: kabul

    #[test]
    fn gecerli_renkler_kabul_ediliyor() {
        assert_eq!(
            renk_dogrula("#2dd4bf"),
            Some(Renk {
                r: 0x2d,
                g: 0xd4,
                b: 0xbf,
                a: 255
            })
        );
        assert_eq!(
            renk_dogrula("#2dd4bf80"),
            Some(Renk {
                r: 0x2d,
                g: 0xd4,
                b: 0xbf,
                a: 0x80
            })
        );
        // `#abc` = `#aabbcc`
        assert_eq!(renk_dogrula("#abc").unwrap().css(), "#aabbccff");
        // Baş/son boşluk kırpılıyor.
        assert_eq!(renk_dogrula("  #0f1115  ").unwrap().css(), "#0f1115ff");
        // Büyük harf onaltılık.
        assert_eq!(renk_dogrula("#0F1115").unwrap().css(), "#0f1115ff");
    }

    #[test]
    fn cikti_her_zaman_normallesiyor() {
        // Girdi biçimi ne olursa olsun arayüze giden dize tek biçimde.
        for g in ["#abc", "#aabbcc", "#aabbccff"] {
            assert_eq!(renk_dogrula(g).unwrap().css(), "#aabbccff");
        }
    }

    // ------------------------------------------------------- renk: RET
    //
    // Bu testler dosyanın varlık sebebi. Her biri gerçek bir saldırı yolu.

    #[test]
    fn url_reddediliyor() {
        // Uzak sunucuya sinyal: her yeni sekmede tema yazarına ping.
        assert_eq!(renk_dogrula("url(https://saldirgan.net/p.png)"), None);
        assert_eq!(renk_dogrula("#fff; background: url(https://x/)"), None);
    }

    #[test]
    fn var_ve_expression_reddediliyor() {
        assert_eq!(renk_dogrula("var(--accent)"), None);
        assert_eq!(renk_dogrula("expression(alert(1))"), None);
        assert_eq!(renk_dogrula("image-set('x.png')"), None);
    }

    #[test]
    fn fonksiyon_bicimleri_reddediliyor() {
        // Kendi başlarına zararsız ama beyaz liste dar tutuluyor: her yeni
        // biçim, ayrıştırıcıda yeni bir kaçış yüzeyi.
        for g in [
            "rgb(45,212,191)",
            "rgba(45,212,191,0.5)",
            "hsl(174 70% 50%)",
            "color-mix(in srgb, red, blue)",
        ] {
            assert_eq!(renk_dogrula(g), None, "{g}");
        }
    }

    #[test]
    fn adlandirilmis_renkler_reddediliyor() {
        for g in ["red", "transparent", "currentColor", "inherit"] {
            assert_eq!(renk_dogrula(g), None, "{g}");
        }
    }

    #[test]
    fn kacis_ve_yorum_dizileri_reddediliyor() {
        // CSS'te `\` kaçış başlatıyor, `/*` yorum, `;` bildirim bitiriyor.
        // Üçü de ayrıştırıcı kandırmanın klasik yolu. İçerideki boşluk da
        // reddediliyor: `#ff ff ff` iki ayrı jetona bölünebilirdi.
        for g in [r"#fff\0a", "#fff/*", "#ff ff ff", "#fff;", "#fff}"] {
            assert_eq!(renk_dogrula(g), None, "{g:?}");
        }
    }

    #[test]
    fn bastaki_sondaki_bosluk_zararsiz() {
        // `trim` yalnız uçtaki boşluğu atıyor ve **çıktı yeniden üretiliyor**:
        // ne gelirse gelsin CSS'e giden dize `#rrggbbaa`. Yani buradan bir
        // kaçış mümkün değil; satır sonu taşıyan bir tema dosyası da
        // çalışıyor.
        assert_eq!(renk_dogrula("#fff\n").unwrap().css(), "#ffffffff");
        assert_eq!(renk_dogrula("\t#fff  ").unwrap().css(), "#ffffffff");
    }

    #[test]
    fn bozuk_uzunluklar_reddediliyor() {
        for g in [
            "#",
            "#f",
            "#ff",
            "#ffff",
            "#fffff",
            "#fffffff",
            "#fffffffff",
        ] {
            assert_eq!(renk_dogrula(g), None, "{g}");
        }
        // `#` olmadan geçmiyor.
        assert_eq!(renk_dogrula("2dd4bf"), None);
    }

    #[test]
    fn onaltilik_olmayan_basamak_reddediliyor() {
        assert_eq!(renk_dogrula("#gggggg"), None);
        assert_eq!(renk_dogrula("#12345z"), None);
    }

    // ------------------------------------------------------------- ölçü

    #[test]
    fn gecerli_olculer() {
        assert_eq!(olcu_dogrula("12px"), Some(Olcu(12)));
        assert_eq!(olcu_dogrula("0px"), Some(Olcu(0)));
        assert_eq!(olcu_dogrula(" 32px "), Some(Olcu(32)));
    }

    #[test]
    fn sinirin_ustu_reddediliyor() {
        // 32 pikselden büyük yarıçap düğmeyi daireye çeviriyor.
        assert_eq!(olcu_dogrula("33px"), None);
        assert_eq!(olcu_dogrula("9999px"), None);
    }

    #[test]
    fn birim_zorunlu_ve_yalniz_px() {
        // Birimsiz değer CSS'te sessizce yok sayılıyor: tema uygulanmış
        // görünüp uygulanmamış olurdu.
        assert_eq!(olcu_dogrula("12"), None);
        assert_eq!(olcu_dogrula("1rem"), None);
        assert_eq!(olcu_dogrula("2em"), None);
        assert_eq!(olcu_dogrula("calc(12px + 1px)"), None);
        assert_eq!(olcu_dogrula("-4px"), None);
        assert_eq!(olcu_dogrula("1.5px"), None);
        assert_eq!(olcu_dogrula("px"), None);
    }

    // ---------------------------------------------------------- kontrast

    #[test]
    fn siyah_beyaz_en_yuksek_kontrast() {
        let siyah = renk_dogrula("#000000").unwrap();
        let beyaz = renk_dogrula("#ffffff").unwrap();
        assert!((kontrast(siyah, beyaz) - 21.0).abs() < 0.01);
        // Sıra önemsiz.
        assert!((kontrast(beyaz, siyah) - 21.0).abs() < 0.01);
    }

    #[test]
    fn ayni_renk_en_dusuk_kontrast() {
        let r = renk_dogrula("#2dd4bf").unwrap();
        assert!((kontrast(r, r) - 1.0).abs() < 0.001);
    }

    #[test]
    fn mui_varsayilani_esigi_geciyor() {
        // Kendi temamız kendi kuralımızdan geçmezse kural yanlış demektir.
        let bg = renk_dogrula("#0f1115").unwrap();
        let text = renk_dogrula("#e8eaed").unwrap();
        assert!(kontrast(bg, text) >= KONTRAST_ESIGI);
    }

    #[test]
    fn on_accent_tuzagi_yakalaniyor() {
        // `docs/Temalar.md` tuzağı: teal AÇIK bir renk, üstündeki yazı koyu
        // olmak zorunda. Tema yazarı `accent`i koyulaştırıp `on-accent`i
        // güncellemezse birincil düğmeler okunmaz hâle geliyor.
        let accent = renk_dogrula("#2dd4bf").unwrap();
        let dogru = renk_dogrula("#04211d").unwrap(); // koyu
        let yanlis = renk_dogrula("#ffffff").unwrap(); // açık üstüne açık
        assert!(kontrast(accent, dogru) >= KONTRAST_ESIGI);
        assert!(kontrast(accent, yanlis) < KONTRAST_ESIGI);
    }

    // ---------------------------------------------------------- dogrula()

    fn harita(cift: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
        cift.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn beyaz_listedeki_jetonlar_kabul_ediliyor() {
        let (kabul, atilan) = dogrula(&harita(&[("bg", "#101010"), ("radius", "8px")]));
        assert_eq!(
            kabul,
            vec![
                ("--bg".into(), "#101010ff".into()),
                ("--radius".into(), "8px".into())
            ]
        );
        assert!(atilan.is_empty());
    }

    #[test]
    fn listede_olmayan_jeton_sessizce_atiliyor() {
        // İleri sürüm temaları eski Muiren'de de açılsın diye hata değil.
        let (kabul, atilan) = dogrula(&harita(&[("gelecek-jeton", "#fff")]));
        assert!(kabul.is_empty());
        assert_eq!(atilan, vec!["gelecek-jeton"]);
    }

    #[test]
    fn degistirilemeyen_jetonlar_listede_yok() {
        // Uyarı diyaloğunu yeşile boyayabilen tema, uyarıyı yok edebilir
        // demektir (`docs/Temalar.md`).
        for yasak in ["error", "warning", "ok", "shadow-1", "font-ui"] {
            assert!(!RENK_JETONLARI.contains(&yasak), "{yasak}");
            assert!(!OLCU_JETONLARI.contains(&yasak), "{yasak}");
        }
    }

    #[test]
    fn gecerli_anahtar_gecersiz_deger_atiliyor() {
        let (kabul, atilan) = dogrula(&harita(&[("bg", "url(https://x/)")]));
        assert!(kabul.is_empty());
        assert_eq!(atilan, vec!["bg"]);
    }
}
