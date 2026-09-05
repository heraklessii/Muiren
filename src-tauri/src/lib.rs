//! Muiren — çok sekmede az RAM harcayan masaüstü web tarayıcısı.
//!
//! Bu dosyanın işi kurulum: pencereyi ve kabuk webview'ini yaratmak, motoru ve
//! sürücüyü bağlamak, komutları kaydetmek. Karar veren kod modüllerde
//! (`docs/Architecture.md`).
//!
//! ## Pencere düzeni
//!
//! Tek bir **düz pencere** (`ana`) ve içinde iki tür webview:
//!
//! ```text
//! ┌─ ana (Window) ────────────────────────────────┐
//! │ kabuk (Webview, tam pencere, ilk yaratılan)   │
//! │   ├ sekme şeridi + gezinme çubuğu (üstte)     │
//! │   └ yeni sekme sayfası (alt bölge, sekme yoksa)│
//! │                                               │
//! │ sekme-N (Webview, kabuğun altındaki bölge)    │  ← sonra yaratılıyor,
//! │                                               │     z-sırasında üstte
//! └───────────────────────────────────────────────┘
//! ```
//!
//! Kabuk **tam pencere** çünkü adres çubuğu önerileri ve menüler kabuk
//! yüksekliğinin dışına taşmak zorunda. Sekme webview'i onun üstünde, arayüzün
//! bildirdiği içerik alanında duruyor (`icerik_alani` komutu).
//!
//! `WebviewWindow` yerine `Window` + `add_child` kullanılıyor: Tauri v2'nin
//! `unstable` çoklu webview API'si. Bu API minor sürümlerde değişebilir; bu
//! yüzden kırılma `motor/gercek.rs` ile bu dosyada kalıyor (`docs/Setup.md`).

pub mod bridge;
pub mod commands;
pub mod engel;
pub mod favicon;
pub mod gunluk;
pub mod hata;
pub mod history;
pub mod kisayol;
pub mod memory;
pub mod motor;
pub mod olaylar;
pub mod olcum;
pub mod settings;
pub mod tabs;
pub mod theme;

use std::sync::{Arc, Weak};

use tauri::{LogicalPosition, LogicalSize, Manager, RunEvent, WebviewUrl};

use motor::{Dinleyici, Kurulu, Motor};
use tabs::surucu::Surucu;

/// Baştaki UTF-8 BOM'unu atar.
///
/// `oturum.json` ve `ayarlar.json` insanların elle düzenleyebileceği dosyalar.
/// Windows'ta Not Defteri ve PowerShell'in `Set-Content -Encoding utf8`i
/// dosyanın başına `EF BB BF` koyuyor; `serde_json` bunu görünce ayrıştırmayı
/// reddediyor. Kırpılmazsa kullanıcı dosyasına bir satır ekliyor ve bütün
/// oturumunu kaybediyor — sebebini de göremiyor, çünkü okuma sessizce
/// varsayılana düşüyor.
pub(crate) fn bom_kirp(veri: &[u8]) -> &[u8] {
    veri.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(veri)
}

/// Pencerenin açılış boyutu. Kabuk kasıtlı olarak ince (iki satır); ekranın
/// geri kalanı sayfanın (`docs/Frontend.md`).
const PENCERE_GENISLIK: f64 = 1280.0;
const PENCERE_YUKSEKLIK: f64 = 820.0;
const ASGARI_GENISLIK: f64 = 900.0;
const ASGARI_YUKSEKLIK: f64 = 560.0;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Günlükçü en başta: kurulmadan önce yazılan `log::warn!`/`log::error!`
    // satırları kayboluyor ve Tauri'nin yuttuğu webview hataları tam da
    // açılışta çıkıyor (`gunluk.rs`).
    gunluk::kur();

    let uygulama = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::tabs::sekme_ac,
            commands::tabs::sekme_kapat,
            commands::tabs::sekme_etkinlestir,
            commands::tabs::sekme_tasi,
            commands::tabs::sekme_sabitle,
            commands::tabs::sekme_sessize_al,
            commands::tabs::sekme_uyutma_istisnasi,
            commands::tabs::sekme_listesi,
            commands::tabs::sekme_geri_al,
            commands::tabs::sekme_uyut,
            commands::tabs::sekme_at,
            commands::tabs::icerik_alani,
            commands::tabs::yetenekler,
            commands::gezinme::gezin,
            commands::gezinme::geri,
            commands::gezinme::ileri,
            commands::gezinme::yenile,
            commands::gezinme::durdur,
            commands::ayarlar::ayarlar_oku,
            commands::ayarlar::ayarlar_yaz,
            commands::tabs::kisayol_bas,
            commands::bellek::bellek_ozeti,
            commands::bellek::hepsini_uyut,
            commands::bellek::gecikme_ozeti,
            commands::bellek::gecikme_sifirla,
            commands::bellek::oyun_modu,
            commands::gecmis::gecmis_ara,
            commands::gecmis::gecmis_sil,
            commands::gecmis::gecmis_temizle,
            commands::gecmis::yer_imi_ekle,
            commands::gecmis::yer_imi_sil,
            commands::gecmis::yer_imi_listesi,
            commands::gecmis::yer_imi_mi,
            commands::gecmis::yer_imi_klasor_ekle,
            commands::gecmis::yer_imi_klasorleri,
            commands::tema::tema_listesi,
            commands::tema::tema_yukle,
            commands::tema::tema_uygula,
            commands::tema::tema_sil,
            commands::tema::tema_disa_aktar,
            commands::tema::tema_etkin,
            commands::tema::tema_arkaplan,
            commands::kopru::kopru_durumu,
            commands::kopru::kopru_tazele,
            commands::kopru::muiget_gonder,
            commands::kopru::indirme_izin_ver,
            commands::kopru::muiply_ac,
            commands::kopru::muiwatch_bagla,
            commands::kopru::muiwatch_birak,
            commands::kopru::favicon_oku,
            commands::ayarlar::engel_ozeti,
            commands::ayarlar::engel_sayaci,
            commands::ayarlar::veri_temizle,
            commands::tabs::grup_listesi,
            commands::tabs::grup_ac,
            commands::tabs::grup_sil,
            commands::tabs::grup_ata,
            commands::tabs::grup_guncelle,
            commands::tabs::ortu_gorunur,
        ])
        .setup(|app| {
            let veri_dizini = app.path().app_data_dir()?;
            std::fs::create_dir_all(&veri_dizini)?;

            // Ayarlar pencereden ÖNCE okunuyor: `surec_politikasi` ve
            // `renderer_tavani` Chromium bayraklarına dönüşüyor ve bayraklar
            // WebView2 ortamı kurulurken, yani ilk webview yaratılırken
            // geçiyor (`docs/Depolama.md`).
            let ayarlar = settings::oku(&veri_dizini.join("ayarlar.json"));

            // Bayrak dizesi burada **bir kez** hesaplanıyor ve hem kabuğa hem
            // motora aynı kopya gidiyor. Ayarlar çalışırken değişse bile bu
            // dize değişmiyor; değişseydi o andan sonra açılan sekmeler
            // kabuktan farklı bir WebView2 ortamı ister ve hiç açılmazdı
            // (`motor::gercek::WebView2Motor::sekme_ac`).
            let bayraklar = motor::ek_bayraklar(&ayarlar);

            let pencere = tauri::window::WindowBuilder::new(app, motor::ANA_PENCERE)
                .title("Muiren")
                .inner_size(PENCERE_GENISLIK, PENCERE_YUKSEKLIK)
                .min_inner_size(ASGARI_GENISLIK, ASGARI_YUKSEKLIK)
                .visible(false)
                // Kenarlıksız: başlık çubuğunun işini sekme şeridi görüyor
                // (`docs/Frontend.md`). Ayrı bir başlık satırı, ekranın
                // tepesinde sayfaya ait olması gereken 32 pikseli alıyor.
                // Sürükleme ve Aero Snap `data-tauri-drag-region` ile
                // şeridin zemininden sürüyor.
                .decorations(false)
                .build()?;

            // Kabuk webview'i pencerenin tamamını kaplıyor. Boyut mantıksal
            // birimle veriliyor: `inner_size()` fiziksel piksel döndürüyor ve
            // pencere daha ölçülmeden çağrılırsa (ilk kare) küçük bir değer
            // veriyor — o değerle yaratılan webview 14×14 doğuyor ve bir daha
            // büyümüyordu.
            pencere.add_child(
                kabuk_webview(&bayraklar),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(PENCERE_GENISLIK, PENCERE_YUKSEKLIK),
            )?;

            // Kabuk pencereyle birlikte büyüyor ama bu iş `auto_resize`e
            // BIRAKILMIYOR.
            //
            // `auto_resize` yaratma anındaki `webview/pencere` oranını
            // dondurup her yeniden boyutlandırmada onu uyguluyor. Yani
            // doğruluğu, webview eklenirken `window.inner_size()`in gerçek
            // değeri vermesine bağlı — ve bu dosyanın hemen yukarıdaki notu o
            // ölçümün güvenilmez olabildiğini yazıyor (pencere ölçülmeden
            // çağrıldığında 14×14 doğan webview). Oran bir kez yanlış
            // donduğunda kabuk bir daha asla pencereye oturmuyor ve belirtisi
            // sinsi: şeridin sağ ucundaki düğmeler görünür alanın dışında
            // kalıyor, sekmeler doğru göründüğü için sorun arayüzde sanılıyor.
            //
            // Oran yerine **mutlak** boyut: her `Resized` olayında kabuk
            // pencerenin iç boyutuna eşitleniyor. Tek bağımlılık o anki
            // gerçek boyut.
            //
            // Boyut **mantıksal** birimle veriliyor. `inner_size()` fiziksel
            // piksel döndürüyor ama `set_bounds` aldığı değeri mantıksal
            // sayıp DPI ölçeğiyle çarpıyor; fiziksel geçirmek %125 ölçekte
            // 1600 pikseli 2000'e çıkarıp kabuğu pencereden taşırıyor.
            // Ölçek burada bir kez uygulanıyor.
            //
            // > Ölçülen değerler (%125 ölçekli ekran): pencere içi
            // > 1600×1025 fiziksel → 1280×820 mantıksal; kabuğun CSS görünüm
            // > alanı 1280 px. `MUIREN_LOG=debug` ikisini de yazıyor.
            {
                let tutamak = app.handle().clone();
                pencere.on_window_event(move |olay| {
                    if !matches!(olay, tauri::WindowEvent::Resized(_)) {
                        return;
                    }
                    let (Some(p), Some(kabuk)) = (
                        tutamak.get_window(motor::ANA_PENCERE),
                        tutamak.get_webview(motor::KABUK),
                    ) else {
                        return;
                    };

                    // Simge durumu ayrı bir olay olarak GELMİYOR; Windows'ta
                    // küçültme bir `Resized` olarak düşüyor ve durumu ancak
                    // pencereye sorarak öğreniyoruz. Bu yüzden bayrak burada,
                    // boyutlandırma işinin yanında çevriliyor.
                    //
                    // Gözcü bunu okuyup baskıyı en az `Yuksek` sayıyor
                    // (`memory/gozcu.rs`): kullanıcı tarayıcıya bakmıyorsa
                    // sekmelerin uyanık beklemesinin karşılığı yok
                    // (`docs/Bellek.md`, "Gözcü"). Sürücü henüz kurulmamışsa
                    // (ilk `Resized` pencere yaratılırken gelebiliyor) sessizce
                    // atlanıyor; açılışta pencere zaten görünür.
                    if let (Ok(gizli), Some(surucu)) = (
                        p.is_minimized(),
                        tutamak.try_state::<Arc<Surucu<tauri::Wry>>>(),
                    ) {
                        memory::gozcu::pencere_gizli(surucu.inner(), gizli);
                    }
                    let (Ok(boyut), Ok(olcek)) = (p.inner_size(), p.scale_factor()) else {
                        return;
                    };
                    let m = boyut.to_logical::<f64>(olcek);
                    let _ = kabuk.set_bounds(tauri::Rect {
                        position: LogicalPosition::new(0.0, 0.0).into(),
                        size: m.into(),
                    });
                    // Kabuğun pencereye oturduğunu görmenin tek ucuz yolu:
                    // ekran görüntüsü almadan, `MUIREN_LOG=debug` ile.
                    log::debug!(
                        "kabuk boyutlandı: {}x{} fiziksel → {:.0}x{:.0} mantıksal (ölçek {olcek})",
                        boyut.width,
                        boyut.height,
                        m.width,
                        m.height
                    );
                });
            }

            // Pencere gizli yaratılıp burada gösteriliyor: kabuk webview'i
            // eklenmeden gösterilseydi kullanıcı bir kare boyunca boş bir
            // dikdörtgen görürdü.
            pencere.center()?;
            pencere.show()?;
            pencere.set_focus()?;

            // Gösterildikten SONRA bir kez daha eşitle: pencere ancak burada
            // gerçek boyutunu biliyor ve `Resized` olayı gelmeden önceki ilk
            // kare yanlış boyutta kalırdı.
            if let (Ok(boyut), Ok(olcek), Some(kabuk)) = (
                pencere.inner_size(),
                pencere.scale_factor(),
                app.get_webview(motor::KABUK),
            ) {
                let m = boyut.to_logical::<f64>(olcek);
                let _ = kabuk.set_bounds(tauri::Rect {
                    position: LogicalPosition::new(0.0, 0.0).into(),
                    size: m.into(),
                });
                log::debug!(
                    "kabuk açılışta eşitlendi: {}x{} fiziksel → {:.0}x{:.0} mantıksal (ölçek {olcek})",
                    boyut.width,
                    boyut.height,
                    m.width,
                    m.height
                );
            }

            let motor = Kurulu::yeni(app.handle().clone(), bayraklar);
            let surucu = Surucu::yeni(app.handle().clone(), motor, veri_dizini);
            app.manage(surucu.clone());

            // Motor sürücüyü ZAYIF referansla tanıyor: güçlü olsaydı
            // sürücü → motor → sürücü döngüsü kurulur ve kapanışta ikisi de
            // düşmezdi.
            let guclu: Arc<dyn Dinleyici> = surucu.clone();
            let zayif: Weak<dyn Dinleyici> = Arc::downgrade(&guclu);
            surucu.motor().dinleyici_ata(zayif);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Muiren başlatılamadı");

    uygulama.run(|app, olay| match olay {
        // Oturum `setup`ta DEĞİL burada açılıyor: `Ready`, olay döngüsünün
        // ayakta olduğu ilk an ve sekme webview'leri o noktadan sonra
        // yaratılıyor.
        //
        // Buradaki her `sekme_ac` hatası yalnız `eprintln!` ile görünüyor.
        // Motorun kendi hataları için o yetmiyor: Tauri ve wry webview
        // yaratma hatalarını `log::error!` ile yazıp `Ok` dönebiliyor, bu
        // yüzden `gunluk::kur()` en başta çağrılıyor.
        RunEvent::Ready => {
            let Some(surucu) = app.try_state::<Arc<Surucu<tauri::Wry>>>() else {
                return;
            };
            if let Err(e) = surucu.inner().clone().baslat() {
                eprintln!("muiren: oturum açılamadı: {e}");
            }

            // Ölçüm modu — yalnız `MUIREN_OLCUM` tanımlıysa
            // (`docs/olcumler/README.md`). Oturum açıldıktan SONRA
            // başlıyor: koşucunun ilk işi "boş tarayıcı" satırını ölçmek ve
            // oturumdan gelen sekmeler o sayıya karışmadan önce sayılmaları
            // gerekiyor — sayıldıklarında rapora not olarak düşüyorlar.
            if let Some(plan) = olcum::istenen_plan() {
                olcum::kosucu::baslat(surucu.inner(), plan);
            }
        }
        // Kapanışta oturumu diske yaz. Debounce'lu döngü son 2 saniyeyi
        // kaçırabilir; kapanış kaybı kabul edilmiyor (`docs/Sekmeler.md`).
        RunEvent::ExitRequested { .. } => {
            if let Some(surucu) = app.try_state::<Arc<Surucu<tauri::Wry>>>() {
                let _ = surucu.oturum_yaz();
            }
        }
        _ => {}
    });
}

/// Kabuk webview'i.
///
/// `bayraklar` çağırandan geliyor, burada hesaplanmıyor: **aynı dize** sekme
/// webview'lerine de gitmek zorunda. wry her webview için ayrı bir WebView2
/// ortamı kuruyor ve WebView2 aynı kullanıcı veri klasörünü paylaşan ikinci
/// ortamı yalnız seçenekleri aynıysa yaratıyor — ayrışırlarsa sekme sessizce
/// ölü doğuyor (gerekçenin tamamı `motor/gercek.rs`, `sekme_ac`).
fn kabuk_webview<R: tauri::Runtime>(bayraklar: &str) -> tauri::webview::WebviewBuilder<R> {
    // `auto_resize` KULLANILMIYOR: oran hesabı yaratma anındaki pencere
    // boyutuna bağlı ve o boyut ilk karede güvenilir değil. Kabuğun boyutunu
    // `lib.rs` içindeki `Resized` işleyicisi mutlak olarak veriyor.
    let builder = tauri::webview::WebviewBuilder::new(motor::KABUK, WebviewUrl::default());

    // Boş dize "dokunma" demek: `additional_browser_args("")` çağrılsaydı
    // wry'nin kendi varsayılanları da silinirdi. Motorsuz derlemede
    // (`motor/yok.rs`) dize boş ve iki taraf da dokunmuyor — yine tutarlı.
    if bayraklar.is_empty() {
        builder
    } else {
        builder.additional_browser_args(bayraklar)
    }
}
