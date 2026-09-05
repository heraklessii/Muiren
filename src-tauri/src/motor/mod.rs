//! Motor — dış dünyaya açılan tek kapı.
//!
//! ```text
//!   commands/*.rs        ince sarmalayıcı, Tauri'ye bakan yüz
//!         │
//!   tabs/surucu.rs       "sekmeyi etkinleştir", "sekmeyi at" gibi işler
//!         │
//!   motor/{gercek,yok}.rs   ← burası: WebView2'ye giden tek yol
//! ```
//!
//! Dizin adı motorun adını taşımıyor (`webview2/` değil `motor/`) çünkü motorun
//! değişmesi gerçekçi bir ihtimal: WebView2 Windows'a bağlı ve süreç modeli
//! üzerinde kontrolümüz sınırlı. Gerekçe ve alternatiflerin değerlendirmesi
//! `docs/Architecture.md` ile `docs/Roadmap.md` karar #6'da.
//!
//! **Kural (CLAUDE.md #4):** [`Motor`] yüzeyine eklenen her metot hem
//! `gercek.rs` hem `yok.rs` içinde uygulanır. Trait bunu derleyiciye
//! zorlatıyor — biri unutulursa `--no-default-features` derlemesi kırılıyor.
//!
//! `unsafe` yalnız `gercek.rs` içinde. Başka bir modülde çıkarsa mimari
//! bozulmuş demektir.

#[cfg(feature = "motor")]
mod gercek;
#[cfg(feature = "motor")]
pub use gercek::{ek_bayraklar, WebView2Motor as Kurulu};

#[cfg(not(feature = "motor"))]
mod yok;
#[cfg(not(feature = "motor"))]
pub use yok::{ek_bayraklar, YokMotor as Kurulu};

use serde::{Deserialize, Serialize};
use tauri::Runtime;

use crate::hata::Sonuc;
use crate::tabs::SekmeId;

/// Bir WebView2 sürecinin sekme karşılığı (`memory/esleme.rs`).
///
/// Motorun ölçüme verdiği tek şey bu: bir PID ve o süreçte çerçevesi bulunan
/// sekmeler. **Bellek rakamı burada yok** ve olmayacak — onu Win32 ölçüyor
/// (`memory/olcum.rs`), ikisini birleştiren yer saf tarafta. Motor bellek de
/// ölçseydi `esleme.rs` içindeki hesap `--no-default-features` derlemesinde
/// test edilemezdi; ayrım `esik.rs`/`olcum.rs` ayrımının aynısı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurecKaydi {
    pub pid: u32,
    /// Render süreci mi. Değilse (tarayıcı, GPU, ağ, yardımcı) sekmeye
    /// düşmüyor, ortak gider sayılıyor.
    pub render: bool,
    /// Bu süreçte çerçevesi bulunan sekmeler.
    ///
    /// Alt çerçeveler (iframe) **üst düzey çerçevelerine kadar yürütülüp**
    /// sekmeye çevriliyor: gömülü bir oynatıcının ayrı süreci, onu gösteren
    /// sayfanın maliyetinin parçası.
    pub sekmeler: Vec<SekmeId>,
    /// Kabuğun kendi çerçevesi bu süreçte mi.
    ///
    /// Kabuk arayüzü de bir render sürecinde koşuyor ve o süreç listeye
    /// giriyor. Tanınmasaydı her turda "sahibi bulunamayan render süreci"
    /// sayılır, eşleme **hiçbir zaman** tam çıkmaz ve `olcum_yaklasik`
    /// kalıcı olarak true kalırdı (`docs/Bellek.md`).
    ///
    /// Sahiplik sayılıyor ama sekme sayılmıyor: kabuğun payı **ortak
    /// gidere** yazılıyor, çünkü kabuk bir sekmenin maliyeti değil,
    /// Muiren'in kendi maliyeti — kullanıcı onu kapatarak yer açamaz.
    pub kabuk: bool,
}

/// Sekme webview'inin pencere içindeki yeri, mantıksal (DPI'dan bağımsız)
/// piksel cinsinden.
///
/// Değeri **arayüz** bildiriyor (`icerik_alani` komutu): kabuğun yüksekliği,
/// yan panelin açık olup olmadığı ve tam ekran durumu orada biliniyor. Backend
/// bunu bir sabit olarak tutsaydı, panel açıldığında sayfa panelin altında
/// kalırdı (CLAUDE.md #9 ile aynı gerekçe: ölçü arayüzün işi).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dikdortgen {
    pub x: f64,
    pub y: f64,
    pub genislik: f64,
    pub yukseklik: f64,
}

impl Dikdortgen {
    /// Pencere henüz ölçülmeden bir sekme açılırsa kullanılan geçici alan.
    /// Arayüz ilk `icerik_alani` çağrısında düzeltiyor.
    pub const BOS: Dikdortgen = Dikdortgen {
        x: 0.0,
        y: 0.0,
        genislik: 1.0,
        yukseklik: 1.0,
    };
}

/// `ICoreWebView2_19::SetMemoryUsageTargetLevel` karşılığı.
///
/// Uyutmadan önceki ara adım: sayfa çalışmaya devam ediyor ama motor
/// önbelleklerini boşaltıyor. Yeteneği olmayan Runtime'da bu eylem hiç
/// üretilmiyor (`docs/Setup.md` yetenek tablosu).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BellekSeviyesi {
    Normal,
    Dusuk,
}

/// Kurulu WebView2 Runtime'ın **gerçekten** desteklediği çağrılar.
///
/// `cast()` başarısızlığı bir hata değil, bir yetenek yokluğu. Politika buna
/// göre geri çekiliyor: `askiya_alma` yoksa `Uyuyan` durumu atlanıp doğrudan
/// `Atilmis` kullanılıyor, tarayıcı çalışmaya devam ediyor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Yetenekler {
    /// Motor derlemeye dahil mi (`--no-default-features` değilse true).
    pub motor: bool,
    /// `ICoreWebView2_3`: `TrySuspend` / `Resume`.
    pub askiya_alma: bool,
    /// `ICoreWebView2_19`: `SetMemoryUsageTargetLevel`.
    pub bellek_hedefi: bool,
    /// `ICoreWebView2Environment13::GetProcessExtendedInfos` +
    /// `ICoreWebView2_20::FrameId`: süreç → sekme eşlemesi. **İkisi birden**
    /// gerekiyor ve ikisi ayrı soruyu cevaplıyor — biri süreçte hangi
    /// çerçevelerin olduğunu, diğeri hangi çerçevenin hangi sekmeye ait
    /// olduğunu. Yoksa bellek paneli sekme başına rakam gösteremiyor ve
    /// "yaklaşık" diyor (`docs/Bellek.md`, "Ölçüm").
    pub surec_bilgisi: bool,
    /// `ICoreWebView2_4`: `DownloadStarting`. Yoksa Muiget köprüsü indirmeyi
    /// yakalayamıyor ve motor kendi indirmesini yapıyor — karar #4'ün
    /// "kullanıcı indirme yapamaz duruma düşmüyor" yarısı.
    pub indirme_olayi: bool,
    /// `ICoreWebView2_15`: `FaviconChanged` + `GetFavicon`. Yoksa sekme
    /// çubuğunda yalnız durum noktası kalıyor.
    pub favicon: bool,
}

/// Motorun indirme olayında **ne yapacağı**.
///
/// Karar dinleyicide (`tabs/surucu.rs`), uygulama motorda. Bu ayrım
/// [`Motor`] trait'inin genel kuralıyla aynı: motor "iptal edeyim mi" diye
/// sormuyor, sorulan cevabı uyguluyor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndirmeKarari {
    /// WebView2 kendi indirmesini yapsın.
    Motorda,
    /// Motor indirmeyi iptal etsin; devir dinleyicinin işi.
    Iptal,
}

/// Motorun temizleyebileceği kalemler (`docs/IPC.md`, `veri_temizle`).
///
/// Muiren'in kendi depoları (geçmiş, yer imleri, favicon, oturum) **burada
/// yok**: onları motor bilmiyor ve temizlemesi sürücünün işi. Ayrım kasıtlı —
/// "hepsini sil" dediğinde iki tarafın da silinmesi gerekiyor ve tek bir
/// bayrak kümesi bunu gizlerdi.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorTemizligi {
    pub cerezler: bool,
    /// Disk önbelleği.
    pub onbellek: bool,
    /// `localStorage`, IndexedDB, Service Worker'lar — sitenin bıraktığı her
    /// şey. Çerezden ayrı tutuluyor çünkü çerez silmek oturumu kapatıyor,
    /// site verisi silmek sitenin çevrimdışı kopyasını siliyor; kullanıcı
    /// ikisini ayrı ayrı isteyebiliyor.
    pub site_verisi: bool,
    /// Otomatik doldurma (adres, kart ipuçları). **Şifreler dahil değil**:
    /// Muiren şifre saklamıyor (CLAUDE.md kapsam dışı) ve işletim sisteminin
    /// yöneticisine ait bir veriyi silmek bizim işimiz değil.
    pub otomatik_doldurma: bool,
}

impl MotorTemizligi {
    /// Silinecek bir şey var mı. Yoksa motora hiç gidilmiyor.
    pub fn bos_mu(&self) -> bool {
        !(self.cerezler || self.onbellek || self.site_verisi || self.otomatik_doldurma)
    }
}

/// Gezinmenin hangi aşamada olduğu (`docs/IPC.md`, `muiren://gezinme`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Asama {
    Basladi,
    Bitti,
}

/// Motordan yukarı doğru tek bağ.
///
/// Motor `tabs`ı tanımıyor: başlık değişince "sekme kaydını güncelle" demiyor,
/// "başlık değişti" diyor. Kaydı güncelleyen ve olayı yayınlayan
/// `tabs/surucu.rs`.
///
/// Dinleyici **zayıf** referansla tutuluyor ([`Motor::dinleyici_ata`]): güçlü
/// olsaydı sürücü → motor → sürücü döngüsü kurulur ve kapanışta ikisi de
/// düşmezdi.
pub trait Dinleyici: Send + Sync + 'static {
    /// `ham` doğrudan `document.title`; **temizlenmemiş** (CLAUDE.md #8).
    /// Temizleme dinleyicinin işi.
    fn baslik_degisti(&self, id: SekmeId, ham: String);
    fn adres_degisti(&self, id: SekmeId, url: String);
    fn gezinme(&self, id: SekmeId, asama: Asama, url: String, basarili: bool);
    fn gecmis_degisti(&self, id: SekmeId, geri_var: bool, ileri_var: bool);
    /// Sayfa ses çalmaya başladı/bitirdi. Koruma kuralı #2 buna dayanıyor:
    /// ses çalan sekme uyutulmuyor (`docs/Bellek.md`).
    fn ses_degisti(&self, id: SekmeId, caliyor: bool);
    /// `window.open` / `target="_blank"`. Motor asıl pencere isteğini
    /// engelliyor; sekmeyi açmak **ya da açmamak** dinleyicinin işi.
    ///
    /// `kullanici_baslatti` WebView2'nin kendi ayrımı
    /// (`NewWindowRequestedEventArgs::IsUserInitiated`) ve pop-up
    /// politikasının tek girdisi: kullanıcının tıkladığı bağlantı her zaman
    /// açılıyor (`engel/mod.rs`).
    fn yeni_pencere(&self, ebeveyn: SekmeId, url: String, kullanici_baslatti: bool);

    /// Üst düzey gezinmeye izin veriliyor mu.
    ///
    /// **Eşzamanlı**: `NavigationStarting` olayında `SetCancel` geri çağrı
    /// bitmeden çağrılmak zorunda. `indirme_basladi` ile aynı kural — gövde
    /// ucuz olmalı.
    fn gezinme_izni(
        &self,
        id: SekmeId,
        url: &str,
        kullanici_baslatti: bool,
        yonlendirme: bool,
    ) -> bool;

    /// Alt kaynak isteğine izin veriliyor mu (`WebResourceRequested`).
    ///
    /// **Sayfa başına yüzlerce çağrı.** Gövde bir `bool` okuması ve —
    /// filtre açıksa — bir küme sorgusu; başka hiçbir şey.
    fn istek_izni(&self, id: SekmeId, url: &str) -> bool;

    /// Bu webview için istek süzgeci kaydedilsin mi.
    ///
    /// Yaratma anında **bir kez** soruluyor. Sebebi maliyet:
    /// `WebResourceRequested` süzgeci her alt kaynağı kabuk sürecinden
    /// geçiriyor ve kapalı bir filtre için o bedeli ödemenin karşılığı yok.
    /// Sonucu: ayar değiştiğinde yalnız **sonradan açılan ya da yeniden
    /// yüklenen** sekmeler etkileniyor; arayüz bunu bir rozetle söylüyor
    /// (`surec_politikasi` rozetiyle aynı kalıp).
    fn istek_suzgeci_acik(&self) -> bool;
    /// Sayfa odaktayken bir kısayola basıldı. Ham tuş geliyor; tabloyu
    /// (`crate::kisayol`) dinleyici çalıştırıyor — motor hangi tuşun ne
    /// yaptığını bilmiyor.
    fn kisayol(&self, id: SekmeId, tus: String, ctrl: bool, shift: bool, alt: bool);
    /// Sayfanın dikey kaydırma oranı okundu (0.0–1.0). Atılmadan hemen önce
    /// isteniyor; atılmış sekme buradan geri geliyor (`docs/Sekmeler.md`).
    fn kaydirma_okundu(&self, id: SekmeId, oran: f32);

    /// Sayfa bir indirme başlattı.
    ///
    /// **Eşzamanlı dönüyor** ve bu bilinçli: WebView2 `DownloadStarting`
    /// olayında ya iptal edilir ya edilmez, ve karar geri çağrı bitmeden
    /// verilmek zorunda. Dinleyici burada yalnız ucuz okuma yapıyor (ayar +
    /// köprü önbelleği); asıl devir ayrı bir iş parçacığına gidiyor.
    ///
    /// `dosya_adi` motorun önerdiği ad — kökeni sayfanın
    /// `Content-Disposition` başlığı, yani **güvenilmez** (CLAUDE.md #8).
    fn indirme_basladi(
        &self,
        id: SekmeId,
        url: String,
        dosya_adi: String,
        boyut: u64,
    ) -> IndirmeKarari;

    /// Sayfa tam ekran bir öğe gösterdi/bıraktı (video, oyun). Koruma
    /// kuralı #8 buna dayanıyor (`docs/Bellek.md`).
    fn tam_ekran_degisti(&self, id: SekmeId, tam_ekran: bool);

    /// Sayfanın favicon'u değişti; ham baytlar geliyor.
    ///
    /// Adres **değil** baytlar: uzak bir adresi kabuğa basmak, açılan her
    /// sitenin kabuk penceresine ağ isteği yaptırabilmesi demek olurdu
    /// (`tabs::Sekme::favicon` notu). Baytları saklamak ve kimliğe çevirmek
    /// dinleyicinin işi.
    fn favicon_geldi(&self, id: SekmeId, veri: Vec<u8>);
}

/// Her iki uygulamanın da tutmak zorunda olduğu yüzey.
///
/// Burada bilinçli olarak **karar yok**: motor "bu sekmeyi uyut" der, "uyusun
/// mu" diye sormaz. Uyku kararı `memory/esik.rs` içinde ve saf.
pub trait Motor<R: Runtime>: Send + Sync + 'static {
    fn yetenekler(&self) -> Yetenekler;

    /// Motor olaylarını alacak dinleyici. Sürücü kurulduktan **sonra**
    /// bağlanıyor.
    fn dinleyici_ata(&self, dinleyici: std::sync::Weak<dyn Dinleyici>);

    /// Sekme için bir webview yaratır ve `url`e gider.
    ///
    /// Konum/boyut için en son bildirilen içerik alanı kullanılıyor
    /// ([`Motor::alan_ayarla`]). Yeni webview **gizli** doğuyor; görünür hâle
    /// gelmesi [`Motor::goster`] ile.
    ///
    /// `gizli` webview'i `SetIsInPrivateModeEnabled` ile doğuruyor. Bu bir
    /// **denetleyici** seçeneği, ortam seçeneği değil — yani CLAUDE.md #16'yı
    /// bozmuyor: bayrak dizesi ve kullanıcı veri klasörü aynı kalıyor,
    /// dolayısıyla gizli sekmenin WebView2 ortamı da kabuğunkiyle uyumlu
    /// (`motor/gercek.rs`, `sekme_ac`).
    fn sekme_ac(&self, id: SekmeId, url: &str, gizli: bool) -> Sonuc<()>;
    fn sekme_kapat(&self, id: SekmeId) -> Sonuc<()>;

    fn gezin(&self, id: SekmeId, url: &str) -> Sonuc<()>;
    fn geri(&self, id: SekmeId) -> Sonuc<()>;
    fn ileri(&self, id: SekmeId) -> Sonuc<()>;
    fn yenile(&self, id: SekmeId) -> Sonuc<()>;
    fn durdur(&self, id: SekmeId) -> Sonuc<()>;

    /// İçerik alanını kaydeder ve o an görünür olan webview'i oraya taşır.
    fn alan_ayarla(&self, dikdortgen: Dikdortgen) -> Sonuc<()>;
    fn goster(&self, id: SekmeId, dikdortgen: Dikdortgen) -> Sonuc<()>;
    fn gizle(&self, id: SekmeId) -> Sonuc<()>;
    /// Sekmenin sesini kapatır/açar. Sessize alınan sekme uyku korumasını
    /// kaybediyor — kararı sürücü veriyor, motor yalnız uyguluyor.
    fn ses_kes(&self, id: SekmeId, sessiz: bool) -> Sonuc<()>;

    // Bellek politikasının motordan istediği tek şey bunlar:
    fn uyut(&self, id: SekmeId) -> Sonuc<()>;
    fn uyandir(&self, id: SekmeId) -> Sonuc<()>;
    fn bellek_hedefi(&self, id: SekmeId, seviye: BellekSeviyesi) -> Sonuc<()>;
    /// Webview'i tamamen yıkar. Sekme **kaydı** duruyor; kullanıcı tıklayınca
    /// `sekme_ac` ile URL'den yeniden doğuyor (`docs/Sekmeler.md`).
    fn at(&self, id: SekmeId) -> Sonuc<()>;

    /// Bu sekmenin şu an bir webview'i var mı.
    fn var_mi(&self, id: SekmeId) -> bool;

    /// Sayfanın dikey kaydırma oranını **ister**; cevap
    /// [`Dinleyici::kaydirma_okundu`] ile geliyor.
    ///
    /// Ateşle-unut, çünkü `ExecuteScript` eşzamansız ve sonucu beklemek ana
    /// iş parçacığını sayfanın JS motoruna bağlardı. Sekme atılırken en son
    /// bilinen oran kullanılıyor; bir tur geriden gelmesi kabul edilebilir
    /// (bir ekranlık kayma), sayfayı bekletmek değil.
    fn kaydirma_iste(&self, id: SekmeId) -> Sonuc<()>;

    /// Sayfayı verilen orana kaydırır. Atılmış sekme yeniden yüklendikten
    /// sonra çağrılıyor.
    fn kaydirma_uygula(&self, id: SekmeId, oran: f32) -> Sonuc<()>;

    /// Tarayıcı verisini siler (`ICoreWebView2Profile2::ClearBrowsingData`).
    ///
    /// **Ateşle-unut.** Silme eşzamansız ve sonucu beklemek arayüzü, motorun
    /// disk işine bağlardı. Kullanıcıya "silindi" değil "silme başlatıldı"
    /// demek daha dürüst olurdu ama fark ölçülemeyecek kadar küçük: işlem
    /// saniyenin altında bitiyor ve başarısızlığı `MUIREN_LOG` görüyor.
    ///
    /// `id` **yok**: profil bütün sekmelerde ortak. Bir sekmenin çerezini
    /// silmek diye bir şey yok; profilin çerezleri var.
    fn veri_temizle(&self, kalemler: MotorTemizligi) -> Sonuc<()>;

    /// Süreç → sekme eşlemesini **ister**; cevap [`Motor::surec_haritasi`]
    /// ile geliyor.
    ///
    /// Ateşle-unut, [`Motor::kaydirma_iste`] ile aynı gerekçe:
    /// `GetProcessExtendedInfos` eşzamansız ve bellek gözcüsü kendi iş
    /// parçacığında koşuyor. Cevabı beklemek gözcüyü her turda WebView2'nin
    /// kendi döngüsüne bağlardı — tam da "karar veren kod arayüzden bağımsız
    /// çalışmalı" kuralının kırıldığı yer.
    ///
    /// **Uyuyan sekmeye dokunmuyor** (CLAUDE.md #5): çağrı ortam düzeyinde,
    /// sayfaya hiç inmiyor ve hiçbir betik çalıştırmıyor.
    fn surec_bilgisi_iste(&self) -> Sonuc<()>;

    /// Son bilinen süreç → sekme anlık görüntüsü.
    ///
    /// Bir tur geriden gelebiliyor ve bu kabul edilmiş: gözcünün periyodu
    /// saniyeler mertebesinde, süreç tablosu o sürede kendini tekrarlıyor.
    /// Anlık görüntü hiç yoksa boş dönüyor ve `esleme.rs` "eşleme tam değil"
    /// diyor — doğrulanmadan `tam` denmiyor.
    fn surec_haritasi(&self) -> Vec<SurecKaydi>;

    /// Kurulu WebView2 Runtime sürümü — bilinmiyorsa `None`.
    ///
    /// **Neden motor yüzeyinde:** `docs/olcumler/README.md` her raporun
    /// başında bu sürümü şart koşuyor ve gerekçesi somut — WebView2 Runtime
    /// kendi kendine güncelleniyor, iki ölçüm arasındaki farkın sebebi bizim
    /// kodumuz değil o güncelleme olabilir. Ölçüm modu raporu Rust tarafında
    /// yazıyor ve kullanıcıdan sürümü elle bulmasını istemek, protokolün
    /// "sürüm alanları boş bırakılmıyor" kuralını ilk koşuda kırardı.
    ///
    /// Çağrı bir webview gerektirmiyor (ortam düzeyinde bile değil, kurulu
    /// çalışma zamanına bakıyor), dolayısıyla uyuyan bir sekmeye dokunma
    /// riski yok (CLAUDE.md #5).
    fn calisma_zamani_surumu(&self) -> Option<String>;
}

/// Sekme webview'lerinin etiket öneki.
///
/// Tauri webview'leri etiketle adresliyor; `SekmeId`yi etikete çevirmenin tek
/// yeri burası. Kabuk webview'i ayrı bir etiket kullanıyor ([`KABUK`]) ve izin
/// dosyası (`capabilities/varsayilan.json`) yalnız onu tanıyor — sekme
/// webview'leri, yani açılan web sayfaları, hiçbir ayrıcalıklı kanal görmüyor.
pub const SEKME_ONEKI: &str = "sekme-";
pub const KABUK: &str = "kabuk";
pub const ANA_PENCERE: &str = "ana";

pub fn etiket(id: SekmeId) -> String {
    format!("{SEKME_ONEKI}{id}")
}
