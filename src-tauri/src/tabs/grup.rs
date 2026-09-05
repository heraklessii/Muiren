//! Sekme grupları — **saf**.
//!
//! `docs/Roadmap.md` Faz 5: "Sekme grupları, katlama, grup bazlı uyku eşiği."
//! Üçü aynı veri yapısının üç yüzü ve üçüncüsü projenin tezine bağlanıyor:
//!
//! > Katlanmış bir grup, kullanıcının "buraya şimdilik bakmıyorum" demesi.
//! > Bunu bilen bir tarayıcının o sekmeleri normal eşikte beklemesi için hiçbir
//! > sebep yok.
//!
//! ## Renk neden serbest bir dize değil
//!
//! Grup rengi arayüzde bir CSS değerine dönüşüyor. Kullanıcının yazdığı bir
//! dizeyi oraya bırakmak, `theme/jeton.rs` içinde uzun uzun kapatılan kapıyı
//! yan taraftan açmak olurdu. Bu yüzden renk bir **enum**: sayılı, kapalı ve
//! arayüzdeki jetonlara eşleniyor (CLAUDE.md #9 ile aynı gerekçe — ölçü ve
//! renk sabitleri `src/styles.css` içinde).
//!
//! ## Eşik neden `Option`
//!
//! `None` "genel ayarı kullan" demek, `Some(0)` değil. Grubun eşiğini
//! ayarların eşiğine eşitleyerek kopyalasaydık, kullanıcı genel eşiği
//! değiştirdiğinde grupları eski değerde kalırdı ve sebebini bulamazdı.

use serde::{Deserialize, Serialize};

use super::GrupId;

/// Grup rengi. Arayüzdeki jetonlara eşleniyor (`src/styles.css`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GrupRengi {
    /// Aile vurgusu. Varsayılan.
    Teal,
    Mor,
    Kehribar,
    Kirmizi,
    Yesil,
    Mavi,
    Gri,
}

impl Default for GrupRengi {
    fn default() -> Self {
        GrupRengi::Teal
    }
}

/// Grup adının üst sınırı. Ad kullanıcıdan geliyor ve sekme çubuğunda bir
/// etiket olarak çiziliyor; sınırsız uzunlukta bir ad şeridi yiyor.
const AD_SINIRI: usize = 40;

/// Grubun uyku eşiği için kabul edilen aralık. Alt sınır `settings::duzelt`
/// ile aynı (30 sn): daha kısası, kullanıcının sekme değiştirirken
/// öncekini kaybetmesi demek.
const ESIK_ALT: u64 = 30;
const ESIK_UST: u64 = 86_400;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grup {
    pub id: GrupId,
    pub ad: String,
    pub renk: GrupRengi,
    /// Katlanmış grubun sekmeleri sekme çubuğunda gizli.
    pub katli: bool,
    /// Bu gruba özel uyku eşiği. `None` ise genel ayar geçerli.
    ///
    /// Katlama ile **birleştirilmiyor**: "katlanınca hemen uyusun" varsayılan
    /// bir davranış olarak dayatılmıyor, ayrı bir tercih olarak duruyor.
    /// Kullanıcı grubu düzen için katlıyor olabilir.
    pub uyku_esigi_sn: Option<u64>,
}

impl Grup {
    pub fn yeni(id: GrupId, ad: &str) -> Self {
        Grup {
            id,
            ad: ad_temizle(ad),
            renk: GrupRengi::default(),
            katli: false,
            uyku_esigi_sn: None,
        }
    }
}

/// Grup adını arayüze basılabilir hâle getirir.
///
/// Ad kullanıcıdan geliyor, sayfadan değil — yani `tabs::baslik_temizle`
/// kadar düşmanca bir girdi değil. Yine de aynı iki iş yapılıyor: kontrol
/// karakterleri (kopyala-yapıştır ile gelen satır sonları) düşüyor ve ad
/// kırpılıyor. Boş ad **boş kalıyor**; arayüz onu "Adsız grup" diye çiziyor
/// ve o metin arayüzde duruyor, burada değil (`docs/Frontend.md`).
pub fn ad_temizle(ham: &str) -> String {
    let temiz: String = ham
        .chars()
        .filter(|c| !c.is_control())
        .filter(|c| {
            !matches!(*c,
                '\u{200E}' | '\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
                | '\u{200B}'..='\u{200D}'
                | '\u{FEFF}')
        })
        .collect();
    let kirpilmis: String = temiz.trim().chars().take(AD_SINIRI).collect();
    kirpilmis.trim_end().to_string()
}

/// Grup eşiğini geçerli aralığa çeker.
///
/// `settings::duzelt` ile aynı kalıp ve aynı gerekçe (CLAUDE.md #13):
/// düzeltilmiş değer kullanıcıya geri gösteriliyor, sessizce farklı bir
/// değerle çalışmak yok.
pub fn esik_duzelt(ham: Option<u64>) -> Option<u64> {
    ham.map(|e| e.clamp(ESIK_ALT, ESIK_UST))
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn ad_kirpiliyor() {
        assert_eq!(ad_temizle(&"a".repeat(100)).chars().count(), AD_SINIRI);
    }

    #[test]
    fn ad_satir_sonlarini_atiyor() {
        assert_eq!(ad_temizle("İş\nSekmeleri"), "İşSekmeleri");
    }

    #[test]
    fn ad_ters_akis_karakterini_atiyor() {
        assert!(!ad_temizle("grup\u{202E}kotu").contains('\u{202E}'));
    }

    #[test]
    fn turkce_karakterler_korunuyor() {
        assert_eq!(ad_temizle("  Alışveriş  "), "Alışveriş");
    }

    #[test]
    fn bos_ad_bos_kaliyor() {
        // "Adsız grup" metni arayüzde; backend Türkçe metin göndermiyor
        // (`docs/IPC.md`).
        assert_eq!(ad_temizle("   "), "");
    }

    #[test]
    fn esik_araliga_cekiliyor() {
        assert_eq!(esik_duzelt(Some(0)), Some(ESIK_ALT));
        assert_eq!(esik_duzelt(Some(999_999)), Some(ESIK_UST));
        assert_eq!(esik_duzelt(Some(600)), Some(600));
    }

    #[test]
    fn esiksiz_grup_genel_ayari_kullaniyor() {
        // `None` "genel ayarı kullan" demek. Buraya varsayılan bir sayı
        // yazsaydık kullanıcı genel eşiği değiştirdiğinde gruplar eski
        // değerde kalırdı.
        assert_eq!(esik_duzelt(None), None);
    }

    #[test]
    fn yeni_grup_adi_temizlenmis_geliyor() {
        let g = Grup::yeni(1, "  Ders\nNotları  ");
        assert_eq!(g.ad, "DersNotları");
        assert_eq!(g.renk, GrupRengi::Teal);
        assert!(!g.katli);
        assert_eq!(g.uyku_esigi_sn, None);
    }
}
