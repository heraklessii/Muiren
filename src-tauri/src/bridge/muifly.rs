//! Muifly köprüsü — oyun modu.
//!
//! ## Faz 4'te doğrulanan: köprünün karşı ucu yok
//!
//! `docs/Kopruler.md` şunu doğrulanacak diye bırakmıştı: *"`muifly://durum`
//! bir Tauri olayı — yani süreç içi. Süreçler arası dinlemek için Muifly
//! tarafında bir dışa açılma noktası gerekiyor."*
//!
//! Bakıldı: `Muifly/src-tauri/src/commands.rs` içinde `OLAY_DURUM =
//! "muifly://durum"` ve olay `app.emit` ile yayınlanıyor. **Süreç içi.**
//! Named pipe, tek-örnek mesajı ya da izlenebilir bir durum dosyası yok.
//!
//! Doğrulamanın sonucu, doğrulamayı isteyen cümlenin kendi yazdığı sonuç:
//!
//! > "Karşılığı yoksa Faz 4 yalnız kendi algılamamızla çıkıyor."
//!
//! Yani bu modül bugün **iki iş** yapıyor:
//!
//! 1. Muifly kurulu mu — arayüz "Muifly ile" ifadesini yalnız kuruluysa
//!    gösteriyor.
//! 2. [`super::oyun`] ile **kendi algılamamızı** koşturuyor, varsayılan
//!    kapalı (`oyun_algilama` ayarı).
//!
//! Muifly'a bir şey **gönderilmiyor** ve gönderilmeyecek: `docs/Kopruler.md`,
//! "Muiren → Muifly: bir şey göndermiyoruz. Tarayıcının oyun optimizasyon
//! aracına söyleyecek sözü yok."
//!
//! ## Neden bir izleme döngüsü, gözcünün turu değil
//!
//! Bellek gözcüsü 10 saniyede bir dönüyor (`gozcu_periyodu_sn`). Oyun
//! algılaması ondan **daha sık** olmalı: oyun açıldıktan 10 saniye sonra
//! sekmeleri uyutmak, tam da oyunun yüklendiği ve belleğin en gergin olduğu
//! anı kaçırmak demek. Ayrı bir döngü, ayrı bir periyot.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::{Bulunan, Kopru, KopruDurumu, Yuk};
use crate::hata::{MuirenHata, Sonuc};

const URUN: &str = "Muifly";
const IKILI: &str = "Muifly.exe";

/// Algılama turu. Gözcünün 10 saniyesinden kısa: oyun açılışı belleğin en
/// gergin olduğu an ve orada geç kalmanın karşılığı yok.
pub const TUR: Duration = Duration::from_secs(3);

/// Oyun modunun **kim tarafından** açıldığı (`docs/IPC.md`,
/// `muiren://oyun-modu` olayının `kaynak` alanı).
///
/// Arayüzde görünüyor çünkü fark kullanıcı için gerçek: elle açtığı modu
/// kendisi kapatır, algılanan modu "yanlış algıladı" diye kapatır ve ikincisi
/// ayarlara gitmesi gereken bir geri bildirim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kaynak {
    Elle,
    Algilama,
}

pub struct Muifly {
    bulunan: Bulunan,
    /// İzleme döngüsü koşuyor mu. İki kez başlatılmasın diye.
    izliyor: AtomicBool,
}

impl Muifly {
    pub fn yeni() -> Self {
        Muifly {
            bulunan: Bulunan::yeni(),
            izliyor: AtomicBool::new(false),
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

    /// İzleme döngüsünü bir kez başlatır.
    ///
    /// `ayar_acik` her turda **yeniden** soruluyor, açılışta bir kez değil:
    /// kullanıcı ayarı çalışırken değiştirdiğinde döngünün yeniden
    /// başlatılmasını beklemek "ayarı açtım ama çalışmıyor" demek olurdu.
    ///
    /// `uygula` yalnız **değişimde** çağrılıyor. Her turda "oyun modu açık"
    /// demek, gözcüye ve arayüze saniyede bir gereksiz olay göndermek olurdu.
    pub fn izlemeyi_baslat<A, U>(self: &Arc<Self>, ayar_acik: A, uygula: U)
    where
        A: Fn() -> bool + Send + 'static,
        U: Fn(bool, Kaynak) + Send + 'static,
    {
        if self.izliyor.swap(true, Ordering::AcqRel) {
            return;
        }
        let zayif = Arc::downgrade(self);
        std::thread::Builder::new()
            .name("muiren-oyun".into())
            .spawn(move || {
                let mut onceki = false;
                loop {
                    std::thread::sleep(TUR);
                    // Köprü düştüyse (uygulama kapandı) döngü de bitiyor.
                    if zayif.upgrade().is_none() {
                        return;
                    }
                    if !ayar_acik() {
                        // Ayar kapatıldığında algılamanın açtığı mod da
                        // kapanıyor: aksi hâlde kullanıcı ayarı kapatıp
                        // sekmelerinin neden hâlâ uyuduğunu anlamıyor.
                        if onceki {
                            onceki = false;
                            uygula(false, Kaynak::Algilama);
                        }
                        continue;
                    }
                    let simdi = super::oyun::oyun_calisiyor();
                    if simdi != onceki {
                        onceki = simdi;
                        uygula(simdi, Kaynak::Algilama);
                    }
                }
            })
            .ok();
    }
}

impl Kopru for Muifly {
    fn ad(&self) -> &'static str {
        URUN
    }

    fn kurulu(&self) -> bool {
        self.yol().is_some()
    }

    /// Muifly'a **hiçbir şey gönderilmiyor** (`docs/Kopruler.md`).
    ///
    /// Yüzey [`Kopru`] için var; gövdesi bilinçli olarak hata. Sessizce
    /// `Ok(())` dönseydi, bir gün buraya bir çağrı eklenir ve hiçbir şey
    /// yapmadığı fark edilmezdi.
    fn devret(&self, _yuk: Yuk) -> Sonuc<()> {
        Err(MuirenHata::Kopru(
            "Muifly'a devir yok: tarayıcının oyun aracına söyleyecek sözü yok".into(),
        ))
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn muifly_devir_kabul_etmiyor() {
        let k = Muifly::yeni();
        assert!(k.devret(Yuk::Oda("abc".into())).is_err());
    }

    #[test]
    fn izleme_bir_kez_basliyor() {
        let k = Arc::new(Muifly::yeni());
        // Ayar kapalı: döngü hiçbir şey uygulamıyor ama bayrağı alıyor.
        k.izlemeyi_baslat(|| false, |_, _| {});
        assert!(k.izliyor.load(Ordering::Acquire));
        // İkinci çağrı ikinci bir iş parçacığı doğurmuyor.
        k.izlemeyi_baslat(|| false, |_, _| {});
        assert!(k.izliyor.load(Ordering::Acquire));
    }
}
