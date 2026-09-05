//! Engelleme — pop-up politikası ve istek filtreleme.
//!
//! `docs/Roadmap.md` Faz 4 iki ayrı madde sayıyor ve **ayrı** kalıyorlar,
//! çünkü varsayılanları farklı:
//!
//! | | Varsayılan | Sebep |
//! |---|---|---|
//! | Pop-up ve yönlendirme engelleme | **açık** | Politika, liste değil. Kullanıcının bir şey yazmasını gerektirmiyor ve yanlış pozitifi ucuz: engellenen pencere arayüzde bir rozetle geri alınabiliyor. |
//! | İstek filtreleme | **kapalı** | Karar #5: "Filtre listeleri bile kullanıcı tarafından konuyor." Kutudan çıkan bir liste yok; boş bir filtreyi açık tutmanın karşılığı da yok. |
//!
//! ## Pop-up politikası neden bir liste değil
//!
//! `window.open` iki farklı şey olabiliyor: kullanıcının tıkladığı bağlantı
//! ve sayfanın kendi kendine açtığı pencere. WebView2 bu ayrımı zaten
//! yapıyor (`NewWindowRequestedEventArgs::IsUserInitiated`) ve doğru sinyal
//! bu — hangi sitenin pop-up açtığının listesini tutmak, her yeni siteyle
//! güncellenmesi gereken bir borç olurdu.
//!
//! Engellenen pencere **sessizce kaybolmuyor**: `muiren://engellendi` olayı
//! yayınlanıyor, arayüz adres çubuğunda bir rozet gösteriyor ve kullanıcı
//! "yine de aç" diyebiliyor. Sessiz engelleme, kullanıcının tarayıcıyı bozuk
//! sanması demek.

pub mod liste;

use std::sync::Mutex;

use serde::Serialize;

use crate::settings::Settings;
use crate::tabs::SekmeId;

/// Bir şeyin neden engellendiği (`docs/IPC.md`, `muiren://engellendi`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EngelSebebi {
    /// Sayfanın kendi kendine açtığı pencere.
    PopUp,
    /// Kullanıcı hareketi olmadan başlayan üst düzey yönlendirme.
    Yonlendirme,
    /// Kullanıcının filtre listesi.
    Filtre,
}

/// Engelleme kararlarının tek yeri.
///
/// Liste, ayarlar değiştiğinde **yeniden derleniyor** ([`Engelleyici::guncelle`])
/// ve her istekte değil: `WebResourceRequested` bir sayfada yüzlerce kez
/// tetikleniyor ve her seferinde metin ayrıştırmak, sayfa yüklemesini
/// ayrıştırıcıya bağlamak olurdu.
#[derive(Debug, Default)]
pub struct Engelleyici {
    liste: Mutex<liste::Liste>,
    /// Son derlemede anlaşılmayan satırlar; ayarlar ekranı gösteriyor.
    anlasilmayanlar: Mutex<Vec<String>>,
    /// Açık mı — ayarların kopyası. Her istekte `Settings` kilidi almamak
    /// için ayrı tutuluyor.
    filtre_acik: Mutex<bool>,
    popup_acik: Mutex<bool>,
    /// Sekme başına engellenen istek sayısı. Arayüz rozette gösteriyor.
    sayaclar: Mutex<std::collections::HashMap<SekmeId, u32>>,
}

/// Arayüze giden engelleme özeti.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngelOzeti {
    pub filtre_acik: bool,
    pub popup_acik: bool,
    pub kural_sayisi: usize,
    /// Ayrıştırılamayan satırlar. Sessizce atmak, kullanıcının çalışmayan
    /// bir listeyle dolaşması demek.
    pub anlasilmayanlar: Vec<String>,
}

impl Engelleyici {
    pub fn yeni(ayarlar: &Settings) -> Self {
        let e = Engelleyici::default();
        e.guncelle(ayarlar);
        e
    }

    /// Ayarlardan yeniden derler. `ayarlar_yaz` her çağrıldığında buraya
    /// uğruyor — iki kopya arasında ayrışma olmasın diye.
    pub fn guncelle(&self, ayarlar: &Settings) {
        let cozum = liste::Liste::coz(&ayarlar.filtre_kurallari);
        *self.liste.lock().unwrap() = cozum.liste;
        *self.anlasilmayanlar.lock().unwrap() = cozum.anlasilmayanlar;
        *self.filtre_acik.lock().unwrap() = ayarlar.filtre_acik;
        *self.popup_acik.lock().unwrap() = ayarlar.engelleme_acik;
    }

    pub fn ozet(&self) -> EngelOzeti {
        EngelOzeti {
            filtre_acik: *self.filtre_acik.lock().unwrap(),
            popup_acik: *self.popup_acik.lock().unwrap(),
            kural_sayisi: self.liste.lock().unwrap().kural_sayisi(),
            anlasilmayanlar: self.anlasilmayanlar.lock().unwrap().clone(),
        }
    }

    /// Bu istek engellensin mi.
    ///
    /// **Sıcak yol**: sayfa başına yüzlerce çağrı. Filtre kapalıysa ya da
    /// liste boşsa tek bir `bool` okumasıyla dönüyor — kapalı filtrenin
    /// maliyeti sıfıra yakın olmalı.
    pub fn istek_engelli_mi(&self, url: &str) -> bool {
        if !*self.filtre_acik.lock().unwrap() {
            return false;
        }
        let liste = self.liste.lock().unwrap();
        !liste.bos_mu() && liste.engelli_mi(url)
    }

    /// Yeni pencere isteği engellensin mi.
    ///
    /// `kullanici_baslatti` WebView2'den geliyor
    /// (`NewWindowRequestedEventArgs::IsUserInitiated`). Kullanıcının
    /// tıkladığı bağlantı **her zaman** açılıyor: pop-up engelleyicinin
    /// kullanıcıyı engellemesi, özelliği kapattıran bir hata.
    pub fn pencere_engelli_mi(&self, url: &str, kullanici_baslatti: bool) -> Option<EngelSebebi> {
        if self.istek_engelli_mi(url) {
            return Some(EngelSebebi::Filtre);
        }
        if kullanici_baslatti || !*self.popup_acik.lock().unwrap() {
            return None;
        }
        Some(EngelSebebi::PopUp)
    }

    /// Üst düzey gezinme engellensin mi.
    ///
    /// Kullanıcı hareketi olmadan başlayan **yönlendirmeler** engelleniyor;
    /// hareketsiz ilk gezinme değil. Ayrım şart: sayfanın kendi
    /// `<meta refresh>`i ile sunucunun 302'si arasındaki fark burada ve
    /// ikincisi webin normal işleyişi.
    ///
    /// Filtre kuralı ayrı: liste bir adresi engelliyorsa gezinme kullanıcı
    /// isteğiyle bile açılmıyor — kullanıcı o kuralı kendisi yazdı.
    pub fn gezinme_engelli_mi(
        &self,
        url: &str,
        kullanici_baslatti: bool,
        yonlendirme: bool,
    ) -> Option<EngelSebebi> {
        if self.istek_engelli_mi(url) {
            return Some(EngelSebebi::Filtre);
        }
        if !*self.popup_acik.lock().unwrap() || kullanici_baslatti || !yonlendirme {
            return None;
        }
        Some(EngelSebebi::Yonlendirme)
    }

    /// Sekmenin engellenen istek sayısını artırıp yeni değeri döndürür.
    pub fn say(&self, id: SekmeId) -> u32 {
        let mut sayaclar = self.sayaclar.lock().unwrap();
        let sayac = sayaclar.entry(id).or_insert(0);
        *sayac = sayac.saturating_add(1);
        *sayac
    }

    /// Sekmenin sayacı. Gezinme başlayınca sıfırlanıyor: rozet **bu
    /// sayfada** kaç istek engellendiğini gösteriyor, sekmenin ömrü boyunca
    /// kaçını değil.
    pub fn sayac(&self, id: SekmeId) -> u32 {
        self.sayaclar.lock().unwrap().get(&id).copied().unwrap_or(0)
    }

    pub fn sayaci_sifirla(&self, id: SekmeId) {
        self.sayaclar.lock().unwrap().remove(&id);
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::settings::{self, Settings};

    fn ayarlar(filtre: bool, popup: bool, kurallar: &[&str]) -> Settings {
        settings::duzelt(Settings {
            filtre_acik: filtre,
            engelleme_acik: popup,
            filtre_kurallari: kurallar.iter().map(|k| k.to_string()).collect(),
            ..Default::default()
        })
    }

    #[test]
    fn kapali_filtre_hicbir_seyi_engellemiyor() {
        let e = Engelleyici::yeni(&ayarlar(false, true, &["reklam.example.com"]));
        assert!(!e.istek_engelli_mi("https://reklam.example.com/x"));
    }

    #[test]
    fn acik_filtre_kurali_uyguluyor() {
        let e = Engelleyici::yeni(&ayarlar(true, true, &["reklam.example.com"]));
        assert!(e.istek_engelli_mi("https://reklam.example.com/x"));
        assert!(!e.istek_engelli_mi("https://haber.example.com/x"));
    }

    #[test]
    fn kullanici_tikladigi_baglanti_engellenmiyor() {
        // Pop-up engelleyicinin kullanıcıyı engellemesi, özelliği kapattıran
        // hata.
        let e = Engelleyici::yeni(&ayarlar(false, true, &[]));
        assert_eq!(e.pencere_engelli_mi("https://ornek.com/", true), None);
    }

    #[test]
    fn kendiliginden_acilan_pencere_engelleniyor() {
        let e = Engelleyici::yeni(&ayarlar(false, true, &[]));
        assert_eq!(
            e.pencere_engelli_mi("https://ornek.com/", false),
            Some(EngelSebebi::PopUp)
        );
    }

    #[test]
    fn popup_kapaliyken_pencere_aciliyor() {
        let e = Engelleyici::yeni(&ayarlar(false, false, &[]));
        assert_eq!(e.pencere_engelli_mi("https://ornek.com/", false), None);
    }

    #[test]
    fn filtre_kullanici_tiklamasini_da_engelliyor() {
        // Kuralı kullanıcı kendisi yazdı; tıklaması onu geçersiz kılmıyor.
        let e = Engelleyici::yeni(&ayarlar(true, false, &["reklam.example.com"]));
        assert_eq!(
            e.pencere_engelli_mi("https://reklam.example.com/", true),
            Some(EngelSebebi::Filtre)
        );
    }

    #[test]
    fn kullanici_baslatmayan_yonlendirme_engelleniyor() {
        let e = Engelleyici::yeni(&ayarlar(false, true, &[]));
        assert_eq!(
            e.gezinme_engelli_mi("https://kotu.example/", false, true),
            Some(EngelSebebi::Yonlendirme)
        );
    }

    #[test]
    fn normal_gezinme_engellenmiyor() {
        let e = Engelleyici::yeni(&ayarlar(false, true, &[]));
        // Kullanıcının yazdığı adres.
        assert_eq!(e.gezinme_engelli_mi("https://ornek.com/", true, false), None);
        // Yönlendirme olmayan, hareketsiz ilk yükleme (oturum geri yükleme).
        assert_eq!(
            e.gezinme_engelli_mi("https://ornek.com/", false, false),
            None
        );
        // Kullanıcının tıklamasıyla başlayan yönlendirme (302) — webin
        // normal işleyişi.
        assert_eq!(e.gezinme_engelli_mi("https://ornek.com/", true, true), None);
    }

    #[test]
    fn ayar_degisince_liste_yeniden_derleniyor() {
        let e = Engelleyici::yeni(&ayarlar(true, true, &[]));
        assert!(!e.istek_engelli_mi("https://reklam.example.com/x"));
        e.guncelle(&ayarlar(true, true, &["reklam.example.com"]));
        assert!(e.istek_engelli_mi("https://reklam.example.com/x"));
    }

    #[test]
    fn anlasilmayan_satirlar_ozette_gorunuyor() {
        let e = Engelleyici::yeni(&ayarlar(true, true, &["||izleyici.com^$third-party"]));
        let ozet = e.ozet();
        assert_eq!(ozet.kural_sayisi, 0);
        assert_eq!(ozet.anlasilmayanlar.len(), 1);
    }

    #[test]
    fn sayac_sekme_basina_tutuluyor() {
        let e = Engelleyici::default();
        assert_eq!(e.say(1), 1);
        assert_eq!(e.say(1), 2);
        assert_eq!(e.say(2), 1);
        assert_eq!(e.sayac(1), 2);
        e.sayaci_sifirla(1);
        assert_eq!(e.sayac(1), 0);
        assert_eq!(e.sayac(2), 1, "başka sekmenin sayacı sıfırlandı");
    }
}
