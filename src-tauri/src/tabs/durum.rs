//! Sekme durum makinesi — saf.
//!
//! Girdi: mevcut durum + bir olay. Çıktı: yeni durum. Sistem çağrısı yok,
//! zaman okuma yok, motor çağrısı yok. Diyagram `docs/Bellek.md` içinde.
//!
//! ```text
//!                  tıklandı
//!       ┌──────────────────────────────┐
//!       │                              ▼
//!  ┌─────────┐  odak gitti      ┌──────────┐
//!  │Arkaplan │◄─────────────────│  Etkin   │
//!  └────┬────┘                  └──────────┘
//!       │ uyku eşiği                        ▲
//!       ▼                                   │
//!  ┌─────────┐                              │
//!  │ Uyuyan  │──────────────────────────────┤ tıklandı
//!  └────┬────┘                              │
//!       │ atma eşiği / sistem baskısı       │
//!       ▼                                   │
//!  ┌─────────┐                              │
//!  │ Atilmis │──────────────────────────────┘
//!  └─────────┘
//! ```
//!
//! Kararın **ne zaman** verileceği burada değil: onu `memory/esik.rs` (Faz 2)
//! veriyor. Burası yalnız "bu olay bu durumda ne yapar" sorusunun cevabı.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Durum {
    /// Bakılan sekme. Webview var ve görünür.
    Etkin,
    /// Webview var, gizli. Geri dönüş anında.
    Arkaplan,
    /// Webview var, askıda (`TrySuspend`). Sayfa durumu (form, video konumu,
    /// giriş yapılmış oturum) yerinde; uyanma göz kırpması kadar.
    Uyuyan,
    /// Webview **yok**. Sıfır render belleği. URL ve kaydırma konumundan
    /// yeniden doğuyor; doldurulmuş form gider — bu yüzden atma kararı
    /// ihtiyatlı verilir (`docs/Bellek.md`, koruma kuralları).
    Atilmis,
}

impl Durum {
    /// Bu durumda sekmenin webview'i olmalı mı.
    ///
    /// Tek yerde durması önemli: "hangi durumlarda webview var" sorusuna iki
    /// farklı yerde cevap verilirse, biri unutulduğunda sızan bir webview ya da
    /// ölü bir sekme oluyor.
    pub fn webview_gerekli(self) -> bool {
        !matches!(self, Durum::Atilmis)
    }

    /// Bu durum render belleği tüketiyor mu (bellek panelindeki "uyanık"
    /// sayısı).
    pub fn uyanik(self) -> bool {
        matches!(self, Durum::Etkin | Durum::Arkaplan)
    }
}

/// Durumu değiştirebilecek olaylar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Olay {
    /// Kullanıcı sekmeye tıkladı ya da klavyeyle geçti.
    Etkinlestirildi,
    /// Başka bir sekme etkin oldu.
    OdakGitti,
    /// Gözcü ya da kullanıcı uyuttu.
    Uyutuldu,
    /// Gözcü, kullanıcı ya da sistem baskısı attı.
    Atildi,
}

/// Neden bu geçiş oldu. Arayüzde ipucu olarak gösteriliyor
/// (`docs/IPC.md`, `sekme-durum-degisti` olayının `sebep` alanı).
///
/// Kullanıcı sekmesinin neden uyuduğunu bilmezse özelliği kapatıyor; sebebi
/// görünce ayarı değiştiriyor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Sebep {
    BostaKaldi,
    SistemBaskisi,
    OyunModu,
    /// Pencere simge durumundaydı — kullanıcı tarayıcıya bakmıyordu.
    ///
    /// `SistemBaskisi`den ayrı, çünkü ikisi kullanıcıya farklı şey söylüyor:
    /// biri "makinende bellek daralıyor", diğeri "sen bakmıyordun, ben de
    /// bekletmedim". Tek sebebe indirilseydi pencereyi küçülten kullanıcı,
    /// makinesinde olmayan bir bellek sorunu arardı.
    PencereGizli,
    Elle,
    UyanikSinir,
    /// Kullanıcı tıkladı — uyanma yönündeki geçişler.
    Etkilesim,
}

/// Saf geçiş.
///
/// Reddedilen bir geçiş hata değil, **aynı durum**: gözcü ile arayüz arasında
/// her zaman birkaç yüz milisaniye gecikme var ve gözcü "uyut" derken
/// kullanıcı o sekmeye tıklamış olabilir. O yarışta kullanıcı kazanıyor.
pub fn gecis(durum: Durum, olay: Olay) -> Durum {
    use Durum::*;
    use Olay::*;

    match (durum, olay) {
        // Tıklama her durumdan Etkin'e götürüyor. Atilmis ise yeniden
        // yükleniyor, Uyuyan ise Resume ediliyor — ikisi de arayüze aynı
        // görünüyor, farkı sürücü biliyor.
        (_, Etkinlestirildi) => Etkin,

        (Etkin, OdakGitti) => Arkaplan,
        (d, OdakGitti) => d,

        // Etkin sekme uyutulmaz/atılmaz: koruma kuralı #1 (`docs/Bellek.md`).
        // Burada da kapatılıyor ki eşik tarafındaki bir hata sekmeyi
        // kullanıcının gözünün önünde donduramasın.
        (Etkin, Uyutuldu) | (Etkin, Atildi) => Etkin,

        // Zaten atılmış bir sekmeyi uyutmanın karşılığı yok; Atilmis daha ucuz.
        (Atilmis, Uyutuldu) => Atilmis,
        (_, Uyutuldu) => Uyuyan,

        (_, Atildi) => Atilmis,
    }
}

#[cfg(test)]
mod testler {
    use super::Durum::*;
    use super::Olay::*;
    use super::*;

    #[test]
    fn tiklama_her_durumdan_etkine_goturuyor() {
        for d in [Etkin, Arkaplan, Uyuyan, Atilmis] {
            assert_eq!(gecis(d, Etkinlestirildi), Etkin, "{d:?}");
        }
    }

    #[test]
    fn odak_gidince_arkaplana_dusuyor() {
        assert_eq!(gecis(Etkin, OdakGitti), Arkaplan);
    }

    #[test]
    fn odak_gitti_uyuyani_uyandirmiyor() {
        // Bu bir tuzak: "odak gitti" olayı bütün sekmelere değil, yalnız eski
        // etkin sekmeye gidiyor. Yine de uyuyan bir sekmeye ulaşırsa uykuyu
        // bozmamalı — yoksa sekme değiştirmek bütün uykuyu iptal ederdi.
        assert_eq!(gecis(Uyuyan, OdakGitti), Uyuyan);
        assert_eq!(gecis(Atilmis, OdakGitti), Atilmis);
        assert_eq!(gecis(Arkaplan, OdakGitti), Arkaplan);
    }

    #[test]
    fn etkin_sekme_uyutulmuyor_ve_atilmiyor() {
        assert_eq!(gecis(Etkin, Uyutuldu), Etkin);
        assert_eq!(gecis(Etkin, Atildi), Etkin);
    }

    #[test]
    fn arkaplan_uyuyor_uyuyan_atiliyor() {
        assert_eq!(gecis(Arkaplan, Uyutuldu), Uyuyan);
        assert_eq!(gecis(Uyuyan, Atildi), Atilmis);
    }

    #[test]
    fn atilmis_sekme_uyutulmuyor() {
        assert_eq!(gecis(Atilmis, Uyutuldu), Atilmis);
    }

    #[test]
    fn arkaplan_dogrudan_atilabiliyor() {
        // Askıya alma yeteneği olmayan Runtime'da `Uyuyan` atlanıyor
        // (`docs/Setup.md` yetenek tablosu); politika doğrudan atmaya
        // düşüyor ve makine bunu kabul etmek zorunda.
        assert_eq!(gecis(Arkaplan, Atildi), Atilmis);
    }

    #[test]
    fn webview_yalniz_atilmista_yok() {
        assert!(Etkin.webview_gerekli());
        assert!(Arkaplan.webview_gerekli());
        assert!(Uyuyan.webview_gerekli());
        assert!(!Atilmis.webview_gerekli());
    }

    #[test]
    fn uyanik_sayimi_uyuyani_saymiyor() {
        assert!(Etkin.uyanik());
        assert!(Arkaplan.uyanik());
        assert!(!Uyuyan.uyanik());
        assert!(!Atilmis.uyanik());
    }
}
