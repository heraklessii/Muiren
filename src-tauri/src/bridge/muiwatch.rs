//! Muiwatch köprüsü — birlikte izleme.
//!
//! ## Bu köprünün bugünkü sınırı (Faz 5'te ölçüldü)
//!
//! `docs/Kopruler.md` bu köprüyü "çift yönlü değerli" diye yazıyor ve
//! haklı: Muiwatch'ın **Faz 3 — Co-Browsing** maddesi gömülü bir tarayıcı
//! gerektiriyor, Muiren tam olarak o tarayıcı. Ama Muiwatch tarafına
//! bakıldığında bugün alınacak bir kapı yok:
//!
//! - `Muiwatch/docs/ROADMAP.md` Faz 3 **başlanmamış**.
//! - Muiwatch'ta tek-örnek (single instance) ya da derin bağlantı (deep link)
//!   eklentisi **yok**; yani "şu odaya bağlan" diyecek bir argüman kapısı
//!   bulunmuyor.
//! - `nav_event` / `video_event` / `sync_heartbeat` mesajları WebRTC
//!   DataChannel üzerinde, yani Muiwatch **süreci içinde** yaşıyor. Süreçler
//!   arası bir uç yok.
//!
//! Bu yüzden Muiren'in bu fazdaki payı üç şeyle sınırlı ve üçü de yazıldı:
//!
//! 1. **Kurulum algılama** — Muiwatch yoksa arayüz menüyü hiç çizmiyor.
//! 2. **Uygulamayı açma** — oda kimliğiyle birlikte. Argüman bugün karşı
//!    tarafta okunmuyor; okunduğunda burası değişmiyor.
//! 3. **Koruma kuralı #7** — asıl iş bu. Muiwatch oturumuna bağlı sekme
//!    uyutulmuyor ve atılmıyor.
//!
//! ### Neden koruma kuralı asıl iş
//!
//! `docs/Bellek.md` koruma listesi #7 ve `docs/Kopruler.md` aynı cümleyi
//! kuruyor: uyuyan sekme `nav_event` alamaz, senkron **sessizce** kopar ve
//! kullanıcı bunu "Muiwatch bozuk" diye okur. Bu, protokolün hazır olup
//! olmamasından bağımsız bir Muiren kuralı; bugün yazılabilir ve bugün test
//! edilebilir. Kayıt [`Oturumlar`] içinde, kararı `memory/esik.rs` veriyor.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use super::{Bulunan, Kopru, KopruDurumu, Yuk};
use crate::hata::{MuirenHata, Sonuc};
use crate::tabs::SekmeId;

const URUN: &str = "Muiwatch";
const IKILI: &str = "Muiwatch.exe";

/// Oda kimliğinin üst sınırı. Kimlik kullanıcıdan ya da bir bağlantıdan
/// geliyor; sınırsız uzunlukta bir dize komut satırına gidiyor.
const ODA_SINIRI: usize = 64;

/// Oda kimliği geçerli mi.
///
/// **Saf ve testli, ve bir güvenlik sınırı.** Kimlik komut satırına gidiyor;
/// `super::calistir` kabuk kullanmıyor ama dar bir süzgeç yine de doğru:
/// harf, rakam, `-` ve `_` dışında hiçbir şey geçmiyor. Boşluk ve `-` öneki
/// ayrıca eleniyor — ikisi de argüman ayrıştırıcılarında sürpriz üretiyor.
pub fn oda_gecerli(oda: &str) -> bool {
    !oda.is_empty()
        && oda.len() <= ODA_SINIRI
        && !oda.starts_with('-')
        && oda
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Hangi sekme hangi odaya bağlı.
///
/// Sekme kaydında (`tabs::Sekme`) bir alan **değil**, ayrı bir tablo: Muiwatch
/// oturumu sekmenin kendi durumu değil, bir köprünün durumu. Sekme kapanınca
/// kayıt da düşüyor ([`Oturumlar::birak`]).
#[derive(Debug, Default)]
pub struct Oturumlar {
    kayit: Mutex<HashMap<SekmeId, String>>,
}

impl Oturumlar {
    pub fn yeni() -> Self {
        Oturumlar::default()
    }

    pub fn bagla(&self, id: SekmeId, oda: String) {
        self.kayit.lock().unwrap().insert(id, oda);
    }

    pub fn birak(&self, id: SekmeId) {
        self.kayit.lock().unwrap().remove(&id);
    }

    pub fn bagli_mi(&self, id: SekmeId) -> bool {
        self.kayit.lock().unwrap().contains_key(&id)
    }

    /// Koruma kararına giden liste. `esik.rs` saf kalmak zorunda; oraya
    /// kilit değil hazır bir küme gidiyor.
    pub fn bagli_sekmeler(&self) -> Vec<SekmeId> {
        self.kayit.lock().unwrap().keys().copied().collect()
    }
}

pub struct Muiwatch {
    bulunan: Bulunan,
    pub oturumlar: Oturumlar,
}

impl Muiwatch {
    pub fn yeni() -> Self {
        Muiwatch {
            bulunan: Bulunan::yeni(),
            oturumlar: Oturumlar::yeni(),
        }
    }

    pub fn yol(&self) -> Option<PathBuf> {
        self.bulunan.yol(|| super::uygulama_ara(URUN, IKILI))
    }

    pub fn unut(&self) {
        self.bulunan.unut();
    }

    pub fn durum(&self) -> KopruDurumu {
        let yol = self.yol();
        KopruDurumu {
            ad: URUN,
            kurulu: yol.is_some(),
            yol: yol.map(|y| y.to_string_lossy().into_owned()),
        }
    }
}

impl Kopru for Muiwatch {
    fn ad(&self) -> &'static str {
        URUN
    }

    fn kurulu(&self) -> bool {
        self.yol().is_some()
    }

    fn devret(&self, yuk: Yuk) -> Sonuc<()> {
        let Yuk::Oda(oda) = yuk else {
            return Err(MuirenHata::Kopru("Muiwatch yalnız oda alıyor".into()));
        };
        if !oda_gecerli(&oda) {
            return Err(MuirenHata::Kopru("geçersiz oda kimliği".into()));
        }
        let ikili = self
            .yol()
            .ok_or_else(|| MuirenHata::KopruYok(URUN.into()))?;
        // `--oda <kimlik>`: Muiwatch bugün bu argümanı okumuyor (tek-örnek
        // eklentisi yok) ve uygulama yalnızca açılıyor. Argüman yine de
        // geçiriliyor — Muiwatch Faz 3'ü yazıldığında Muiren tarafında
        // değişiklik gerekmesin.
        super::calistir(&ikili, &["--oda", &oda])
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn normal_oda_gecerli() {
        assert!(oda_gecerli("abc123"));
        assert!(oda_gecerli("bolum-12_grup"));
    }

    #[test]
    fn bos_ve_uzun_oda_reddediliyor() {
        assert!(!oda_gecerli(""));
        assert!(!oda_gecerli(&"a".repeat(ODA_SINIRI + 1)));
    }

    #[test]
    fn tire_ile_baslayan_oda_reddediliyor() {
        // `--gizli-bayrak` bir oda kimliği değil; argüman olarak geçseydi
        // Muiwatch'a bayrak diye giderdi.
        assert!(!oda_gecerli("--gizli"));
        assert!(!oda_gecerli("-oda"));
    }

    #[test]
    fn bosluk_ve_ozel_karakter_reddediliyor() {
        assert!(!oda_gecerli("oda 1"));
        assert!(!oda_gecerli("oda&kotu"));
        assert!(!oda_gecerli("oda\"x"));
        assert!(!oda_gecerli("oda\nx"));
    }

    #[test]
    fn oturum_baglaniyor_ve_birakiliyor() {
        let o = Oturumlar::yeni();
        assert!(!o.bagli_mi(7));
        o.bagla(7, "abc".into());
        assert!(o.bagli_mi(7));
        assert_eq!(o.bagli_sekmeler(), vec![7]);
        o.birak(7);
        assert!(!o.bagli_mi(7));
    }
}
