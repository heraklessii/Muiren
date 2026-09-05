//! Tek hata tipi.
//!
//! `docs/IPC.md`: hata **serileştirilebilir bir enum**, backend Türkçe metin
//! göndermiyor. Sebep şu: arayüz metni kendi çeviriyor ve kendi bağlamına göre
//! biçimlendiriyor ("bu sekme kapanmış" ile "sekme bulunamadı" aynı hatanın iki
//! farklı cümlesi). Backend cümle gönderirse arayüzün elinde string eşleştirmek
//! dışında seçenek kalmıyor.
//!
//! `ayrinti` alanı çevrilmeyen teknik veri taşıyor: bir HRESULT, bir dosya
//! yolu, bir adres. Kullanıcıya gösterilirse gösteriliyor ama cümlenin kendisi
//! arayüzden geliyor.

use serde::Serialize;

use crate::tabs::SekmeId;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "tur", content = "ayrinti")]
pub enum MuirenHata {
    /// Verilen id'de sekme yok. Gözcü ile arayüz arasındaki gecikmede normal:
    /// arayüz kapanmış bir sekmeye komut göndermiş olabilir.
    SekmeYok(SekmeId),
    /// Gelen dize adrese çevrilemedi. Ayrıntı: ham dize.
    GecersizAdres(String),
    /// `--no-default-features` derlemesi. Kabuk çalışıyor, sayfa açılmıyor.
    MotorYok,
    /// Kurulu WebView2 Runtime bu çağrıyı desteklemiyor (`docs/Setup.md`,
    /// "Yetenekler"). Bir hata değil, bir yetenek yokluğu; politika buna göre
    /// geri çekiliyor. Ayrıntı: yeteneğin adı.
    YetenekYok(String),
    /// Motorun kendi hatası. Ayrıntı: ham mesaj (çevrilmiyor).
    Motor(String),
    /// Disk. Ayrıntı: ham mesaj.
    Dosya(String),
    /// Ayrıştırma/serileştirme. Ayrıntı: ham mesaj.
    Bicim(String),
    /// Kardeş uygulama kurulu değil. Ayrıntı: uygulamanın adı.
    ///
    /// Bir hata **değil**, bir yokluk (`docs/Kopruler.md`: köprüler isteğe
    /// bağlı). Arayüz bunu gördüğünde uyarı basmıyor; menü öğesini hiç
    /// çizmemesi gerekiyordu zaten ve buraya düşmesi yarış demek
    /// (kullanıcı Muiren açıkken uygulamayı kaldırmış).
    KopruYok(String),
    /// Devretme başarısız. Ayrıntı: ham sebep (çevrilmiyor).
    ///
    /// **Sessiz kalmıyor** (`docs/Kopruler.md`): köprü kırıldığında kullanıcı
    /// tarayıcıyı suçlamasın diye arayüz küçük bir bildirim gösteriyor.
    Kopru(String),
}

pub type Sonuc<T> = std::result::Result<T, MuirenHata>;

impl std::fmt::Display for MuirenHata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Yalnız günlük ve `Debug` için. Kullanıcı bu metni görmüyor.
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MuirenHata {}

impl From<tauri::Error> for MuirenHata {
    fn from(e: tauri::Error) -> Self {
        MuirenHata::Motor(e.to_string())
    }
}

impl From<std::io::Error> for MuirenHata {
    fn from(e: std::io::Error) -> Self {
        MuirenHata::Dosya(e.to_string())
    }
}

impl From<serde_json::Error> for MuirenHata {
    fn from(e: serde_json::Error) -> Self {
        MuirenHata::Bicim(e.to_string())
    }
}

impl From<url::ParseError> for MuirenHata {
    fn from(e: url::ParseError) -> Self {
        MuirenHata::GecersizAdres(e.to_string())
    }
}
