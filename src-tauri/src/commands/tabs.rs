//! Sekme komutları. Gövdeler tek satır; karar `tabs/surucu.rs` içinde.

use std::sync::Arc;

use tauri::State;

use crate::hata::Sonuc;
use crate::motor::{Dikdortgen, Yetenekler};
use crate::tabs::surucu::Surucu;
use crate::tabs::{Grup, GrupId, GrupRengi, SekmeId, SekmeOzeti};

/// Komut katmanı çalışma zamanını GENEL tutmuyor: `State<Arc<Surucu<R>>>`
/// üzerinden `R` çıkarsanamıyor (durum tipe göre aranıyor) ve Tauri makrosu
/// hangi çalışma zamanını arayacağını bilemiyor. Sürücünün kendisi hâlâ
/// genel; sabitlenen yalnız bu ince katman.
type Durum<'a> = State<'a, Arc<Surucu<tauri::Wry>>>;

// Aşağıdaki komutların `async` olması bir üslup tercihi DEĞİL: hepsinin çağrı
// ağacı `Motor::sekme_ac`e, yani `Window::add_child`e ulaşıyor ve eşzamanlı
// bir komuttan webview yaratmak uygulamayı kilitliyor. Gerekçenin tamamı
// `commands` modül notunda.

#[tauri::command]
pub async fn sekme_ac(
    surucu: Durum<'_>,
    url: Option<String>,
    ebeveyn: Option<SekmeId>,
    arkaplanda: bool,
    gizli: bool,
) -> Sonuc<SekmeId> {
    surucu.inner().sekme_ac(url, ebeveyn, arkaplanda, gizli)
}

#[tauri::command]
pub async fn sekme_kapat(surucu: Durum<'_>, id: SekmeId) -> Sonuc<Option<SekmeId>> {
    surucu.inner().sekme_kapat(id)
}

#[tauri::command]
pub async fn sekme_etkinlestir(surucu: Durum<'_>, id: SekmeId) -> Sonuc<()> {
    surucu.inner().sekme_etkinlestir(id)
}

#[tauri::command]
pub fn sekme_tasi(surucu: Durum<'_>, id: SekmeId, hedef: usize) -> Sonuc<()> {
    surucu.inner().sekme_tasi(id, hedef)
}

#[tauri::command]
pub fn sekme_sabitle(surucu: Durum<'_>, id: SekmeId, sabit: bool) -> Sonuc<()> {
    surucu.inner().sekme_sabitle(id, sabit)
}

#[tauri::command]
pub fn sekme_sessize_al(surucu: Durum<'_>, id: SekmeId, sessiz: bool) -> Sonuc<()> {
    surucu.inner().sekme_sessize_al(id, sessiz)
}

#[tauri::command]
pub fn sekme_uyutma_istisnasi(surucu: Durum<'_>, id: SekmeId, istisna: bool) -> Sonuc<()> {
    surucu.inner().sekme_uyutma_istisnasi(id, istisna)
}

/// Uyuyan sekmeyi **uyandırmıyor** (CLAUDE.md #5): bütün alanlar bellekteki
/// kayıttan geliyor.
#[tauri::command]
pub fn sekme_listesi(surucu: Durum<'_>) -> Vec<SekmeOzeti> {
    surucu.inner().sekme_listesi()
}

#[tauri::command]
pub async fn sekme_geri_al(surucu: Durum<'_>) -> Sonuc<Option<SekmeId>> {
    surucu.inner().sekme_geri_al()
}

#[tauri::command]
pub fn sekme_uyut(surucu: Durum<'_>, id: SekmeId) -> Sonuc<()> {
    surucu.inner().sekme_uyut(id)
}

#[tauri::command]
pub fn sekme_at(surucu: Durum<'_>, id: SekmeId) -> Sonuc<()> {
    surucu.inner().sekme_at(id)
}

/// Kabuğun altında kalan alanı bildiriyor.
///
/// Ölçü arayüzde biliniyor (kabuk yüksekliği, yan panel, tam ekran); backend
/// bunu sabit tutsaydı panel açıldığında sayfa panelin altında kalırdı.
#[tauri::command]
pub fn icerik_alani(surucu: Durum<'_>, alan: Dikdortgen) -> Sonuc<()> {
    surucu.inner().icerik_alani(alan)
}

/// Motorun gerçekten neyi desteklediği.
///
/// Arayüz buna bakıp yapamayacağı işin düğmesini çizmiyor — sessizce
/// tıklanan ölü düğme bırakmamak için.
#[tauri::command]
pub fn yetenekler(surucu: Durum<'_>) -> Yetenekler {
    surucu.inner().yetenekler()
}

/// Kısayolu uygular. Dönüş: tabloda karşılığı var mıydı.
///
/// Kabuk odaktayken bu kapıdan, sayfa odaktayken motorun hızlandırıcı
/// kaydından giriliyor — **tablo tek yerde** (`crate::kisayol`,
/// `docs/IPC.md`).
///
/// Dönüş `bool` değil `Sonuc<bool>`: komut `async` (Ctrl+T sekme açıyor,
/// yani webview yaratıyor) ve ödünç alınmış argümanı olan bir `async` komut
/// Tauri'de `Result` döndürmek zorunda. Arayüzde fark yok — `Ok` değeri
/// çözülüyor.
#[tauri::command]
pub async fn kisayol_bas(
    surucu: Durum<'_>,
    tus: String,
    ctrl: bool,
    shift: bool,
    alt: bool,
) -> Sonuc<bool> {
    Ok(surucu.inner().kisayol_uygula(&tus, ctrl, shift, alt))
}

// ------------------------------------------------------------------ gruplar

/// Grup listesi (`docs/IPC.md`, "Komutlar — Gruplar").
#[tauri::command]
pub fn grup_listesi(surucu: Durum<'_>) -> Vec<Grup> {
    surucu.grup_listesi()
}

#[tauri::command]
pub fn grup_ac(surucu: Durum<'_>, ad: String) -> GrupId {
    surucu.grup_ac(ad)
}

/// Grubu siler. **Sekmeler kalıyor**, yalnız gruptan çıkıyorlar.
#[tauri::command]
pub fn grup_sil(surucu: Durum<'_>, id: GrupId) {
    surucu.grup_sil(id);
}

/// Sekmeyi gruba alır; `grup` `null` ise gruptan çıkarır.
#[tauri::command]
pub fn grup_ata(surucu: Durum<'_>, sekme: SekmeId, grup: Option<GrupId>) {
    surucu.grup_ata(sekme, grup);
}

/// Grubun alanlarını günceller.
///
/// Her alan `Option`: verilmeyen alan **değişmiyor**. `uykuEsigiSn` iki
/// katmanlı — verilmezse dokunulmuyor, `null` verilirse grup genel ayara
/// dönüyor, sayı verilirse eşik o oluyor.
#[tauri::command]
pub fn grup_guncelle(
    surucu: Durum<'_>,
    id: GrupId,
    ad: Option<String>,
    renk: Option<GrupRengi>,
    katli: Option<bool>,
    uyku_esigi_sn: Option<Option<u64>>,
) {
    surucu.grup_guncelle(id, ad, renk, katli, uyku_esigi_sn);
}

/// Kabuk tam ekran bir örtü açtı/kapattı (ayarlar, sekme arama).
///
/// Sekme webview'i ayrı bir native pencere ve kabuğun **üstünde** duruyor;
/// kabuğun çizdiği bir örtü sayfanın altında kalıyor. CSS `z-index` iki ayrı
/// pencerenin sırasını değiştiremiyor, o yüzden sayfa gizleniyor
/// (`tabs::surucu::Surucu::ortu_gorunur`).
#[tauri::command]
pub fn ortu_gorunur(surucu: Durum<'_>, acik: bool) -> Sonuc<()> {
    surucu.ortu_gorunur(acik)
}
