//! Muiply köprüsü — yerel oynatma devri.
//!
//! Muiren'in codec eksiği (özellikle HEVC, `docs/Medya.md`) Muiply'ın
//! libmpv'siyle kapanıyor: tarayıcının açamadığı yerel dosyayı kardeş uygulama
//! açıyor.
//!
//! ## Devir yolu
//!
//! Muiply dosya yolunu **komut satırından** alıyor
//! (`Muiply/src-tauri/src/acilis.rs`, `dosya_argumanlari`): `-` ile başlamayan
//! her argüman bir yol sayılıyor, tek-örnek eklentisi de ikinci çağrıyı zaten
//! açık pencereye taşıyor. Yani devir tek satır: `muiply.exe <yol>`.
//!
//! Bunun bir sonucu var ve kodda karşılığı: **`-` ile başlayan bir yol
//! gönderilemez.** Muiply onu bayrak sayıp atardı ve kullanıcı "hiçbir şey
//! olmadı" görürdü. [`Muiply::devret`] bu durumda yolu `.\` ile
//! önekliyor.
//!
//! ## Sınır: yalnız yerel dosya
//!
//! Muiply **bilinçli olarak ağ akışı oynatmıyor** (Muiply CLAUDE.md, kapsam
//! dışı kararlar). Uzak URL devri anlamsız; [`yerel_dosya`] `file://` dışındaki
//! her şeyi eliyor. Muiply bir gün uzak URL desteklerse burası genişletilir —
//! o zamana kadar var olmayan bir özelliğin menüsünü çizmiyoruz.

use std::path::{Path, PathBuf};

use super::{Bulunan, Kopru, KopruDurumu, Yuk};
use crate::hata::{MuirenHata, Sonuc};

const URUN: &str = "Muiply";
const IKILI: &str = "Muiply.exe";

/// Muiply'a devredilmeye değer uzantılar.
///
/// Liste **dar tutuluyor**: bağlam menüsünde "Muiply ile aç" seçeneğinin her
/// bağlantıda çıkması, seçeneği anlamsızlaştırırdı. Muiply'ın kendisi uzantıya
/// bakmadan açmayı deniyor (kullanıcı bilerek bıraktıysa deneme hakkı var);
/// burada dar olan **öneri**, yetenek değil.
const MEDYA_UZANTILARI: &[&str] = &[
    // video
    "mkv", "mp4", "m4v", "avi", "mov", "webm", "wmv", "flv", "ts", "m2ts", "mpg", "mpeg", "ogv",
    // ses
    "mp3", "flac", "m4a", "aac", "ogg", "opus", "wav", "wma", "alac",
];

/// Adres yerel bir medya dosyasını mı gösteriyor.
///
/// **Saf ve testli** çünkü buranın yanlışı iki yönde de kötü: uzak bir adresi
/// yerel sanmak Muiply'ı boşuna açıyor, yerel bir dosyayı kaçırmak köprünün
/// hiç görünmemesi demek.
///
/// `file://` dışındaki şemalar ve medya olmayan uzantılar eleniyor. Yolun
/// **var olup olmadığına bakılmıyor**: bu fonksiyon saf kalıyor, varlık
/// kontrolü [`Muiply::devret`] içinde.
pub fn yerel_dosya(url: &str) -> Option<PathBuf> {
    let ayristirilmis = url::Url::parse(url).ok()?;
    if ayristirilmis.scheme() != "file" {
        return None;
    }
    // `file://sunucu/pay/a.mkv` bir UNC yolu; `to_file_path` onu da çözüyor.
    let yol = ayristirilmis.to_file_path().ok()?;
    let uzanti = yol.extension()?.to_str()?.to_ascii_lowercase();
    if !MEDYA_UZANTILARI.contains(&uzanti.as_str()) {
        return None;
    }
    Some(yol)
}

pub struct Muiply {
    bulunan: Bulunan,
}

impl Muiply {
    pub fn yeni() -> Self {
        Muiply {
            bulunan: Bulunan::yeni(),
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

/// Muiply'ın argüman süzgecinden geçecek hâle getirir.
///
/// `Muiply/src-tauri/src/acilis.rs`: `-` ile başlayan argümanlar bayrak
/// sayılıp **atılıyor**. `-film.mkv` adlı bir dosya bu yüzden hiç açılmıyordu;
/// `.\` öneki yolu aynı dosyaya bırakıp süzgeçten kurtarıyor.
///
/// Saf ve testli: belirtisi "bazen hiçbir şey olmuyor" olan hatalar en pahalı
/// olanlar.
pub fn arguman_guvenli(yol: &Path) -> String {
    let dize = yol.to_string_lossy().into_owned();
    if dize.starts_with('-') {
        format!(r".\{dize}")
    } else {
        dize
    }
}

impl Kopru for Muiply {
    fn ad(&self) -> &'static str {
        URUN
    }

    fn kurulu(&self) -> bool {
        self.yol().is_some()
    }

    fn devret(&self, yuk: Yuk) -> Sonuc<()> {
        let Yuk::DosyaYolu(yol) = yuk else {
            return Err(MuirenHata::Kopru("Muiply yalnız dosya yolu alıyor".into()));
        };
        if !yol.is_file() {
            // Var olmayan bir yolu göndermek Muiply'ı boş açardı; kullanıcı
            // "oynatıcı bozuk" diye okurdu.
            return Err(MuirenHata::Kopru("dosya bulunamadı".into()));
        }
        let ikili = self
            .yol()
            .ok_or_else(|| MuirenHata::KopruYok(URUN.into()))?;
        super::calistir(&ikili, &[&arguman_guvenli(&yol)])
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn yerel_video_taniniyor() {
        let yol = yerel_dosya("file:///C:/Filmler/bolum.mkv");
        assert!(yol.is_some(), "yerel mkv tanınmadı");
        assert!(yol.unwrap().to_string_lossy().ends_with("bolum.mkv"));
    }

    #[test]
    fn uzak_adres_devredilmiyor() {
        // Muiply ağ akışı oynatmıyor (Muiply CLAUDE.md, kapsam dışı).
        assert!(yerel_dosya("https://ornek.com/video.mkv").is_none());
        assert!(yerel_dosya("blob:https://ornek.com/abc").is_none());
    }

    #[test]
    fn medya_olmayan_yerel_dosya_devredilmiyor() {
        assert!(yerel_dosya("file:///C:/rapor.pdf").is_none());
        assert!(yerel_dosya("file:///C:/klasor").is_none());
    }

    #[test]
    fn uzanti_buyuk_harf_olabiliyor() {
        assert!(yerel_dosya("file:///C:/A.MKV").is_some());
    }

    #[test]
    fn yuzde_kodlu_ad_cozuluyor() {
        let yol = yerel_dosya("file:///C:/Filmler/bir%20iki.mp4").unwrap();
        assert!(yol.to_string_lossy().contains("bir iki.mp4"));
    }

    #[test]
    fn tire_ile_baslayan_yol_bayrak_sanilmiyor() {
        // Muiply `-` ile başlayan argümanı bayrak sayıp ATIYOR; önek
        // olmadan dosya hiç açılmıyordu.
        let guvenli = arguman_guvenli(Path::new("-film.mkv"));
        assert!(!guvenli.starts_with('-'), "bayrak sanılacak: {guvenli}");
        assert!(guvenli.ends_with("-film.mkv"));
    }

    #[test]
    fn normal_yol_degistirilmiyor() {
        assert_eq!(
            arguman_guvenli(Path::new(r"C:\Filmler\a.mkv")),
            r"C:\Filmler\a.mkv"
        );
    }
}
