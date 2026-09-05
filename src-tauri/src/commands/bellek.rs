//! Bellek komutları — **ince sarmalayıcılar** (CLAUDE.md #1).
//!
//! Karar yok: eşik `memory/esik.rs` içinde, uygulama `memory/gozcu.rs`
//! içinde. Buradaki her gövde tek satır, çünkü aynı işlere gözcünün
//! döngüsünden de geliniyor ve iki kopya sessizce ayrışır.

use std::sync::Arc;

use crate::bridge::muifly::Kaynak;
use crate::hata::Sonuc;
use crate::memory::gecikme::GecikmeOzeti;
use crate::memory::{gozcu, BellekOzeti};
use crate::tabs::surucu::Surucu;

type Durum<'a> = tauri::State<'a, Arc<Surucu<tauri::Wry>>>;

/// Son bellek özeti.
///
/// Gözcü henüz ilk turunu koşmadıysa `None`; arayüz o zaman ilk
/// `muiren://bellek-ozeti` olayını bekliyor. Komut kendi ölçümünü yapsaydı
/// panel her açılışta süreç tablosunu tarardı.
#[tauri::command]
pub fn bellek_ozeti(surucu: Durum<'_>) -> Option<BellekOzeti> {
    surucu.bellek_ozeti()
}

/// Uyanma gecikmesi dağılımı (`docs/Bellek.md` kabul tablosunun iki satırı).
///
/// `bellek_ozeti` ile birleştirilmedi: o özet gözcünün turuna bağlı ve
/// olayla da yayınlanıyor, bu ise kullanıcının sekme değiştirmesiyle
/// değişiyor. Tek yapıda olsalardı panel, gecikme tazelensin diye gözcü
/// turunu beklerdi.
#[tauri::command]
pub fn gecikme_ozeti(surucu: Durum<'_>) -> GecikmeOzeti {
    surucu.gecikme_ozeti()
}

/// Ölçüm oturumuna temiz başlamak için defteri boşaltır.
#[tauri::command]
pub fn gecikme_sifirla(surucu: Durum<'_>) {
    surucu.gecikme_sifirla()
}

/// Korumalı olmayan her uyanık sekmeyi uyutur. Dönüş: uyutulan sayısı.
#[tauri::command]
pub fn hepsini_uyut(surucu: Durum<'_>) -> u32 {
    gozcu::hepsini_uyut(surucu.inner())
}

/// Oyun modunu elle açar/kapatır.
///
/// `Kaynak::Elle`: kullanıcı düğmeye bastı. `Kaynak::Algilama` yalnız
/// `bridge::muifly` izleme döngüsünden geliyor ve arayüz ikisini ayırt
/// ediyor (`docs/IPC.md`, `muiren://oyun-modu`).
#[tauri::command]
pub fn oyun_modu(surucu: Durum<'_>, acik: bool) -> Sonuc<()> {
    gozcu::oyun_modu(surucu.inner(), acik, Kaynak::Elle);
    Ok(())
}
