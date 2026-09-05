//! Gezinme komutları.

use std::sync::Arc;

use tauri::State;

use crate::hata::Sonuc;
use crate::tabs::surucu::Surucu;
use crate::tabs::SekmeId;

/// Komut katmanı çalışma zamanını GENEL tutmuyor: `State<Arc<Surucu<R>>>`
/// üzerinden `R` çıkarsanamıyor (durum tipe göre aranıyor) ve Tauri makrosu
/// hangi çalışma zamanını arayacağını bilemiyor. Sürücünün kendisi hâlâ
/// genel; sabitlenen yalnız bu ince katman.
type Durum<'a> = State<'a, Arc<Surucu<tauri::Wry>>>;

/// `girdi` ham kullanıcı metni.
///
/// URL mi arama mı ayrımı arayüzde yapılıyor (`src/lib/url.ts`) çünkü kullanıcı
/// yazarken canlı geri bildirim gerekiyor; backend gelen dizeyi yine de
/// doğruluyor (`docs/IPC.md`).
/// `async`: boş bir sekmede gezinmek webview yaratıyor ve eşzamanlı bir
/// komuttan yaratmak uygulamayı kilitliyor (`commands` modül notu).
#[tauri::command]
pub async fn gezin(surucu: Durum<'_>, id: SekmeId, girdi: String) -> Sonuc<()> {
    surucu.inner().gezin(id, girdi)
}

#[tauri::command]
pub fn geri(surucu: Durum<'_>, id: SekmeId) -> Sonuc<()> {
    surucu.inner().geri(id)
}

#[tauri::command]
pub fn ileri(surucu: Durum<'_>, id: SekmeId) -> Sonuc<()> {
    surucu.inner().ileri(id)
}

#[tauri::command]
pub fn yenile(surucu: Durum<'_>, id: SekmeId) -> Sonuc<()> {
    surucu.inner().yenile(id)
}

#[tauri::command]
pub fn durdur(surucu: Durum<'_>, id: SekmeId) -> Sonuc<()> {
    surucu.inner().durdur(id)
}
