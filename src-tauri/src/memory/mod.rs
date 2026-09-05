//! Bellek politikası — **projenin sebebi** (`docs/Bellek.md`).
//!
//! Dört dosya, dört ayrı sorumluluk ve aralarındaki sınır kasıtlı:
//!
//! ```text
//!   olcum.rs   "makinede ne kadar boş RAM var"      → Win32, saf değil
//!   esik.rs    "hangi sekme ne zaman uyusun"        → SAF, tamamen testli
//!   gozcu.rs   "bunu her N saniyede bir yap"        → döngü + uygulama
//!   gecikme.rs "uyanma ne kadar sürdü"              → SAF, tamamen testli
//!   esleme.rs  "hangi süreç hangi sekmenin"         → SAF, tamamen testli
//! ```
//!
//! Kararın saf tarafta durması, projenin tezinin `--no-default-features`
//! derlemesinde bile test edilebilmesi demek. Bir Win32 çağrısı `esik.rs`
//! içine sızarsa o derleme kırılıyor ve bunu derleyici söylüyor.

pub mod esik;
pub mod esleme;
pub mod gecikme;
pub mod gozcu;
pub mod olcum;

use serde::Serialize;

use esik::BellekBaskisi;
use esleme::SekmeBellegi;

/// Bellek panelinin beslendiği özet (`docs/IPC.md`).
///
/// `tahmini_kazanc_mb` arayüzde "Muiren şu an ~3.2 GB tasarruf ediyor" olarak
/// gösteriliyor — projenin vitrini. `olcum_yaklasik` true ise arayüz `~`
/// işareti ve bir ipucu ekliyor: **yaklaşık değeri kesin gibi göstermek yok**
/// (`docs/Bellek.md`).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BellekOzeti {
    pub baski: BellekBaskisi,
    /// WebView2 süreçlerinin toplamı (MB).
    pub toplam_mb: u64,
    pub kabuk_mb: u64,
    /// Sistemin toplam ve boş fiziksel belleği — baskı çubuğunun kaynağı.
    pub sistem_toplam_mb: u64,
    pub sistem_bos_mb: u64,
    pub etkin: u32,
    pub arkaplan: u32,
    pub uyuyan: u32,
    pub atilmis: u32,
    /// Uyuyan + atılmış sekmelerin bellekte olsalardı tutacakları yer.
    ///
    /// Eşleme çalışıyorsa **ölçülmüş**: her pasif sekmenin uyanıkken tuttuğu
    /// yerden bugün tuttuğu çıkarılıyor (`esleme::Tur::kazanc_mb`). Eşleme
    /// eksikse ya da bir sekme uyanıkken hiç ölçülmediyse o sekme için
    /// tahmine düşülüyor ve `olcum_yaklasik` true kalıyor.
    pub tahmini_kazanc_mb: u64,
    /// Süreç → sekme eşlemesi güvenilmezse `true`.
    ///
    /// İki koşulun **ikisi birden** sağlanmadıkça true kalıyor: eşlemenin
    /// tam olması (her render sürecinin sahibi bulunmuş) ve her pasif
    /// sekmenin uyanıkken en az bir kez ölçülmüş olması. Yarısı sağlanınca
    /// "kesin" demek, panelin tek büyük rakamını olduğundan güvenilir
    /// gösterirdi (`docs/Bellek.md`, "Ölçüm").
    pub olcum_yaklasik: bool,
    /// Sekme başına ölçülen bellek; yalnız değeri **olan** sekmeler.
    ///
    /// `SekmeOzeti` içine konmadı ve sebebi somut: o yapı `esik.rs`in
    /// girdisi ve eşik kararı sekme başına rakama **dayanmıyor**
    /// (`docs/Bellek.md`). Oraya bir `bellek_mb` alanı koymak, saf karara
    /// bir gün sızacak bir veriyi elinin altına bırakmak olurdu; panelin
    /// ihtiyacı olan yer ise zaten burası.
    pub sekme_mb: Vec<SekmeBellegi>,
    /// Sekmelere düşmeyen WebView2 belleği: tarayıcı, GPU, ağ ve yardımcı
    /// süreçler.
    ///
    /// Panelde ayrı bir satır. Sekme başına rakamların toplamı `toplam_mb`ye
    /// eşit çıkmıyor ve farkı söylemeyen bir panel, kullanıcıya ölçümün
    /// bozuk olduğunu düşündürür.
    pub ortak_mb: u64,
    /// Muiren'in doğurduğu WebView2 süreç sayısı.
    ///
    /// Faz 0/R4'ün ölçüm aracı: aynı siteden 10 sekme açıp buraya bakınca
    /// `--process-per-site` bayrağının geçip geçmediği görünüyor
    /// (`docs/Setup.md`).
    pub surec_sayisi: u32,
}

/// Panelin göstereceği kazanç: **ölçülen** + ölçülemeyenlerin tahmini.
///
/// `docs/Bellek.md` iki yol bırakmıştı ve bu fonksiyon ikisinin sınırı.
/// Eşleme çalışan sekmeler için gerçek rakam (`esleme::Tur::kazanc_mb`)
/// kullanılıyor; uyanıkken hiç ölçülmemiş pasif sekmeler için — oturumdan
/// `Atilmis` doğan, bir kez bile webview almamış olanlar — eski tahmine
/// düşülüyor.
///
/// İkisini toplamak, "ya hep ya hiç" davranmaktan iyi: tek bir ölçülmemiş
/// sekme yüzünden otuz ölçülmüş sekmenin gerçek rakamını atıp tahmine dönmek,
/// paneli sebepsiz yere kabalaştırırdı. Karışımın kendisi `olcum_yaklasik`
/// ile dürüstçe etiketleniyor.
pub fn kazanc(tur: &esleme::Tur, webview_mb: u64, uyanik: u32) -> u64 {
    tur.kazanc_mb + tahmini_kazanc(webview_mb, uyanik, tur.kazanc_bilinmeyen, 0)
}

/// Uyanık bir sekmenin ortalama maliyetinden yola çıkarak kazancı tahmin eder.
///
/// **Geri çekilme yolu.** Eşleme çalışırken kullanılan yer yalnız
/// [`kazanc`] içindeki "hiç ölçülmemiş sekmeler" payı; eşleme hiç yoksa
/// (eski Runtime, motorsuz derleme) tek yol bu kalıyor.
///
/// Saf ve testli, çünkü bu sayı kullanıcıya gösterilen tek büyük rakam ve
/// yanlış olduğunda projenin bütün iddiası şüpheli hâle geliyor.
///
/// Yöntem: ölçülen WebView2 belleğini uyanık sekme sayısına bölüp sekme başına
/// ortalama çıkarıyoruz, sonra uyuyan + atılmış sekme sayısıyla çarpıyoruz.
/// **Bu bir tahmin** ve `olcum_yaklasik` ile öyle etiketleniyor.
///
/// Uyanık sekme yoksa (hepsi uyuyor) ortalama hesaplanamıyor; o durumda
/// sekme başına muhafazakâr bir taban değer kullanılıyor — sıfır göstermek,
/// tam da kazancın en yüksek olduğu anda "hiç tasarruf yok" demek olurdu.
pub fn tahmini_kazanc(webview_mb: u64, uyanik: u32, uyuyan: u32, atilmis: u32) -> u64 {
    /// Render süreci taban maliyeti `docs/Bellek.md` tablosunda 30–60 MB.
    /// Alt sınır seçiliyor: abartmaktansa eksik göstermek.
    const TABAN_MB: u64 = 30;

    let pasif = (uyuyan + atilmis) as u64;
    if pasif == 0 {
        return 0;
    }
    let ortalama = if uyanik > 0 && webview_mb > 0 {
        (webview_mb / uyanik as u64).max(TABAN_MB)
    } else {
        TABAN_MB
    };
    ortalama * pasif
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn pasif_sekme_yoksa_kazanc_sifir() {
        assert_eq!(tahmini_kazanc(1200, 10, 0, 0), 0);
    }

    #[test]
    fn ortalamadan_hesaplaniyor() {
        // 10 uyanık sekme 1200 MB → sekme başına 120 MB.
        // 20 uyuyan + 5 atılmış = 25 pasif → 3000 MB.
        assert_eq!(tahmini_kazanc(1200, 10, 20, 5), 3000);
    }

    #[test]
    fn hepsi_uyuyorken_taban_deger_kullaniliyor() {
        // Kazancın en yüksek olduğu an: sıfır göstermek yanlış olurdu.
        assert_eq!(tahmini_kazanc(0, 0, 40, 10), 30 * 50);
    }

    #[test]
    fn kazanc_olculen_ve_tahmini_birlestiriyor() {
        // Otuz sekmenin yirmi dokuzu ölçüldü, biri hiç webview almadı.
        // Ölçülenler gerçek rakamıyla, kalan biri tabanla giriyor.
        let tur = esleme::Tur {
            kazanc_mb: 2400,
            kazanc_bilinmeyen: 1,
            ..Default::default()
        };
        // Uyanık sekme başına 120 MB ölçülmüş → bilinmeyen için 120 MB.
        assert_eq!(kazanc(&tur, 1200, 10), 2400 + 120);
    }

    #[test]
    fn olculmeyen_yoksa_kazanc_tamamen_olculen() {
        let tur = esleme::Tur {
            kazanc_mb: 2400,
            kazanc_bilinmeyen: 0,
            ..Default::default()
        };
        assert_eq!(kazanc(&tur, 1200, 10), 2400);
    }

    #[test]
    fn esleme_hic_yokken_eski_tahmine_dusuluyor() {
        // Eski Runtime ya da motorsuz derleme: `esleme` boş dönüyor ve
        // bütün pasif sekmeler "bilinmeyen".
        let tur = esleme::Tur {
            kazanc_mb: 0,
            kazanc_bilinmeyen: 25,
            ..Default::default()
        };
        assert_eq!(kazanc(&tur, 1200, 10), 3000);
    }

    #[test]
    fn ortalama_tabanin_altina_inmiyor() {
        // Ölçüm bir sebeple çok düşük geldiyse (süreçler henüz doğmamış)
        // sekme başına 2 MB gibi bir sayı kullanıcıya saçma bir kazanç
        // gösterirdi.
        assert_eq!(tahmini_kazanc(20, 10, 10, 0), 30 * 10);
    }
}
