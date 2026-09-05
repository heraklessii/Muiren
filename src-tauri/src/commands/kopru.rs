//! Köprü ve favicon komutları — **ince sarmalayıcılar** (CLAUDE.md #1).
//!
//! Karar yok: kurulum araması `bridge/mod.rs`, devir kuralları ilgili köprü
//! modülünde, koruma kuralı `memory/esik.rs` içinde. Sözleşme
//! `docs/IPC.md`, "Komutlar — Köprüler".

use std::sync::Arc;

use crate::bridge::{muiply, KopruDurumu};
use crate::hata::{MuirenHata, Sonuc};
use crate::tabs::surucu::Surucu;
use crate::tabs::SekmeId;

type Durum<'a> = tauri::State<'a, Arc<Surucu<tauri::Wry>>>;

/// Hangi kardeş uygulama kurulu.
#[tauri::command]
pub fn kopru_durumu(surucu: Durum<'_>) -> Vec<KopruDurumu> {
    surucu.kopru_durumu()
}

/// Kurulum önbelleklerini düşürüp yeniden arar.
#[tauri::command]
pub fn kopru_tazele(surucu: Durum<'_>) -> Vec<KopruDurumu> {
    surucu.kopru_tazele()
}

/// İndirmeyi Muiget'e devreder.
#[tauri::command]
pub fn muiget_gonder(
    surucu: Durum<'_>,
    url: String,
    dosya_adi: Option<String>,
    kaynak_sayfa: Option<String>,
) -> Sonuc<()> {
    surucu.muiget_gonder(url, dosya_adi, kaynak_sayfa)
}

/// Motorun kendi indirmesine bir kerelik izin verir (`Sor` politikasında
/// "Muiren indirsin").
#[tauri::command]
pub fn indirme_izin_ver(surucu: Durum<'_>, id: SekmeId, url: String) -> Sonuc<()> {
    surucu.indirme_izin_ver(id, url)
}

/// Yerel medya dosyasını Muiply'a devreder.
///
/// `yol` bir dosya yolu; `adres` bir `file://` adresi. Arayüzde ikisi de
/// olabiliyor (bağlam menüsü adresi, sürükle-bırak yolu veriyor), o yüzden
/// ayrım burada değil [`muiply::yerel_dosya`] içinde ve saf.
#[tauri::command]
pub fn muiply_ac(surucu: Durum<'_>, yol: String) -> Sonuc<()> {
    let cozulmus = if yol.starts_with("file:") {
        muiply::yerel_dosya(&yol)
            .ok_or_else(|| MuirenHata::Kopru("yerel medya dosyası değil".into()))?
            .to_string_lossy()
            .into_owned()
    } else {
        yol
    };
    surucu.muiply_ac(cozulmus)
}

/// Sekmeyi bir Muiwatch oturumuna bağlar (koruma kuralı #7).
#[tauri::command]
pub fn muiwatch_bagla(surucu: Durum<'_>, id: SekmeId, oda: String) -> Sonuc<()> {
    surucu.muiwatch_bagla(id, oda)
}

#[tauri::command]
pub fn muiwatch_birak(surucu: Durum<'_>, id: SekmeId) {
    surucu.muiwatch_birak(id);
}

/// Favicon'u `data:image/png;base64,...` olarak verir.
///
/// Arayüz kimliğe göre önbellekliyor: kimlik içeriğin karması, yani aynı
/// kimlik hep aynı baytlar (`favicon/mod.rs`).
#[tauri::command]
pub fn favicon_oku(surucu: Durum<'_>, kimlik: String) -> Option<String> {
    surucu.favicon_oku(&kimlik)
}
