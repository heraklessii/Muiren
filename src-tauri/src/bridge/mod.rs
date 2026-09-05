//! Köprüler — Mui ailesine devir.
//!
//! `docs/Kopruler.md` bu modülün sözleşmesi. Dört kural oradan geliyor ve
//! burada kodla karşılanıyor:
//!
//! 1. **Köprü isteğe bağlı.** Kardeş kurulu değilse arayüz o menü öğesini hiç
//!    çizmiyor ([`KopruDurumu::kurulu`]). Hata verilmiyor, indirme önerisi
//!    zorlanmıyor.
//! 2. **Gönderilen şey her zaman bir URL ya da bir dosya yolu.** Çerez, oturum
//!    başlığı, kimlik bilgisi ve sayfa içeriği hiçbir köprüden geçmiyor —
//!    [`Yuk`] enum'ı bunu tip düzeyinde zorluyor: taşıyabileceği başka bir şey
//!    yok.
//! 3. **Sayfadan gelen ad güvenilmez.** İndirme dosya adı `Content-Disposition`
//!    başlığından, yani saldırganın yazdığı bir dizeden geliyor;
//!    [`dosya_adi_temizle`] geçmeden hiçbir köprüye verilmiyor (CLAUDE.md #8).
//! 4. **Devretme hatası sessiz kalmıyor** ama tarayıcıyı da durdurmuyor:
//!    çağıran `Result` alıyor ve arayüz küçük bir bildirim gösteriyor.
//!
//! ## Neden kurulum araması önbellekli
//!
//! `kurulu()` her sağ tıkta çağrılabiliyor. Kayıt defteri + dosya sistemi
//! taramasını her seferinde yapmak, bağlam menüsünün açılmasını diske
//! bağlamak demek. Sonuç [`Bulunan`] içinde bir kez saklanıyor;
//! [`Kopruler::tazele`] kullanıcı ayarlardan istediğinde yeniden arıyor.

pub mod muifly;
pub mod muiget;
pub mod muiply;
pub mod muiwatch;
pub mod oyun;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

use crate::hata::Sonuc;

/// Bir köprüye devredilebilecek **her şey**.
///
/// Liste bilinçli olarak kısa: köprüden geçebilecek veri türlerini tip
/// düzeyinde sınırlamak, "bir de şu başlığı gönderelim" değişikliğini
/// derleyicinin önüne getiriyor (`docs/Kopruler.md`, "Genel kural").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Yuk {
    /// Muiget'e indirme devri. `dosya_adi` **sayfadan geliyor** ve
    /// [`dosya_adi_temizle`] geçmiş olmak zorunda. `kaynak_sayfa` indirmenin
    /// başladığı sekmenin adresi — hotlink korumalı siteler `Referer`
    /// olmadan indirme vermiyor (gerekçe [`muiget`] başlığında).
    Indirme {
        url: String,
        dosya_adi: Option<String>,
        kaynak_sayfa: Option<String>,
    },
    /// Muiply'a yerel dosya devri.
    DosyaYolu(PathBuf),
    /// Muiwatch oda kimliği. Serbest metin değil: [`muiwatch::oda_gecerli`]
    /// süzgecinden geçiyor.
    Oda(String),
}

/// Dört köprünün ortak yüzeyi (`docs/Kopruler.md`, "Köprü modülünün ortak
/// şekli").
///
/// Test edilebilirlik için trait: `kurulu()` sahte döndüren bir köprüyle
/// arayüzün davranışı, gerçek bir kurulum olmadan doğrulanabiliyor.
pub trait Kopru: Send + Sync {
    fn ad(&self) -> &'static str;
    /// Önbellekli. İlk çağrıda arıyor, sonrakiler kayıttan okuyor.
    fn kurulu(&self) -> bool;
    fn devret(&self, yuk: Yuk) -> Sonuc<()>;
}

/// Arayüze giden köprü durumu (`docs/IPC.md`, `kopru_durumu`).
///
/// `yol` **gösterilmiyor**, teşhis için: kullanıcı "Muiget kurulu ama köprü
/// görmüyor" dediğinde bakılacak ilk yer burası.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KopruDurumu {
    pub ad: &'static str,
    pub kurulu: bool,
    pub yol: Option<String>,
}

// ---------------------------------------------------------------- bulucu

/// Bir kardeş uygulamanın nerede olduğunun **önbellekli** cevabı.
///
/// `OnceLock` değil `Mutex<Option<..>>`: kullanıcı Muiget'i Muiren açıkken
/// kurabiliyor ve o durumda köprünün yeniden aranabilmesi gerekiyor
/// ([`Kopruler::tazele`]).
#[derive(Debug, Default)]
pub struct Bulunan {
    kayit: Mutex<Option<Option<PathBuf>>>,
}

impl Bulunan {
    pub fn yeni() -> Self {
        Bulunan::default()
    }

    /// Aranmış sonucu döndürür; aranmadıysa `ara` ile arar.
    pub fn yol<F>(&self, ara: F) -> Option<PathBuf>
    where
        F: FnOnce() -> Option<PathBuf>,
    {
        let mut kayit = self.kayit.lock().unwrap();
        if let Some(sonuc) = kayit.as_ref() {
            return sonuc.clone();
        }
        let sonuc = ara();
        *kayit = Some(sonuc.clone());
        sonuc
    }

    /// Önbelleği düşürür; sıradaki [`Bulunan::yol`] yeniden arıyor.
    pub fn unut(&self) {
        *self.kayit.lock().unwrap() = None;
    }
}

/// Kardeş uygulamanın çalıştırılabilir dosyasını arar.
///
/// Sıra `docs/Kopruler.md` ile aynı: **kayıt defteri kurulum anahtarı → bilinen
/// kurulum yolları → kurulu değil.** Kayıt defteri önce çünkü kullanıcı
/// uygulamayı başka bir sürücüye kurmuş olabiliyor; bilinen yollar yalnız
/// taşınabilir (kurulumsuz) kopyaları yakalıyor.
///
/// `urun` kurulum anahtarındaki ürün adı (Tauri/NSIS `DisplayName`), `ikili`
/// çalıştırılabilirin adı (`muiget.exe`).
pub fn uygulama_ara(urun: &str, ikili: &str) -> Option<PathBuf> {
    if let Some(yol) = kayittan_ara(urun, ikili) {
        return Some(yol);
    }
    bilinen_yollardan_ara(urun, ikili)
}

/// Kurulum anahtarından. Windows dışında hep `None` — köprü sessizce
/// kapanıyor, derleme kırılmıyor.
#[cfg(windows)]
fn kayittan_ara(urun: &str, ikili: &str) -> Option<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    const KALDIRMA: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

    for kok in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let Ok(kaldirma) = RegKey::predef(kok).open_subkey_with_flags(KALDIRMA, KEY_READ) else {
            continue;
        };
        for anahtar_adi in kaldirma.enum_keys().flatten() {
            let Ok(anahtar) = kaldirma.open_subkey_with_flags(&anahtar_adi, KEY_READ) else {
                continue;
            };
            let ad: String = anahtar.get_value("DisplayName").unwrap_or_default();
            if !ad.eq_ignore_ascii_case(urun) {
                continue;
            }
            // `InstallLocation` NSIS ve MSI'da dizin; `DisplayIcon` bazen
            // doğrudan ikiliyi gösteriyor. İkisi de deneniyor: hangisinin
            // dolu olduğu kurulum aracına göre değişiyor.
            let dizin: String = anahtar.get_value("InstallLocation").unwrap_or_default();
            if !dizin.is_empty() {
                let aday = Path::new(&dizin).join(ikili);
                if aday.is_file() {
                    return Some(aday);
                }
            }
            let ikon: String = anahtar.get_value("DisplayIcon").unwrap_or_default();
            let ikon = ikon.split(',').next().unwrap_or("").trim_matches('"');
            if !ikon.is_empty() {
                let aday = PathBuf::from(ikon);
                if aday.is_file()
                    && aday
                        .file_name()
                        .is_some_and(|a| a.eq_ignore_ascii_case(ikili))
                {
                    return Some(aday);
                }
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn kayittan_ara(_urun: &str, _ikili: &str) -> Option<PathBuf> {
    None
}

/// Kurulum aracının varsayılan hedefleri ve Muiren'in kendi yanı.
///
/// Sonuncusu portföy için önemli: geliştirme sırasında kardeş uygulamalar
/// kurulmuyor, yan yana duruyor.
fn bilinen_yollardan_ara(urun: &str, ikili: &str) -> Option<PathBuf> {
    let mut adaylar: Vec<PathBuf> = Vec::new();

    for degisken in ["LOCALAPPDATA", "PROGRAMFILES", "ProgramFiles(x86)"] {
        if let Ok(kok) = std::env::var(degisken) {
            adaylar.push(Path::new(&kok).join(urun).join(ikili));
            adaylar.push(Path::new(&kok).join("Programs").join(urun).join(ikili));
        }
    }

    // Muiren'in kendi klasörünün kardeşi. `..\Muiget\muiget.exe` —
    // geliştirme düzeni (`CLAUDE.md`, "Diğer Mui Projeleriyle Tutarlılık").
    if let Ok(kendi) = std::env::current_exe() {
        if let Some(dizin) = kendi.parent() {
            adaylar.push(dizin.join(ikili));
            if let Some(ust) = dizin.parent() {
                adaylar.push(ust.join(urun).join(ikili));
            }
        }
    }

    adaylar.into_iter().find(|a| a.is_file())
}

// ------------------------------------------------------------ dosya adı

/// Dosya adının üst sınırı. Windows'ta bileşen sınırı 255; başına indirme
/// klasörü ve sonuna `.crdownload` benzeri bir ek geldiğinde sınır aşılabiliyor.
const AD_SINIRI: usize = 180;

/// Sayfadan gelen dosya adını güvenli hâle getirir.
///
/// **Bu fonksiyon bir güvenlik sınırı.** `Content-Disposition` başlığını
/// saldırgan yazıyor ve devrettiğimiz ad karşı tarafta bir dosya yoluna
/// dönüşüyor. Temizlenmezse `..\..\Startup\kotu.exe` gibi bir ad Muiget'e
/// gidiyor ve sorumluluk bizde kalıyor.
///
/// Yaklaşım **beyaz liste değil daraltma**: yol ayırıcılar ve sürücü harfi
/// düşürülüyor, yalnız son bileşen kalıyor, kontrol ve iki yönlü metin
/// karakterleri atılıyor (`sahte.txt<U+202E>gpj.exe` numarası), Windows'un
/// ayrılmış adları (`CON`, `PRN`, `NUL`, `COM1`…) önek alıyor ve ad
/// kırpılıyor. Sonuç boşsa `None`: adsız devretmek, uydurma bir ad
/// devretmekten dürüst.
pub fn dosya_adi_temizle(ham: &str) -> Option<String> {
    // Yalnız son bileşen. `\` ve `/` birlikte: karşı taraf Windows'ta çalışsa
    // da URL'den gelen ad `/` taşıyor.
    let son = ham.rsplit(['/', '\\']).next().unwrap_or(ham);
    // Sürücü öneki (`C:sahte.txt`) — ayırıcısız hâli göreli yol demek.
    let son = son.rsplit(':').next().unwrap_or(son);

    let temiz: String = son
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
        // Windows'ta dosya adında yasak olanlar.
        .map(|c| if "<>:\"/\\|?*".contains(c) { '_' } else { c })
        .collect();

    // Yalnız nokta ve boşluktan oluşan adlar (`.`, `..`, `   `) düşüyor.
    let temiz = temiz.trim().trim_matches('.').trim();
    if temiz.is_empty() {
        return None;
    }

    let kirpilmis: String = temiz.chars().take(AD_SINIRI).collect();
    let kirpilmis = kirpilmis.trim_end().to_string();
    if kirpilmis.is_empty() {
        return None;
    }

    // Ayrılmış aygıt adları: `NUL.txt` Windows'ta hâlâ NUL aygıtı.
    const AYRILMIS: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let govde = kirpilmis.split('.').next().unwrap_or("");
    if AYRILMIS.iter().any(|a| govde.eq_ignore_ascii_case(a)) {
        return Some(format!("_{kirpilmis}"));
    }

    Some(kirpilmis)
}

// ------------------------------------------------------------- kayıt defteri

/// Dört köprünün kaydı. Sürücü bunu tutuyor; komutlar buradan geçiyor.
pub struct Kopruler {
    pub muiget: muiget::Muiget,
    pub muiply: muiply::Muiply,
    pub muiwatch: muiwatch::Muiwatch,
    /// `Arc` çünkü oyun algılama döngüsü köprüyü zayıf referansla tutuyor
    /// (`muifly::Muifly::izlemeyi_baslat`): uygulama kapanınca döngü de
    /// bitsin diye.
    pub muifly: std::sync::Arc<muifly::Muifly>,
}

impl Default for Kopruler {
    fn default() -> Self {
        Kopruler::yeni()
    }
}

impl Kopruler {
    pub fn yeni() -> Self {
        Kopruler {
            muiget: muiget::Muiget::yeni(),
            muiply: muiply::Muiply::yeni(),
            muiwatch: muiwatch::Muiwatch::yeni(),
            muifly: std::sync::Arc::new(muifly::Muifly::yeni()),
        }
    }

    fn hepsi(&self) -> [&dyn Kopru; 4] {
        [
            &self.muiget,
            &self.muiply,
            &self.muiwatch,
            self.muifly.as_ref(),
        ]
    }

    /// Arayüze giden liste (`docs/IPC.md`, `kopru_durumu`).
    pub fn durumlar(&self) -> Vec<KopruDurumu> {
        vec![
            self.muiget.durum(),
            self.muiply.durum(),
            self.muiwatch.durum(),
            self.muifly.durum(),
        ]
    }

    /// Önbellekleri düşürür: kullanıcı Muiren açıkken bir kardeş kurmuş
    /// olabiliyor.
    pub fn tazele(&self) {
        self.muiget.unut();
        self.muiply.unut();
        self.muiwatch.unut();
        self.muifly.unut();
    }

    /// Kurulu köprü var mı. Arayüzün köprü bölümünü hiç çizmemesi için.
    pub fn herhangi_kurulu(&self) -> bool {
        self.hepsi().iter().any(|k| k.kurulu())
    }
}

/// Süreç başlatmanın tek yeri.
///
/// Tek yerde olması bir güvenlik tercihi: kabuk **kullanılmıyor**
/// (`cmd /c` yok), argümanlar diziyle geçiyor, dolayısıyla dosya adındaki bir
/// `&` ya da `|` komut olarak yorumlanamıyor. Kabuk üzerinden çalıştırmak,
/// [`dosya_adi_temizle`] ile kurulan sınırı boşa çıkarırdı.
pub fn calistir(ikili: &Path, argumanlar: &[&str]) -> Sonuc<()> {
    use std::process::Command;

    let mut komut = Command::new(ikili);
    komut.args(argumanlar);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: kardeş uygulama bir konsol penceresi
        // parlatmasın.
        komut.creation_flags(0x0800_0000);
    }
    komut
        .spawn()
        .map(|_| ())
        .map_err(crate::hata::MuirenHata::from)
}

// Bu modülün veri dizinine ihtiyacı **yok** ve bir kanal da açılmıyor:
// köprüler yalnız bir ikili çalıştırıyor ya da stdio üzerinden konuşuyor.
// Bir zamanlar buraya bir `OnceLock<PathBuf>` konmuştu; kullanılmayan bir
// API, modülün neye eriştiği hakkında yanlış bir söz veriyor.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn dizin_tirmanmasi_dusuyor() {
        assert_eq!(
            dosya_adi_temizle(r"..\..\Startup\kotu.exe").as_deref(),
            Some("kotu.exe")
        );
        assert_eq!(
            dosya_adi_temizle("../../etc/passwd").as_deref(),
            Some("passwd")
        );
    }

    #[test]
    fn mutlak_yol_dusuyor() {
        assert_eq!(
            dosya_adi_temizle(r"C:\Windows\System32\kotu.dll").as_deref(),
            Some("kotu.dll")
        );
    }

    #[test]
    fn surucu_oneki_ayiriciyi_atlatamiyor() {
        // `C:sahte.txt` Windows'ta "C sürücüsünün geçerli dizini" demek;
        // ayırıcı olmadığı için yol ayırıcı süzgecine takılmıyor.
        assert_eq!(
            dosya_adi_temizle("C:sahte.txt").as_deref(),
            Some("sahte.txt")
        );
    }

    #[test]
    fn ters_akis_karakteri_atiliyor() {
        // `sahte.txt<U+202E>gpj.exe` sekme çubuğunda ve indirme listesinde
        // `sahte.txtexe.jpg` görünüyor; çalıştırılan yine `.exe`.
        let ad = dosya_adi_temizle("sahte.txt\u{202E}gpj.exe").unwrap();
        assert!(!ad.contains('\u{202E}'), "iki yönlü metin karakteri kaldı");
        assert!(ad.ends_with(".exe"), "gerçek uzantı gizlendi: {ad}");
    }

    #[test]
    fn kontrol_karakterleri_atiliyor() {
        assert_eq!(dosya_adi_temizle("a\nb\tc.txt").as_deref(), Some("abc.txt"));
    }

    #[test]
    fn ayrilmis_aygit_adi_onek_aliyor() {
        assert_eq!(dosya_adi_temizle("NUL.txt").as_deref(), Some("_NUL.txt"));
        assert_eq!(dosya_adi_temizle("com1").as_deref(), Some("_com1"));
    }

    #[test]
    fn bos_ad_none_donuyor() {
        assert_eq!(dosya_adi_temizle("   "), None);
        assert_eq!(dosya_adi_temizle(".."), None);
        assert_eq!(dosya_adi_temizle("/"), None);
    }

    #[test]
    fn uzun_ad_kirpiliyor() {
        let uzun = format!("{}.zip", "a".repeat(500));
        assert_eq!(dosya_adi_temizle(&uzun).unwrap().chars().count(), AD_SINIRI);
    }

    #[test]
    fn turkce_karakterler_korunuyor() {
        assert_eq!(
            dosya_adi_temizle("Şişli Raporu ığüöç.pdf").as_deref(),
            Some("Şişli Raporu ığüöç.pdf")
        );
    }

    #[test]
    fn onbellek_bir_kez_ariyor() {
        let bulunan = Bulunan::yeni();
        let sayac = std::sync::atomic::AtomicU32::new(0);
        let ara = || {
            sayac.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            None::<PathBuf>
        };
        bulunan.yol(ara);
        bulunan.yol(ara);
        assert_eq!(sayac.load(std::sync::atomic::Ordering::SeqCst), 1);

        // `unut` sonrası yeniden aranıyor: kullanıcı Muiren açıkken kardeş
        // uygulamayı kurabiliyor.
        bulunan.unut();
        bulunan.yol(ara);
        assert_eq!(sayac.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
