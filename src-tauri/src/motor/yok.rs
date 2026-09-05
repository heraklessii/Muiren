//! Motorsuz derlemenin karşılığı — `cargo test --no-default-features`.
//!
//! Yüzeyi [`gercek`](super) ile **birebir aynı** olmak zorunda; [`Motor`]
//! trait'i bunu derleyiciye zorlatıyor (CLAUDE.md #4).
//!
//! İki işe yarıyor:
//!
//! - Windows'suz makinede ve CI'da derleniyor/koşuyor.
//! - `tabs/`, `settings/` modüllerinin motora **bağımlı olmadığını** kanıtlıyor.
//!   Biri `tabs/agac.rs` içine bir Win32 çağrısı yazmaya kalkarsa motorsuz
//!   derleme kırılıyor.
//!
//! Her çağrı `Ok(())` dönüp hiçbir şey yapmıyor — hata dönmüyor, çünkü
//! `tabs/surucu.rs` testleri bu motorla uçtan uca koşuyor: sekme sırası,
//! durum geçişi ve oturum yazımı motorsuz da doğrulanabilmeli. Arayüz bu
//! derlemede olduğunu [`Yetenekler::motor`] alanından anlıyor ve sayfa
//! açan denetimleri çizmiyor — sessizce tıklanan ölü düğme bırakmamak için.

use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Mutex;

use tauri::{AppHandle, Runtime};

use super::{BellekSeviyesi, Dikdortgen, Dinleyici, Motor, SurecKaydi, Yetenekler};
use crate::hata::Sonuc;
use crate::tabs::SekmeId;

/// Motorsuz derlemede Chromium bayrağı geçirilmiyor.
///
/// Boş dize `additional_browser_args`a **verilmiyor**: verilseydi wry'nin kendi
/// varsayılanları da silinirdi. Çağıran (`lib.rs`) boş dizeyi "dokunma" diye
/// okuyor.
pub fn ek_bayraklar(_ayarlar: &crate::settings::Settings) -> String {
    String::new()
}

pub struct YokMotor<R: Runtime> {
    /// Gerçek motor bunları webview olarak tutuyor; burada yalnız hangi
    /// sekmenin "webview'i var" sayıldığı tutuluyor ki `var_mi` anlamlı olsun.
    acik: Mutex<HashSet<SekmeId>>,
    // `fn() -> R` çünkü `PhantomData<R>` motoru `R`nin Send/Sync olmasına
    // bağlardı; Tauri `Runtime` bunları vaat etmiyor ve sürücü Tauri
    // durumunda tutulabilmek için Send + Sync olmak zorunda.
    _r: PhantomData<fn() -> R>,
}

impl<R: Runtime> YokMotor<R> {
    /// `_bayraklar` motorsuz derlemede kullanılmıyor ama yüzey `gercek.rs`
    /// ile birebir aynı kalıyor (CLAUDE.md #4): `lib.rs` iki derlemede de
    /// aynı satırı yazıyor.
    pub fn yeni(_app: AppHandle<R>, _bayraklar: String) -> Self {
        YokMotor {
            acik: Mutex::new(HashSet::new()),
            _r: PhantomData,
        }
    }
}

impl<R: Runtime> Motor<R> for YokMotor<R> {
    fn yetenekler(&self) -> Yetenekler {
        Yetenekler::default() // motor: false — hepsi kapalı
    }

    fn dinleyici_ata(&self, _dinleyici: std::sync::Weak<dyn Dinleyici>) {
        // Olay üretmeyen motorun dinleyiciye ihtiyacı yok. Yüzey yine de
        // birebir aynı (CLAUDE.md #4).
    }

    fn sekme_ac(&self, id: SekmeId, _url: &str, _gizli: bool) -> Sonuc<()> {
        self.acik.lock().unwrap().insert(id);
        Ok(())
    }

    fn sekme_kapat(&self, id: SekmeId) -> Sonuc<()> {
        self.acik.lock().unwrap().remove(&id);
        Ok(())
    }

    fn gezin(&self, _id: SekmeId, _url: &str) -> Sonuc<()> {
        Ok(())
    }

    fn geri(&self, _id: SekmeId) -> Sonuc<()> {
        Ok(())
    }

    fn ileri(&self, _id: SekmeId) -> Sonuc<()> {
        Ok(())
    }

    fn yenile(&self, _id: SekmeId) -> Sonuc<()> {
        Ok(())
    }

    fn durdur(&self, _id: SekmeId) -> Sonuc<()> {
        Ok(())
    }

    fn alan_ayarla(&self, _dikdortgen: Dikdortgen) -> Sonuc<()> {
        Ok(())
    }

    fn goster(&self, _id: SekmeId, _dikdortgen: Dikdortgen) -> Sonuc<()> {
        Ok(())
    }

    fn gizle(&self, _id: SekmeId) -> Sonuc<()> {
        Ok(())
    }

    fn kabuk_odakla(&self) -> Sonuc<()> {
        Ok(())
    }

    fn ses_kes(&self, _id: SekmeId, _sessiz: bool) -> Sonuc<()> {
        Ok(())
    }

    fn uyut(&self, _id: SekmeId) -> Sonuc<()> {
        Ok(())
    }

    fn uyandir(&self, _id: SekmeId) -> Sonuc<()> {
        Ok(())
    }

    fn bellek_hedefi(&self, _id: SekmeId, _seviye: BellekSeviyesi) -> Sonuc<()> {
        Ok(())
    }

    fn at(&self, id: SekmeId) -> Sonuc<()> {
        self.acik.lock().unwrap().remove(&id);
        Ok(())
    }

    fn var_mi(&self, id: SekmeId) -> bool {
        self.acik.lock().unwrap().contains(&id)
    }

    fn kaydirma_iste(&self, _id: SekmeId) -> Sonuc<()> {
        // Sayfa yok, kaydırma da yok. Dinleyiciye hiç cevap gelmiyor ve
        // sürücü son bilinen oranı kullanmaya devam ediyor.
        Ok(())
    }

    fn kaydirma_uygula(&self, _id: SekmeId, _oran: f32) -> Sonuc<()> {
        Ok(())
    }

    fn veri_temizle(&self, _kalemler: super::MotorTemizligi) -> Sonuc<()> {
        // Motor yok, profil yok. Sürücü kendi depolarını (geçmiş, favicon)
        // yine de temizliyor — `veri_temizle` testleri bu derlemede de
        // anlamlı kalsın diye.
        Ok(())
    }

    fn surec_bilgisi_iste(&self) -> Sonuc<()> {
        Ok(())
    }

    fn surec_haritasi(&self) -> Vec<SurecKaydi> {
        // Süreç yok, eşleme de yok. **Boş liste** dönüyor ve `esleme.rs`
        // bunu "eşleme tam değil" diye okuyor; uydurulmuş bir kayıt dönmek,
        // motorsuz derlemede paneli dolu gösterip hesabı test edilemez
        // kılardı.
        Vec::new()
    }

    fn calisma_zamani_surumu(&self) -> Option<String> {
        // Çalışma zamanı yok. Rapor bunu `?` ile işaretliyor; uydurma bir
        // sürüm yazmak, ölçümü taşınamaz yapardı.
        None
    }
}
