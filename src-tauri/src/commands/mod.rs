//! Tauri komutları — **ince sarmalayıcılar**.
//!
//! **Buraya iş mantığı yazılmıyor** (CLAUDE.md #1). Gerekçe soyut değil: aynı
//! karara bellek gözcüsünün döngüsünden de gelinebiliyor ve iki kopya
//! birbirinden sessizce ayrışır. Bir komutun gövdesi tek satırdan uzunsa
//! büyük ihtimalle yanlış yerde.
//!
//! Sözleşme `docs/IPC.md` içinde ve **önce orası** yazılıyor (CLAUDE.md #3).

pub mod ayarlar;
pub mod bellek;
pub mod gecmis;
pub mod gezinme;
pub mod kopru;
pub mod tabs;
pub mod tema;
