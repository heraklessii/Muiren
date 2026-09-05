//! Geçmiş ve yer imleri — SQLite (`docs/Depolama.md`).
//!
//! Oturum JSON'da, geçmiş burada. Ayrımın sebebi somut: oturum saniyede
//! birkaç kez **tamamı** yeniden yazılan küçük bir veri, geçmiş ise büyüyen,
//! sorgulanan ve seçmeli silinen bir veri.
//!
//! ```text
//!   gecisler.rs   şema ve sürüm yükseltme
//!   depo.rs       sorgular — geçmiş, yer imi, budama
//!   mod.rs        ortak tipler + Türkçe küçültme
//! ```

pub mod depo;
pub mod gecisler;

use serde::{Deserialize, Serialize};

/// Türkçe küçük harf.
///
/// **CLAUDE.md #10'un kaynağı.** Üç ayrı katman aynı tuzağa düşüyor:
///
/// - SQLite'ın `LOWER()` fonksiyonu ASCII dışına dokunmuyor: `"İSTANBUL"`
///   olduğu gibi kalıyor ve kullanıcı `"istanbul"` yazınca eşleşmiyor.
/// - Rust'ın `str::to_lowercase` fonksiyonu Unicode **varsayılanını**
///   uyguluyor: `'İ'` (U+0130) → `"i̇"` yani `i` + birleşen nokta. İki
///   karakter, ve `"istanbul"` ile eşleşmiyor.
/// - JavaScript'te `toLowerCase()` aynı; `toLocaleLowerCase("tr")` gerekiyor.
///
/// Bu yüzden iki harf elle ele alınıyor, gerisi Unicode'a bırakılıyor:
///
/// | Büyük | Türkçe küçük |
/// |---|---|
/// | `I` | `ı` |
/// | `İ` | `i` |
///
/// Sonuç `gecmis.baslik_kucuk` sütununa yazılıyor ve arama orada koşuyor
/// (`docs/Depolama.md`).
pub fn turkce_kucult(ham: &str) -> String {
    let mut cikti = String::with_capacity(ham.len());
    for c in ham.chars() {
        match c {
            'I' => cikti.push('ı'),
            'İ' => cikti.push('i'),
            _ => cikti.extend(c.to_lowercase()),
        }
    }
    cikti
}

/// Adresin alan adı (host). Geçmişte gruplama ve arama için ayrı sütunda.
///
/// Ayrıştırma hatası boş dize dönüyor: geçmiş kaydının yazılmaması, bir
/// kaydın alan adının boş olmasından daha kötü.
pub fn alan_adi(url: &str) -> String {
    let kalan = url.split_once("://").map(|(_, k)| k).unwrap_or(url);
    let host = kalan.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or(host);
    host.split(':').next().unwrap_or(host).to_ascii_lowercase()
}

/// Geçmiş kaydı (`docs/IPC.md`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GecmisKaydi {
    pub id: i64,
    pub url: String,
    pub baslik: String,
    /// Unix epoch, saniye.
    pub ziyaret: i64,
    /// Bu adrese kaç kez gidildi. Adres çubuğu önerilerinde sıralama ölçütü.
    pub sayac: i64,
    pub alan: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YerImi {
    pub id: i64,
    pub url: String,
    pub baslik: String,
    pub klasor: Option<i64>,
    pub sira: i64,
    pub eklendi: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YerImiKlasor {
    pub id: i64,
    pub ad: String,
    pub ebeveyn: Option<i64>,
    pub sira: i64,
}

/// `gecmis_temizle` komutunun aralığı (`docs/IPC.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Aralik {
    SonSaat,
    SonGun,
    SonHafta,
    Hepsi,
}

impl Aralik {
    /// Bu aralığın başlangıç zaman damgası (unix epoch, saniye).
    ///
    /// `simdi` parametre çünkü saf kalsın ve test edilebilsin.
    pub fn baslangic(self, simdi: i64) -> i64 {
        match self {
            Aralik::SonSaat => simdi - 3_600,
            Aralik::SonGun => simdi - 86_400,
            Aralik::SonHafta => simdi - 7 * 86_400,
            // `0` değil `i64::MIN`: sistem saati geriye alınmış bir makinede
            // negatif damgalı kayıtlar olabiliyor ve "hepsi" hepsi demek.
            Aralik::Hepsi => i64::MIN,
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn turkce_buyuk_i_dogru_kuculuyor() {
        // Rust'ın kendi `to_lowercase`i burada yanlış cevap veriyor; testin
        // asıl işi o farkı sabitlemek.
        assert_eq!(turkce_kucult("İSTANBUL"), "istanbul");
        assert_eq!(turkce_kucult("ISPARTA"), "ısparta");
        assert_ne!("İSTANBUL".to_lowercase(), "istanbul");
    }

    #[test]
    fn diger_turkce_harfler_bozulmuyor() {
        assert_eq!(turkce_kucult("ÇĞÖŞÜ"), "çğöşü");
        assert_eq!(turkce_kucult("Ağrı Dağı"), "ağrı dağı");
    }

    #[test]
    fn ascii_ve_karisik_metin() {
        assert_eq!(turkce_kucult("GitHub"), "github");
        assert_eq!(turkce_kucult("Wikipedia — İçerik"), "wikipedia — içerik");
    }

    #[test]
    fn alan_adi_cikariliyor() {
        assert_eq!(alan_adi("https://ornek.com/yol?a=1"), "ornek.com");
        assert_eq!(alan_adi("https://ali@ornek.com:8443/x"), "ornek.com");
        assert_eq!(alan_adi("http://ALT.Ornek.COM/"), "alt.ornek.com");
        assert_eq!(alan_adi("bozuk"), "bozuk");
    }

    #[test]
    fn aralik_baslangici() {
        assert_eq!(Aralik::SonSaat.baslangic(10_000), 6_400);
        assert_eq!(Aralik::SonGun.baslangic(100_000), 13_600);
        assert_eq!(Aralik::Hepsi.baslangic(10_000), i64::MIN);
    }
}
