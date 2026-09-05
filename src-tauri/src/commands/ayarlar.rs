//! Ayar komutları.

use std::sync::Arc;

use tauri::State;

use crate::engel::EngelOzeti;
use crate::hata::Sonuc;
use crate::settings::Settings;
use crate::tabs::surucu::{Surucu, Temizlik, TemizlikRaporu};
use crate::tabs::SekmeId;

/// Komut katmanı çalışma zamanını GENEL tutmuyor: `State<Arc<Surucu<R>>>`
/// üzerinden `R` çıkarsanamıyor (durum tipe göre aranıyor) ve Tauri makrosu
/// hangi çalışma zamanını arayacağını bilemiyor. Sürücünün kendisi hâlâ
/// genel; sabitlenen yalnız bu ince katman.
type Durum<'a> = State<'a, Arc<Surucu<tauri::Wry>>>;

#[tauri::command]
pub fn ayarlar_oku(surucu: Durum<'_>) -> Settings {
    surucu.inner().ayarlar()
}

/// **Düzeltilmiş** ayarları geri döndürüyor.
///
/// Kullanıcı uyku eşiğine 0 yazarsa `duzelt` onu alt sınıra çekiyor ve arayüz
/// düzeltilmiş değeri gösteriyor — sessizce farklı bir değerle çalışmak yok
/// (CLAUDE.md #13).
#[tauri::command]
pub fn ayarlar_yaz(surucu: Durum<'_>, ayarlar: Settings) -> Sonuc<Settings> {
    surucu.inner().ayarlar_yaz(ayarlar)
}

/// Engelleme özeti: kaç kural derlendi, hangi satırlar anlaşılmadı.
///
/// Ayarlar ekranı bunu kural kutusunun altında gösteriyor — anlaşılmayan
/// satırı sessizce atmak, kullanıcının çalışmayan bir listeyle dolaşması
/// demek (`engel/liste.rs`).
#[tauri::command]
pub fn engel_ozeti(surucu: Durum<'_>) -> EngelOzeti {
    surucu.engel_ozeti()
}

/// Bu sekmede, bu sayfada kaç istek engellendi. Adres çubuğundaki rozet.
#[tauri::command]
pub fn engel_sayaci(surucu: Durum<'_>, id: SekmeId) -> u32 {
    surucu.engel_sayaci(id)
}

/// Veri temizleme (`docs/IPC.md`).
///
/// Hem motorun profilini hem Muiren'in kendi depolarını kapsıyor; ayrım
/// kullanıcıya görünmüyor (`tabs::surucu::Surucu::veri_temizle`).
#[tauri::command]
pub fn veri_temizle(surucu: Durum<'_>, istek: Temizlik) -> Sonuc<TemizlikRaporu> {
    surucu.veri_temizle(istek)
}
