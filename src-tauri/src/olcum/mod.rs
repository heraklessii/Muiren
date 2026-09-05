//! Ölçüm modu — `docs/olcumler/README.md` protokolünün otomatik yarısı.
//!
//! ## Neden var
//!
//! `docs/Roadmap.md` üç fazın da aynı cümleyle açık kaldığını söylüyordu:
//! *"eksik olan araç değil, makine başında geçirilecek zaman."* Protokolün
//! altı satırından dördü elli sekmeyi açıp beklemeyi ve doğru anlarda panele
//! bakmayı gerektiriyor. Bu, insanın en kötü olduğu iş: 15 dakika sonra doğru
//! saniyede ekrana bakmak ve sayıyı elle kopyalamak.
//!
//! Bu modül o işi koşturuyor: sabit bir listeyi açıyor, protokolün istediği
//! anlarda ölçüyor ve raporu `docs/olcumler/` iskeletinde diske yazıyor.
//!
//! ## Neden ayrı bir mod, komut değil
//!
//! Ölçüm **normal kullanımda yok**: `MUIREN_OLCUM` ortam değişkeni
//! tanımlanmadıkça bu modülün tek satırı çalışmıyor, arayüzde bir düğmesi
//! yok, IPC yüzeyine bir komut eklemiyor. Bir ölçüm aracının kullanıcıya
//! sunulan bir özellik hâline gelmesi, elli sekme açan bir düğmenin yanlışlıkla
//! tıklanabilmesi demekti.
//!
//! ## Sınırı — dürüstçe
//!
//! Bu mod **Chrome'u ölçmüyor** ve ölçemez; kabul tablosunun üç satırı
//! karşılaştırmalı ve o sütun elle doluyor. Uyanma gecikmelerini de ölçmüyor:
//! hiçbir sekmeye tıklamıyor, dolayısıyla bir uyanma da üretmiyor. Rapor iki
//! sınırı da kendi içinde yazıyor — üretilen dosyanın eksiğini söylemesi,
//! okuyanın eksiği bulmasından iyi.
//!
//! ## Katmanlar
//!
//! ```text
//!   plan.rs    "hangi adresler, hangi sürelerle"   → SAF, testli
//!   rapor.rs   "sayılar → Markdown"                → SAF, testli
//!   kosucu.rs  "aç, bekle, ölç, yaz"               → iş parçacığı
//! ```

pub mod kosucu;
pub mod plan;
pub mod rapor;

/// Ölçüm planının yolunu taşıyan ortam değişkeni.
pub const ORTAM_DEGISKENI: &str = "MUIREN_OLCUM";

/// Kullanıcı ölçüm istedi mi.
///
/// Boş bir değer "istemedi" sayılıyor: `set MUIREN_OLCUM=` yazıp kapatmaya
/// çalışan biri elli sekmelik bir koşuyla karşılaşmasın.
pub fn istenen_plan() -> Option<std::path::PathBuf> {
    let ham = std::env::var(ORTAM_DEGISKENI).ok()?;
    let kirpik = ham.trim();
    if kirpik.is_empty() {
        return None;
    }
    Some(std::path::PathBuf::from(kirpik))
}
