//! Geçmiş ve yer imi komutları — **ince sarmalayıcılar** (CLAUDE.md #1).
//!
//! Veritabanı açılamamışsa (`docs/Depolama.md`) komutlar hata değil **boş
//! sonuç** döndürüyor: geçmişi olmayan bir tarayıcı çalışır, her tuşta hata
//! kutusu açan bir tarayıcı çalışmaz.

use std::sync::Arc;

use crate::hata::Sonuc;
use crate::history::{Aralik, GecmisKaydi, YerImi, YerImiKlasor};
use crate::tabs::surucu::Surucu;

type Durum<'a> = tauri::State<'a, Arc<Surucu<tauri::Wry>>>;

#[tauri::command]
pub fn gecmis_ara(surucu: Durum<'_>, sorgu: String, limit: u32) -> Sonuc<Vec<GecmisKaydi>> {
    match surucu.gecmis() {
        Some(d) => d.ara(&sorgu, limit),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub fn gecmis_sil(surucu: Durum<'_>, id: i64) -> Sonuc<()> {
    match surucu.gecmis() {
        Some(d) => d.gecmis_sil(id),
        None => Ok(()),
    }
}

#[tauri::command]
pub fn gecmis_temizle(surucu: Durum<'_>, aralik: Aralik) -> Sonuc<u64> {
    match surucu.gecmis() {
        Some(d) => d.gecmis_temizle(aralik),
        None => Ok(0),
    }
}

#[tauri::command]
pub fn yer_imi_ekle(
    surucu: Durum<'_>,
    url: String,
    baslik: String,
    klasor: Option<i64>,
) -> Sonuc<i64> {
    match surucu.gecmis() {
        Some(d) => d.yer_imi_ekle(&url, &baslik, klasor),
        None => Ok(0),
    }
}

#[tauri::command]
pub fn yer_imi_sil(surucu: Durum<'_>, id: i64) -> Sonuc<()> {
    match surucu.gecmis() {
        Some(d) => d.yer_imi_sil(id),
        None => Ok(()),
    }
}

#[tauri::command]
pub fn yer_imi_listesi(surucu: Durum<'_>, klasor: Option<i64>) -> Sonuc<Vec<YerImi>> {
    match surucu.gecmis() {
        Some(d) => d.yer_imi_listesi(klasor),
        None => Ok(Vec::new()),
    }
}

/// Bu adres yer imlerinde mi. Adres çubuğundaki yıldız bunu okuyor.
#[tauri::command]
pub fn yer_imi_mi(surucu: Durum<'_>, url: String) -> Sonuc<Option<i64>> {
    match surucu.gecmis() {
        Some(d) => d.yer_imi_mi(&url),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn yer_imi_klasor_ekle(surucu: Durum<'_>, ad: String, ebeveyn: Option<i64>) -> Sonuc<i64> {
    match surucu.gecmis() {
        Some(d) => d.klasor_ekle(&ad, ebeveyn),
        None => Ok(0),
    }
}

#[tauri::command]
pub fn yer_imi_klasorleri(surucu: Durum<'_>) -> Sonuc<Vec<YerImiKlasor>> {
    match surucu.gecmis() {
        Some(d) => d.klasor_listesi(),
        None => Ok(Vec::new()),
    }
}
