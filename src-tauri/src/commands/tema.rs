//! Tema komutları — **ince sarmalayıcılar** (CLAUDE.md #1).
//!
//! Doğrulama burada değil `theme/paket.rs` içinde ve **tek yerde**: ikinci
//! bir süzgeç, biri gevşediğinde fark edilmeyen bir açık demek
//! (`docs/Temalar.md`).

use std::sync::Arc;

use crate::hata::Sonuc;
use crate::tabs::surucu::Surucu;
use crate::theme::TemaOzeti;

type Durum<'a> = tauri::State<'a, Arc<Surucu<tauri::Wry>>>;

/// Yerleşik + kurulu bütün temalar.
#[tauri::command]
pub fn tema_listesi(surucu: Durum<'_>) -> Vec<TemaOzeti> {
    surucu.tema_listesi()
}

/// `.muitema` dosyasını doğrulayıp kurar ve **uygular**.
///
/// Kurulum kopyalıyor: kullanıcı indirdiği dosyayı sonra silebilmeli.
#[tauri::command]
pub fn tema_yukle(surucu: Durum<'_>, dosya_yolu: String) -> Sonuc<TemaOzeti> {
    surucu.tema_yukle(&dosya_yolu)
}

#[tauri::command]
pub fn tema_uygula(surucu: Durum<'_>, ad: String) -> Sonuc<TemaOzeti> {
    surucu.tema_uygula(&ad)
}

#[tauri::command]
pub fn tema_sil(surucu: Durum<'_>, ad: String) -> Sonuc<()> {
    surucu.tema_sil(&ad)
}

#[tauri::command]
pub fn tema_disa_aktar(surucu: Durum<'_>, ad: String, hedef: String) -> Sonuc<()> {
    surucu.tema_disa_aktar(&ad, &hedef)
}

/// Açılışta uygulanacak tema. Arayüz ilk boyamada bunu okuyup jetonları
/// yazıyor.
#[tauri::command]
pub fn tema_etkin(surucu: Durum<'_>) -> Option<TemaOzeti> {
    surucu.tema_etkin()
}

/// Etkin temanın arka plan görseli, `data:` adresi olarak.
///
/// **Neden ayrı bir komut** — görsel megabaytlarca olabiliyor ve
/// `TemaOzeti` her tema listesinde, her `tema-degisti` olayında gidiyor.
/// Görseli oraya koymak, tema listesini açan kullanıcıya üç duvar kâğıdını
/// birden göndermek olurdu. Arayüz bunu yalnız etkin tema için ve bir kez
/// istiyor (`useTema`).
#[tauri::command]
pub fn tema_arkaplan(surucu: Durum<'_>) -> Option<String> {
    surucu
        .tema_etkin()
        .and_then(|t| t.arkaplan)
        .and_then(|a| crate::theme::arkaplan_veri(&a))
}
