//! Backend → arayüz olay adları.
//!
//! `docs/IPC.md` içindeki tablo ile **birebir aynı**. Ad tek yerde duruyor
//! çünkü bir yerde `sekme-degisti`, başka yerde `sekme_degisti` yazmak sessizce
//! çalışmayan bir dinleyici demek; derleyici bunu yakalamıyor.
//!
//! TypeScript karşılıkları `src/ipc/tipler.ts` içinde ve aynı sırayla.

pub const SEKME_DEGISTI: &str = "muiren://sekme-degisti";
pub const SEKME_GUNCELLENDI: &str = "muiren://sekme-guncellendi";
pub const SEKME_DURUM_DEGISTI: &str = "muiren://sekme-durum-degisti";
pub const BELLEK_OZETI: &str = "muiren://bellek-ozeti";
pub const GEZINME: &str = "muiren://gezinme";
pub const INDIRME_ONERISI: &str = "muiren://indirme-onerisi";
pub const TEMA_DEGISTI: &str = "muiren://tema-degisti";
pub const KOPRU_DURUMU: &str = "muiren://kopru-durumu";
pub const OYUN_MODU: &str = "muiren://oyun-modu";
/// Bir pencere, yönlendirme ya da istek engellendi (`engel/mod.rs`).
///
/// Engellenen şey **sessizce kaybolmuyor**: arayüz adres çubuğunda bir rozet
/// gösteriyor ve kullanıcı "yine de aç" diyebiliyor. Sessiz engelleme,
/// kullanıcının tarayıcıyı bozuk sanması demek.
pub const ENGELLENDI: &str = "muiren://engellendi";
/// Sayfa odaktayken **arayüz işi** bir kısayola basıldı (`docs/IPC.md`,
/// `kisayol_bas`). Kabuk odaktayken bu olaya gerek yok: React zaten kendi
/// dinleyicisinden geçiyor ve eylemi doğrudan uyguluyor.
pub const KISAYOL: &str = "muiren://kisayol";
