//! WebView2 sarmalayıcısı — **projenin en kritik dosyası**.
//!
//! Bozulunca sekmeler sessizce ölü doğuyor: kayıt var, çubukta yeri var, ama
//! sayfa açılmıyor.
//!
//! Burada dört iş var:
//!
//! 1. Sekme başına webview yaratma/yıkma (Tauri v2 `unstable` çoklu webview).
//! 2. Ham `ICoreWebView2` üzerinden `TrySuspend` / `Resume` ve bellek hedefi —
//!    Tauri'nin sardığı API'de bu çağrılar yok (`docs/Setup.md`).
//! 3. Olay köprüsü: başlık, adres, gezinme, geçmiş, ses. Sayfaya **hiçbir
//!    ayrıcalıklı kanal açılmadan** (`docs/IPC.md`) — bilgi WebView2'nin kendi
//!    olaylarından geliyor, sayfanın gönderdiği bir mesajdan değil.
//! 4. `EK_BAYRAKLAR`: Chromium süreç politikası.
//!
//! `unsafe` **yalnız bu dosyada**. Başka bir modülde çıkarsa mimari bozulmuş
//! demektir (`docs/Setup.md`).
//!
//! ## Yetenekler bir hata değil
//!
//! `ICoreWebView2_3`, `_8`, `_19` arayüzlerine `cast` kullanıcının kurulu
//! Runtime sürümüne göre başarısız olabiliyor. Bu bir **yetenek yokluğu**:
//! politika geri çekiliyor (`Uyuyan` atlanıyor, doğrudan `Atilmis`
//! kullanılıyor), tarayıcı çalışmaya devam ediyor.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    GetAvailableCoreWebView2BrowserVersionString, ICoreWebView2, ICoreWebView2Environment13,
    ICoreWebView2FrameInfo2, ICoreWebView2ProcessExtendedInfoCollection, ICoreWebView2Profile2,
    ICoreWebView2_13, ICoreWebView2_15, ICoreWebView2_19, ICoreWebView2_2, ICoreWebView2_20,
    ICoreWebView2_3, ICoreWebView2_4, ICoreWebView2_8, COREWEBVIEW2_BROWSING_DATA_KINDS,
    COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_DOM_STORAGE, COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES,
    COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE, COREWEBVIEW2_BROWSING_DATA_KINDS_GENERAL_AUTOFILL,
    COREWEBVIEW2_BROWSING_DATA_KINDS_SERVICE_WORKERS, COREWEBVIEW2_FAVICON_IMAGE_FORMAT_PNG,
    COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN, COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
    COREWEBVIEW2_PROCESS_KIND_RENDERER, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
};
use webview2_com::{
    take_pwstr, AcceleratorKeyPressedEventHandler, ClearBrowsingDataCompletedHandler,
    ContainsFullScreenElementChangedEventHandler, DocumentTitleChangedEventHandler,
    DownloadStartingEventHandler, ExecuteScriptCompletedHandler, FaviconChangedEventHandler,
    GetFaviconCompletedHandler, GetProcessExtendedInfosCompletedHandler,
    HistoryChangedEventHandler, IsDocumentPlayingAudioChangedEventHandler,
    NavigationCompletedEventHandler, NavigationStartingEventHandler,
    NewWindowRequestedEventHandler, SourceChangedEventHandler, TrySuspendCompletedHandler,
    WebResourceRequestedEventHandler,
};
use windows::core::Interface;
use windows::Win32::System::Com::IStream;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_CONTROL, VK_MENU, VK_SHIFT,
};

use super::{
    etiket, Asama, BellekSeviyesi, Dikdortgen, Dinleyici, IndirmeKarari, Motor, SurecKaydi,
    Yetenekler, ANA_PENCERE,
};
use crate::hata::{MuirenHata, Sonuc};
use crate::settings::{Settings, SurecPolitikasi};
use crate::tabs::SekmeId;

/// WebView2 ortamı kurulurken `AdditionalBrowserArguments` ile geçirilen
/// anahtarlar. **Tek yer burası** ve her satırın yanında gerekçesi var
/// (`docs/Setup.md`).
///
/// Yasaklılar, gerekçeleriyle:
///
/// - `--disable-site-isolation-trials` ve türevleri: en çok kazandıran bayrak
///   ve **kullanılmıyor**. Site izolasyonu Spectre türü yan kanal
///   saldırılarına karşı tarayıcının tek gerçek savunması; içinde bankacılık
///   sekmesi açılan bir programda bu takas yapılmaz (`docs/Bellek.md`).
/// - GPU hızlandırmayı kapatan bayraklar: video takılmasının bir numaralı
///   sebebi (`docs/Medya.md`).
///
/// > Bu bayrakların WebView2'den gerçekten geçtiği **henüz ölçülmedi** —
/// > Faz 0/R4. WebView2 bir kısmını sessizce yok sayıyor. Ölçüm yöntemi:
/// > aynı siteden 10 sekme aç, `Get-Process msedgewebview2` ile süreç say.
const EK_BAYRAKLAR: &[&str] = &[
    // wry'nin varsayılanının bir parçası. `additional_browser_args` verildiğinde
    // wry kendi varsayılanını GEÇİYOR, dolayısıyla burada tekrar edilmezse
    // Edge'in kendi arayüz parçaları (OOUI, PDF arayüzü, SmartScreen) devreye
    // giriyor.
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
    // wry'nin varsayılanındaki `--autoplay-policy=no-user-gesture-required`
    // **bilinçli olarak alınmadı**: bir uygulama kabuğu için makul, bir
    // tarayıcı için değil. Onunla birlikte açılan her sayfa sesli video
    // başlatabiliyor. Chromium'un varsayılan politikası kalıyor: sessiz video
    // oynuyor, sesli olan kullanıcı hareketi bekliyor.
];

/// Ayarları Chromium anahtar dizesine çeviriyor.
///
/// **Bir kez** çağrılıp sonucu saklanıyor (`lib.rs`), her webview'de yeniden
/// hesaplanmıyor: kabuk ile sekmeler aynı dizeyi almazsa sekmelerin WebView2
/// ortamı hiç kurulamıyor (gerekçe [`WebView2Motor::sekme_ac`] içinde).
///
/// Bu ayarlar **yeniden başlatma** gerektiriyor: bayraklar ancak WebView2
/// ortamı yeniden kurulduğunda etkili oluyor (`docs/Depolama.md`).
pub fn ek_bayraklar(ayarlar: &Settings) -> String {
    let mut parcalar: Vec<String> = EK_BAYRAKLAR.iter().map(|b| b.to_string()).collect();

    if ayarlar.surec_politikasi == SurecPolitikasi::Birlesik {
        // Aynı sitenin sekmeleri tek render sürecini paylaşsın. Süreç başına
        // 30-60 MB taban maliyet var; aynı siteden 10 sekme açan kullanıcı
        // varsayılan modelde yarım GB'ı boşa harcıyor. Siteler ARASI izolasyon
        // korunuyor — `--process-per-site` yalnız AYNI sitenin sekmelerini
        // birleştiriyor (`docs/Bellek.md`).
        parcalar.push("--process-per-site".into());
    }
    if ayarlar.renderer_tavani > 0 {
        // Toplam renderer tavanı. Değer `settings`ten geliyor, burada sabit
        // değil (CLAUDE.md #6): 8 GB'lık makinede 4-6 makul, 32 GB'da
        // gereksiz.
        parcalar.push(format!(
            "--renderer-process-limit={}",
            ayarlar.renderer_tavani
        ));
    }

    parcalar.join(" ")
}

/// [`Dikdortgen`]i Tauri'nin beklediği sınırlara çevirir.
///
/// Konum ve boyut **tek çağrıda** veriliyor: `set_position` + `set_size` ayrı
/// ayrı gönderildiğinde ikisi arasında bir kare geçiyor ve webview yanlış
/// yerde bir an görünüyor. Ayrıca ayrı çağrılar wry'de birbirini eziyordu —
/// sekme webview'i 1×1 kalıyordu.
fn sinirlar(d: Dikdortgen) -> tauri::Rect {
    tauri::Rect {
        position: LogicalPosition::new(d.x, d.y).into(),
        size: LogicalSize::new(d.genislik, d.yukseklik).into(),
    }
}

pub struct WebView2Motor<R: Runtime> {
    app: AppHandle<R>,
    /// Kabuk webview'ine verilen Chromium anahtar dizesinin **birebir aynısı**.
    ///
    /// Bir kopya değil, bir zorunluluk: WebView2 aynı kullanıcı veri
    /// klasörünü paylaşan iki ortamı yalnız seçenekleri **aynıysa**
    /// yaratıyor. Ayrıntı [`WebView2Motor::sekme_ac`] içinde.
    ///
    /// Açılışta donduruluyor: `settings` sonradan değişse bile buradaki
    /// dize değişmiyor, çünkü değişseydi o andan sonra açılan her sekme
    /// kabuktan farklı bir ortam isterdi. Bayrak ayarlarının "yeniden
    /// başlatma gerektiriyor" olmasının gerçek sebebi bu (`docs/Depolama.md`).
    bayraklar: String,
    /// Arayüzün bildirdiği içerik alanı; yeni webview'ler buraya doğuyor.
    alan: Mutex<Dikdortgen>,
    dinleyici: Mutex<Option<Weak<dyn Dinleyici>>>,
    /// `Arc` çünkü yetenek ölçümü ana iş parçacığına gönderilen bir kapanışın
    /// içinde bitiyor ve sonucu buraya geri yazması gerekiyor.
    yetenekler: std::sync::Arc<Mutex<Yetenekler>>,
    /// Sekme → **ana çerçeve kimliği** (`ICoreWebView2_20::FrameId`).
    ///
    /// Süreç → sekme eşlemesinin yarısı: `GetProcessExtendedInfos` bir
    /// sürecin taşıdığı çerçeveleri söylüyor, hangi çerçevenin hangi sekme
    /// olduğunu söyleyen tek yer bu tablo.
    ///
    /// **Gezinme bittiğinde tazeleniyor**, ölçüm anında sorulmuyor: sorsaydık
    /// her turda uyuyan sekmelerin webview'lerine dokunmak gerekirdi ve
    /// CLAUDE.md #5 tam olarak bunu yasaklıyor. Gezinme zaten sekmenin uyanık
    /// olduğu andır; ölçüm o anda yazılan değeri okuyor.
    cerceveler: Arc<Mutex<HashMap<SekmeId, u32>>>,
    /// Son süreç anlık görüntüsü (`Motor::surec_haritasi`).
    ///
    /// `Arc` gerekçesi `yetenekler` ile aynı: cevap eşzamansız bir geri
    /// çağrının içinde, ana iş parçacığında geliyor.
    surecler: Arc<Mutex<Vec<SurecKaydi>>>,
}

impl<R: Runtime> WebView2Motor<R> {
    pub fn yeni(app: AppHandle<R>, bayraklar: String) -> Self {
        WebView2Motor {
            app,
            bayraklar,
            alan: Mutex::new(Dikdortgen::BOS),
            dinleyici: Mutex::new(None),
            yetenekler: std::sync::Arc::new(Mutex::new(Yetenekler {
                motor: true,
                // Aşağıdakiler ilk webview yaratıldığında ölçülüyor: `cast`
                // denenmeden bilinmiyorlar.
                ..Yetenekler::default()
            })),
            cerceveler: Arc::new(Mutex::new(HashMap::new())),
            surecler: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn webview(&self, id: SekmeId) -> Sonuc<tauri::webview::Webview<R>> {
        self.app
            .get_webview(&etiket(id))
            .ok_or(MuirenHata::SekmeYok(id))
    }

    /// Ham `ICoreWebView2` üzerinde bir iş çalıştırır.
    ///
    /// Fire-and-forget: `with_webview` kapanışı ana iş parçacığına gönderiyor
    /// ve sonucu geri vermiyor. Bellek politikasının buna ihtiyacı yok —
    /// "uyut" dedikten sonra cevabı beklemenin karşılığı yok, sonuç zaten
    /// bir sonraki ölçümde görünüyor.
    fn ham<F>(&self, id: SekmeId, is: F) -> Sonuc<()>
    where
        F: FnOnce(&ICoreWebView2) + Send + 'static,
    {
        let webview = self.webview(id)?;
        webview
            .with_webview(move |pw| {
                // SAFETY: `controller()` Tauri'nin tuttuğu geçerli bir COM
                // işaretçisi; kapanış ana iş parçacığında, WebView2'nin kendi
                // döngüsünde çalışıyor.
                unsafe {
                    if let Ok(core) = pw.controller().CoreWebView2() {
                        is(&core);
                    }
                }
            })
            .map_err(|e| MuirenHata::Motor(e.to_string()))
    }

    /// Yetenekleri ilk webview üzerinde ölçüyor.
    ///
    /// Ölçüm ana iş parçacığında, biraz sonra bitiyor. O ana kadar politika
    /// "askıya alma yok" varsayıyor ve doğrudan atmaya düşüyor — yanlış yönde
    /// hata yapmak (uyutulabilecek bir sekmeyi atmak) doğru yön: sekme URL'den
    /// geri geliyor, tersi sessizce çalışmayan bir `TrySuspend` demek.
    fn yetenekleri_olc(&self, webview: &tauri::webview::Webview<R>) {
        let hedef = self.yetenekler.clone();
        let _ = webview.with_webview(move |pw| unsafe {
            let Ok(core) = pw.controller().CoreWebView2() else {
                return;
            };
            let mut y = hedef.lock().unwrap();
            y.askiya_alma = core.cast::<ICoreWebView2_3>().is_ok();
            y.bellek_hedefi = core.cast::<ICoreWebView2_19>().is_ok();
            y.indirme_olayi = core.cast::<ICoreWebView2_4>().is_ok();
            y.favicon = core.cast::<ICoreWebView2_15>().is_ok();
            // Süreç → sekme eşlemesi **iki** arayüz istiyor ve biri
            // olmadan diğerinin karşılığı yok: `ICoreWebView2_20::FrameId`
            // olmadan `GetProcessExtendedInfos`ın verdiği çerçeve kimlikleri
            // hiçbir sekmeye bağlanamıyor, tersi durumda da sorulacak bir
            // süreç listesi kalmıyor. `&&` yerine ayrı iki bayrak tutmak,
            // arayüze "yarı çalışan bir eşleme" göstermek olurdu.
            y.surec_bilgisi = core.cast::<ICoreWebView2_20>().is_ok()
                && core
                    .cast::<ICoreWebView2_2>()
                    .and_then(|v2| v2.Environment())
                    .and_then(|o| o.cast::<ICoreWebView2Environment13>())
                    .is_ok();
        });
    }

    /// Sekmenin ana çerçeve kimliğini tabloya yazar.
    ///
    /// Gezinme bittiğinde çağrılıyor. Kimlik gezinmeyle **değişebiliyor**
    /// (farklı kökene geçen sekme yeni bir sürece taşınıyor), o yüzden bir
    /// kez yazılıp bırakılmıyor.
    ///
    /// Sessizce başarısız oluyor: eski bir Runtime'da `ICoreWebView2_20` yok
    /// ve o durumda eşleme zaten kapalı (`Yetenekler::surec_bilgisi`), panel
    /// "yaklaşık" diyor. Her gezinmede bir hata satırı yazmanın karşılığı
    /// yok.
    ///
    /// # Safety
    /// `core` geçerli ve çağrı ana iş parçacığında.
    unsafe fn cerceve_kaydet(
        tablo: &Mutex<HashMap<SekmeId, u32>>,
        id: SekmeId,
        core: &ICoreWebView2,
    ) {
        let Ok(v20) = core.cast::<ICoreWebView2_20>() else {
            return;
        };
        let mut kimlik = 0u32;
        if v20.FrameId(&mut kimlik).is_ok() && kimlik != 0 {
            tablo.lock().unwrap().insert(id, kimlik);
        }
    }

    /// Sekme webview'ine WebView2 olaylarını bağlar.
    ///
    /// Bilginin tamamı motorun kendi olaylarından geliyor; sayfaya enjekte
    /// edilen bir betikten değil. Sayfaya açılan her ayrıcalıklı kanal, açılan
    /// her sitenin o kanala erişmesi demek (`docs/IPC.md`).
    fn olaylari_bagla(&self, webview: &tauri::webview::Webview<R>, id: SekmeId) {
        let Some(dinleyici) = self.dinleyici.lock().unwrap().clone() else {
            return;
        };
        // Kapanış `'static` olmak zorunda: `self`ten tutulacak her şey
        // buradan, kapanışın dışından kopyalanıyor.
        let cerceveler = self.cerceveler.clone();

        let _ = webview.with_webview(move |pw| unsafe {
            let Ok(core) = pw.controller().CoreWebView2() else {
                return;
            };
            let mut token = 0i64;

            // --- başlık ---
            let d = dinleyici.clone();
            let _ = core.add_DocumentTitleChanged(
                &DocumentTitleChangedEventHandler::create(Box::new(move |sender, _| {
                    if let (Some(s), Some(d)) = (sender, d.upgrade()) {
                        let mut ham = windows::core::PWSTR::null();
                        if s.DocumentTitle(&mut ham).is_ok() {
                            // Ham metin dinleyiciye gidiyor; temizlemeyi
                            // `tabs::baslik_temizle` yapıyor (CLAUDE.md #8).
                            d.baslik_degisti(id, take_pwstr(ham));
                        }
                    }
                    Ok(())
                })),
                &mut token,
            );

            // --- adres ---
            let d = dinleyici.clone();
            let _ = core.add_SourceChanged(
                &SourceChangedEventHandler::create(Box::new(move |sender, _| {
                    if let (Some(s), Some(d)) = (sender, d.upgrade()) {
                        let mut ham = windows::core::PWSTR::null();
                        if s.Source(&mut ham).is_ok() {
                            d.adres_degisti(id, take_pwstr(ham));
                        }
                    }
                    Ok(())
                })),
                &mut token,
            );

            // --- gezinme başladı ---
            //
            // İki iş bir arada: engelleme kararı ve "gezinme başladı"
            // bildirimi. Sıra önemli — engellenen bir gezinme için
            // `Basladi` yayınlanmıyor, yoksa adres çubuğu hiç gidilmeyen bir
            // adresi gösterir ve yükleme çubuğu sonsuza kadar döner.
            let d = dinleyici.clone();
            let _ = core.add_NavigationStarting(
                &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                    let (Some(a), Some(d)) = (args, d.upgrade()) else {
                        return Ok(());
                    };
                    let mut ham = windows::core::PWSTR::null();
                    if a.Uri(&mut ham).is_err() {
                        return Ok(());
                    }
                    let url = take_pwstr(ham);

                    // Pop-up/yönlendirme politikasının girdileri. İkisi de
                    // WebView2'nin kendi ayrımı; kendi sezgimizi yazmıyoruz
                    // (`engel/mod.rs`).
                    let mut kullanici = windows::core::BOOL(0);
                    let _ = a.IsUserInitiated(&mut kullanici);
                    let mut yonlendirme = windows::core::BOOL(0);
                    let _ = a.IsRedirected(&mut yonlendirme);

                    if !d.gezinme_izni(id, &url, kullanici.as_bool(), yonlendirme.as_bool()) {
                        let _ = a.SetCancel(true);
                        return Ok(());
                    }
                    d.gezinme(id, Asama::Basladi, url, false);
                    Ok(())
                })),
                &mut token,
            );

            // --- gezinme bitti ---
            //
            // `IsSuccess` false ise geçmişe yazılmıyor, sekme başlığı
            // eskisiyle kalmıyor, oturum dosyasına o URL işlenmiyor
            // (CLAUDE.md #7). Karar dinleyicide; burada yalnız bayrak
            // taşınıyor.
            let d = dinleyici.clone();
            // Süreç → sekme eşlemesinin çerçeve ucu buraya asılıyor: gezinme
            // sekmenin uyanık olduğu andır ve ana çerçeve kimliği bu anda
            // kesinleşiyor. Ölçüm anında sormak, her turda uyuyan sekmelere
            // dokunmak olurdu (CLAUDE.md #5).
            let _ = core.add_NavigationCompleted(
                &NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
                    if let Some(s) = sender.as_ref() {
                        // Dinleyiciden **önce** ve ondan bağımsız: çerçeve
                        // kimliği bellek ölçümünün girdisi ve dinleyicinin
                        // düşmüş olması ölçümü de kör etmemeli.
                        Self::cerceve_kaydet(&cerceveler, id, s);
                    }
                    let (Some(s), Some(a), Some(d)) = (sender, args, d.upgrade()) else {
                        return Ok(());
                    };
                    let mut basarili = windows::core::BOOL(0);
                    let _ = a.IsSuccess(&mut basarili);
                    let mut ham = windows::core::PWSTR::null();
                    let url = if s.Source(&mut ham).is_ok() {
                        take_pwstr(ham)
                    } else {
                        String::new()
                    };
                    d.gezinme(id, Asama::Bitti, url, basarili.as_bool());
                    Ok(())
                })),
                &mut token,
            );

            // --- geri/ileri ---
            let d = dinleyici.clone();
            let _ = core.add_HistoryChanged(
                &HistoryChangedEventHandler::create(Box::new(move |sender, _| {
                    if let (Some(s), Some(d)) = (sender, d.upgrade()) {
                        let mut geri = windows::core::BOOL(0);
                        let mut ileri = windows::core::BOOL(0);
                        let _ = s.CanGoBack(&mut geri);
                        let _ = s.CanGoForward(&mut ileri);
                        d.gecmis_degisti(id, geri.as_bool(), ileri.as_bool());
                    }
                    Ok(())
                })),
                &mut token,
            );

            // --- yeni pencere ---
            //
            // `window.open` ve `target="_blank"` ayrı bir WebView2 penceresi
            // açardı: kabuğun dışında, sekme çubuğunda görünmeyen, bellek
            // politikasının bilmediği bir pencere. Engelleyip kendi sekmemizi
            // açıyoruz.
            let d = dinleyici.clone();
            let _ = core.add_NewWindowRequested(
                &NewWindowRequestedEventHandler::create(Box::new(move |_, args| {
                    if let Some(a) = args {
                        let _ = a.SetHandled(true);
                        let mut ham = windows::core::PWSTR::null();
                        if a.Uri(&mut ham).is_ok() {
                            if let Some(d) = d.upgrade() {
                                // Kullanıcının tıkladığı bağlantı ile
                                // sayfanın kendi kendine açtığı pencere
                                // arasındaki ayrım: pop-up politikasının
                                // tek girdisi (`engel/mod.rs`). Kendi
                                // sezgimizi yazmıyoruz.
                                let mut kullanici = windows::core::BOOL(0);
                                let _ = a.IsUserInitiated(&mut kullanici);
                                d.yeni_pencere(id, take_pwstr(ham), kullanici.as_bool());
                            }
                        }
                    }
                    Ok(())
                })),
                &mut token,
            );

            // --- kısayollar ---
            //
            // Sayfa odaktayken tuşlar kabuğa hiç ulaşmıyor; `docs/Frontend.md`
            // bunu "her kısayolun iki karşılığı olmak zorunda" diye yazıyor.
            // İkinci karşılık burası.
            //
            // Olay **controller** üzerinde, `CoreWebView2` üzerinde değil.
            // Değiştiriciler olayla gelmiyor, klavye durumundan okunuyor.
            // Tuşun ne yaptığına motor karar VERMİYOR: ham tuş dinleyiciye
            // gidiyor, tabloyu (`crate::kisayol`) o çalıştırıyor.
            let d = dinleyici.clone();
            let mut accel_token = 0i64;
            let _ = pw.controller().add_AcceleratorKeyPressed(
                &AcceleratorKeyPressedEventHandler::create(Box::new(move |_, args| {
                    let (Some(a), Some(d)) = (args, d.upgrade()) else {
                        return Ok(());
                    };
                    let mut tur = COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN;
                    let _ = a.KeyEventKind(&mut tur);
                    // Yalnız basma anı. Bırakma anını da işlemek her kısayolu
                    // iki kez tetiklerdi.
                    if tur != COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN
                        && tur != COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN
                    {
                        return Ok(());
                    }

                    let mut vk = 0u32;
                    if a.VirtualKey(&mut vk).is_err() {
                        return Ok(());
                    }
                    let Some(ad) = crate::kisayol::vk_adi(vk) else {
                        return Ok(());
                    };

                    let basili = |k: VIRTUAL_KEY| (GetKeyState(k.0 as i32) as u16 & 0x8000) != 0;
                    let ctrl = basili(VK_CONTROL);
                    let shift = basili(VK_SHIFT);
                    let alt = basili(VK_MENU);

                    // Tabloda karşılığı varsa sayfaya GEÇİRMİYORUZ: Ctrl+T
                    // hem sekme açıp hem sayfada bir şey tetiklerse kullanıcı
                    // ne olduğunu anlamıyor.
                    if crate::kisayol::coz(&ad, ctrl, shift, alt).is_some() {
                        let _ = a.SetHandled(true);
                    }
                    d.kisayol(id, ad, ctrl, shift, alt);
                    Ok(())
                })),
                &mut accel_token,
            );
            // --- ses ---
            //
            // Koruma kuralı #2: ses çalan sekme uyutulmuyor. Arka planda
            // müzik/podcast dinlemek birinci sınıf kullanım (`docs/Bellek.md`).
            // `ICoreWebView2_8` yoksa bu bilgi hiç gelmiyor ve sekmeler
            // korumasız kalıyor — eski Runtime'ın bedeli.
            if let Ok(v8) = core.cast::<ICoreWebView2_8>() {
                let d = dinleyici.clone();
                let _ = v8.add_IsDocumentPlayingAudioChanged(
                    &IsDocumentPlayingAudioChangedEventHandler::create(Box::new(
                        move |sender, _| {
                            if let (Some(s), Some(d)) = (sender, d.upgrade()) {
                                if let Ok(v8) = s.cast::<ICoreWebView2_8>() {
                                    let mut caliyor = windows::core::BOOL(0);
                                    let _ = v8.IsDocumentPlayingAudio(&mut caliyor);
                                    d.ses_degisti(id, caliyor.as_bool());
                                }
                            }
                            Ok(())
                        },
                    )),
                    &mut token,
                );
            }

            // --- istek süzgeci ---
            //
            // **Yalnız filtre açıkken kaydediliyor.** `WebResourceRequested`
            // süzgeci sayfadaki her alt kaynağı kabuk sürecinden geçiriyor;
            // kapalı bir filtre için o bedeli ödemenin karşılığı yok. Bedeli
            // ise şu: ayar değiştiğinde yalnız sonradan açılan ya da yeniden
            // yüklenen sekmeler etkileniyor. Arayüz bunu rozetle söylüyor
            // (`surec_politikasi` rozetiyle aynı kalıp, `docs/Frontend.md`).
            if dinleyici.upgrade().is_some_and(|d| d.istek_suzgeci_acik()) {
                // Süzgeç `*`: bütün kaynak türleri. Daha dar bir süzgeç
                // (yalnız `Script`, `Image`) daha ucuz olurdu ama
                // kullanıcının kuralı bir `fetch` ya da `beacon` adresini
                // gösterdiğinde sessizce çalışmazdı.
                let desen = windows::core::HSTRING::from("*");
                let _ = core.AddWebResourceRequestedFilter(
                    windows::core::PCWSTR(desen.as_ptr()),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                );

                let d = dinleyici.clone();
                let _ = core.add_WebResourceRequested(
                    &WebResourceRequestedEventHandler::create(Box::new(move |sender, args| {
                        let (Some(s), Some(a), Some(d)) = (sender, args, d.upgrade()) else {
                            return Ok(());
                        };
                        let Ok(istek) = a.Request() else {
                            return Ok(());
                        };
                        let mut ham = windows::core::PWSTR::null();
                        if istek.Uri(&mut ham).is_err() {
                            return Ok(());
                        }
                        let url = take_pwstr(ham);
                        if d.istek_izni(id, &url) {
                            return Ok(());
                        }

                        // Engelleme, isteği "iptal etmekle" değil **boş bir
                        // 403 yanıtı vermekle** yapılıyor: WebView2'de
                        // `WebResourceRequested` için iptal diye bir alan
                        // yok, olan tek şey yanıtı biz üretmek. Boş gövde,
                        // sayfanın "yüklenemedi" yolunu çalıştırıyor —
                        // reklam yerinde boşluk kalıyor, sayfa çalışmaya
                        // devam ediyor.
                        if let Ok(v2) = s.cast::<ICoreWebView2_2>() {
                            if let Ok(ortam) = v2.Environment() {
                                let sebep = windows::core::HSTRING::from("Muiren engelledi");
                                let basliklar = windows::core::HSTRING::from("");
                                if let Ok(yanit) = ortam.CreateWebResourceResponse(
                                    None,
                                    403,
                                    windows::core::PCWSTR(sebep.as_ptr()),
                                    windows::core::PCWSTR(basliklar.as_ptr()),
                                ) {
                                    let _ = a.SetResponse(&yanit);
                                }
                            }
                        }
                        Ok(())
                    })),
                    &mut token,
                );
            }

            // --- tam ekran öğe ---
            //
            // Koruma kuralı #8: sayfa tam ekran bir video ya da oyun
            // gösteriyorsa uyutulmuyor (`docs/Bellek.md`). Bu alan uzun süre
            // `SekmeOzeti` içinde vardı ama **hiç doldurulmuyordu**, yani
            // kural yazılı olduğu hâlde çalışmıyordu. Olay temel
            // `ICoreWebView2` üzerinde: yetenek kontrolü gerekmiyor.
            let d = dinleyici.clone();
            let _ = core.add_ContainsFullScreenElementChanged(
                &ContainsFullScreenElementChangedEventHandler::create(Box::new(
                    move |sender, _| {
                        if let (Some(s), Some(d)) = (sender, d.upgrade()) {
                            let mut tam = windows::core::BOOL(0);
                            if s.ContainsFullScreenElement(&mut tam).is_ok() {
                                d.tam_ekran_degisti(id, tam.as_bool());
                            }
                        }
                        Ok(())
                    },
                )),
                &mut token,
            );

            // --- indirme ---
            //
            // Muiget köprüsü (`docs/Kopruler.md`, karar #4). Karar burada
            // **verilmiyor**: dinleyici ayarı ve köprünün kurulu olup
            // olmadığını biliyor, motor yalnız cevabı uyguluyor.
            //
            // Geri çağrı eşzamanlı olmak zorunda — `SetCancel` bu olay
            // bitmeden çağrılmazsa indirme başlıyor. Bu yüzden
            // `indirme_basladi` ucuz: ayar okuyor, devri ayrı bir iş
            // parçacığına atıyor.
            if let Ok(v4) = core.cast::<ICoreWebView2_4>() {
                let d = dinleyici.clone();
                let _ = v4.add_DownloadStarting(
                    &DownloadStartingEventHandler::create(Box::new(move |_, args| {
                        let (Some(a), Some(d)) = (args, d.upgrade()) else {
                            return Ok(());
                        };
                        let Ok(islem) = a.DownloadOperation() else {
                            return Ok(());
                        };

                        let mut ham = windows::core::PWSTR::null();
                        let url = if islem.Uri(&mut ham).is_ok() {
                            take_pwstr(ham)
                        } else {
                            String::new()
                        };
                        // Motorun önerdiği tam yol; bize yalnız son
                        // bileşeni lazım ve o da sayfadan geliyor.
                        let mut ham = windows::core::PWSTR::null();
                        let hedef = if islem.ResultFilePath(&mut ham).is_ok() {
                            take_pwstr(ham)
                        } else {
                            String::new()
                        };
                        let mut boyut = 0i64;
                        let _ = islem.TotalBytesToReceive(&mut boyut);

                        if d.indirme_basladi(id, url, hedef, boyut.max(0) as u64)
                            == IndirmeKarari::Iptal
                        {
                            // İptal + `Handled`: ikisi birlikte veriliyor.
                            // Yalnız `SetCancel` çağrılsaydı WebView2 kendi
                            // "indirme iptal edildi" balonunu gösterirdi ve
                            // kullanıcı devri bir hata sanardı.
                            let _ = a.SetCancel(true);
                            let _ = a.SetHandled(true);
                        }
                        Ok(())
                    })),
                    &mut token,
                );
            }

            // --- favicon ---
            //
            // Adres DEĞİL baytlar alınıyor. `FaviconUri` de var ve daha
            // ucuz görünüyor; kullanılmıyor çünkü o adresi kabuğa basmak,
            // açılan her sitenin kabuk penceresine ağ isteği yaptırabilmesi
            // demek (`tabs::Sekme::favicon`, `docs/IPC.md`). `GetFavicon`
            // baytları motorun kendi önbelleğinden veriyor: ikinci bir ağ
            // isteği yok, sayfaya açılan kanal yok.
            if let Ok(v15) = core.cast::<ICoreWebView2_15>() {
                let d = dinleyici.clone();
                let _ = v15.add_FaviconChanged(
                    &FaviconChangedEventHandler::create(Box::new(move |sender, _| {
                        let (Some(s), Some(d)) = (sender, d.upgrade()) else {
                            return Ok(());
                        };
                        let Ok(v15) = s.cast::<ICoreWebView2_15>() else {
                            return Ok(());
                        };
                        let d = d.clone();
                        // PNG isteniyor: depo tek biçim tanıyor ve `.ico`
                        // çözmek için ikinci bir kod yolu gerekirdi.
                        let _ = v15.GetFavicon(
                            COREWEBVIEW2_FAVICON_IMAGE_FORMAT_PNG,
                            &GetFaviconCompletedHandler::create(Box::new(move |hr, akis| {
                                if hr.is_err() {
                                    return Ok(());
                                }
                                // Sayfası favicon'u olmayan sekmede akış
                                // boş geliyor; hata değil.
                                if let Some(veri) = akis.and_then(|a| akisi_oku(&a)) {
                                    d.favicon_geldi(id, veri);
                                }
                                Ok(())
                            })),
                        );
                        Ok(())
                    })),
                    &mut token,
                );
            }
        });
    }
}

/// Favicon akışının üst sınırı.
///
/// Sayfa bu baytları belirliyor; sınırsız okumak, bir sekmenin kabuğa
/// istediği kadar bellek ayırtabilmesi demek. 512 KB bir PNG favicon için
/// fazlasıyla cömert (tipik olan 1–20 KB).
const FAVICON_SINIRI: usize = 512 * 1024;

/// `IStream`i baytlara çevirir.
///
/// `Read` `S_FALSE` dönebiliyor (akış bitti) ve bu bir **hata değil**; bu
/// yüzden `HRESULT` `ok()` ile değil elle inceleniyor. Okunan bayt sayısı
/// sıfıra düştüğünde bitiyoruz.
fn akisi_oku(akis: &IStream) -> Option<Vec<u8>> {
    let mut cikti: Vec<u8> = Vec::new();
    let mut tampon = [0u8; 8192];
    loop {
        let mut okunan: u32 = 0;
        // SAFETY: `tampon` yığında geçerli ve `okunan` yazılabilir; ikisi de
        // çağrı boyunca yaşıyor.
        let hr = unsafe {
            akis.Read(
                tampon.as_mut_ptr() as *mut std::ffi::c_void,
                tampon.len() as u32,
                Some(&mut okunan),
            )
        };
        if hr.is_err() {
            return None;
        }
        if okunan == 0 {
            break;
        }
        cikti.extend_from_slice(&tampon[..okunan as usize]);
        if cikti.len() > FAVICON_SINIRI {
            // Sınırı aşan favicon **atılıyor**, kırpılmıyor: yarım bir PNG
            // zaten çözülemez ve kırpmak "bozuk ama var" bir kayıt bırakırdı.
            return None;
        }
    }
    (!cikti.is_empty()).then_some(cikti)
}

impl<R: Runtime> Motor<R> for WebView2Motor<R> {
    fn yetenekler(&self) -> Yetenekler {
        *self.yetenekler.lock().unwrap()
    }

    fn dinleyici_ata(&self, dinleyici: Weak<dyn Dinleyici>) {
        *self.dinleyici.lock().unwrap() = Some(dinleyici);
    }

    fn sekme_ac(&self, id: SekmeId, url: &str, gizli: bool) -> Sonuc<()> {
        if self.app.get_webview(&etiket(id)).is_some() {
            return Ok(());
        }
        let pencere = self
            .app
            .get_window(ANA_PENCERE)
            .ok_or_else(|| MuirenHata::Motor("ana pencere yok".into()))?;
        let adres = url::Url::parse(url)?;
        let alan = *self.alan.lock().unwrap();

        let mut olusturucu =
            tauri::webview::WebviewBuilder::new(etiket(id), WebviewUrl::External(adres));

        // Bayraklar sekme webview'ine de veriliyor ve **kabuktakiyle birebir
        // aynı dize** olmak zorunda.
        //
        // Uzun süre "ortam ilk webview'de kuruluyor, sekmelere vermenin etkisi
        // yok" varsayıldı. Yanlış: wry her webview için ayrı bir
        // `CreateCoreWebView2EnvironmentWithOptions` çağırıyor ve WebView2 aynı
        // kullanıcı veri klasörünü paylaşan ikinci ortamı yalnız **seçenekleri
        // aynıysa** yaratıyor; farklıysa `ERROR_INVALID_STATE` (0x8007139F)
        // dönüyor.
        //
        // Bayrak verilmediğinde wry kendi varsayılanını koyuyor ve o varsayılan
        // `--autoplay-policy=no-user-gesture-required` içeriyor; kabuğun
        // dizesi içermiyordu. İki dize ayrışınca sekmenin ortamı kurulamıyor,
        // hata `tauri-runtime-wry` içinde `log::error!` ile yutuluyor ve
        // `add_child` yine de `Ok` dönüyordu. Görünen tablo: pencerede HWND
        // var (wry kabı ortamdan ÖNCE yaratıyor), sekme kaydı var, sayfa yok.
        // Faz 1'i bloke eden sorun buydu (`docs/Roadmap.md`).
        //
        // Bu yüzden dize `settings`ten her seferinde yeniden hesaplanmıyor,
        // açılışta dondurulup burada okunuyor.
        if !self.bayraklar.is_empty() {
            olusturucu = olusturucu.additional_browser_args(&self.bayraklar);
        }

        // Gizli sekme: `ICoreWebView2ControllerOptions::SetIsInPrivateModeEnabled`.
        //
        // Bunun bir **denetleyici** seçeneği olması kritik. Yukarıdaki uzun
        // notun anlattığı tuzak ortam seçenekleriyle ilgili: bayrak dizesi ya
        // da kullanıcı veri klasörü ayrışırsa webview sessizce ölü doğuyor.
        // InPrivate o kümede değil — wry onu
        // `CreateCoreWebView2ControllerWithOptions` üzerinden geçiriyor ve
        // ortam aynı kalıyor (`wry/src/webview2/mod.rs`, `create_controller`).
        // Yani gizli sekme için ayrı bir veri klasörü **açmıyoruz**; açsaydık
        // ikinci bir ortam kurulur ve Faz 1'i bloke eden hataya davetiye
        // çıkarırdı.
        if gizli {
            olusturucu = olusturucu.incognito(true);
        }

        let webview = pencere
            .add_child(
                olusturucu,
                LogicalPosition::new(alan.x, alan.y),
                LogicalSize::new(alan.genislik, alan.yukseklik),
            )
            .map_err(|e| MuirenHata::Motor(e.to_string()))?;

        // Webview GİZLİ doğuyor: arka planda açılan bir sekme, kullanıcının
        // baktığı sayfanın üstünü kaplamamalı.
        let _ = webview.hide();

        if !self.yetenekler().askiya_alma {
            self.yetenekleri_olc(&webview);
        }
        self.olaylari_bagla(&webview, id);
        Ok(())
    }

    fn sekme_kapat(&self, id: SekmeId) -> Sonuc<()> {
        self.at(id)
    }

    fn gezin(&self, id: SekmeId, url: &str) -> Sonuc<()> {
        let adres = url::Url::parse(url)?;
        self.webview(id)?
            .navigate(adres)
            .map_err(|e| MuirenHata::Motor(e.to_string()))
    }

    fn geri(&self, id: SekmeId) -> Sonuc<()> {
        self.ham(id, |core| unsafe {
            let _ = core.GoBack();
        })
    }

    fn ileri(&self, id: SekmeId) -> Sonuc<()> {
        self.ham(id, |core| unsafe {
            let _ = core.GoForward();
        })
    }

    fn yenile(&self, id: SekmeId) -> Sonuc<()> {
        self.ham(id, |core| unsafe {
            let _ = core.Reload();
        })
    }

    fn durdur(&self, id: SekmeId) -> Sonuc<()> {
        self.ham(id, |core| unsafe {
            let _ = core.Stop();
        })
    }

    fn alan_ayarla(&self, dikdortgen: Dikdortgen) -> Sonuc<()> {
        *self.alan.lock().unwrap() = dikdortgen;
        // Bütün sekme webview'leri aynı dikdörtgeni paylaşıyor; görünür olan
        // biri, gerisi gizli. Hepsini birden taşımak, gizli birini
        // gösterirken ikinci bir yerleştirme turu gerektirmiyor.
        for (etiket_adi, webview) in self.app.webviews() {
            if etiket_adi.starts_with(super::SEKME_ONEKI) {
                let _ = webview.set_bounds(sinirlar(dikdortgen));
            }
        }
        Ok(())
    }

    fn goster(&self, id: SekmeId, dikdortgen: Dikdortgen) -> Sonuc<()> {
        let webview = self.webview(id)?;
        webview
            .set_bounds(sinirlar(dikdortgen))
            .map_err(|e| MuirenHata::Motor(e.to_string()))?;
        webview
            .show()
            .map_err(|e| MuirenHata::Motor(e.to_string()))?;
        let _ = webview.set_focus();
        Ok(())
    }

    fn kabuk_odakla(&self) -> Sonuc<()> {
        let kabuk = self
            .app
            .get_webview(super::KABUK)
            .ok_or_else(|| MuirenHata::Motor("kabuk webview yok".into()))?;
        kabuk
            .set_focus()
            .map_err(|e| MuirenHata::Motor(e.to_string()))
    }

    fn gizle(&self, id: SekmeId) -> Sonuc<()> {
        // Kapanmış bir sekmeyi gizlemek hata değil: gözcü ile arayüz arasında
        // gecikme var.
        if let Ok(w) = self.webview(id) {
            let _ = w.hide();
        }
        Ok(())
    }

    fn ses_kes(&self, id: SekmeId, sessiz: bool) -> Sonuc<()> {
        self.ham(id, move |core| unsafe {
            if let Ok(v8) = core.cast::<ICoreWebView2_8>() {
                let _ = v8.SetIsMuted(sessiz);
            }
        })
    }

    fn uyut(&self, id: SekmeId) -> Sonuc<()> {
        if !self.yetenekler().askiya_alma {
            return Err(MuirenHata::YetenekYok("TrySuspend".into()));
        }
        self.ham(id, |core| unsafe {
            let Ok(v3) = core.cast::<ICoreWebView2_3>() else {
                return;
            };
            // `TrySuspend` yalnız GÖRÜNMEYEN bir webview'de çalışıyor; sürücü
            // bu çağrıdan önce `gizle` diyor.
            let _ = v3.TrySuspend(&TrySuspendCompletedHandler::create(Box::new(
                |_sonuc, _basarili| Ok(()),
            )));
        })
    }

    fn uyandir(&self, id: SekmeId) -> Sonuc<()> {
        self.ham(id, |core| unsafe {
            if let Ok(v3) = core.cast::<ICoreWebView2_3>() {
                let _ = v3.Resume();
            }
        })
    }

    fn bellek_hedefi(&self, id: SekmeId, seviye: BellekSeviyesi) -> Sonuc<()> {
        if !self.yetenekler().bellek_hedefi {
            return Err(MuirenHata::YetenekYok("SetMemoryUsageTargetLevel".into()));
        }
        let hedef = match seviye {
            BellekSeviyesi::Normal => COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
            BellekSeviyesi::Dusuk => COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
        };
        self.ham(id, move |core| unsafe {
            if let Ok(v19) = core.cast::<ICoreWebView2_19>() {
                let _ = v19.SetMemoryUsageTargetLevel(hedef);
            }
        })
    }

    fn at(&self, id: SekmeId) -> Sonuc<()> {
        // Sekme KAYDI duruyor; yıkılan yalnız webview. Kullanıcı tıklayınca
        // URL ve kaydırma konumundan yeniden doğuyor (`docs/Sekmeler.md`).
        //
        // Çerçeve kaydı da düşüyor: webview'i olmayan sekmenin çerçevesi de
        // yok ve bırakılan kimlik, WebView2 onu başka bir sekmeye yeniden
        // dağıttığında yanlış sekmeye bellek yazdırırdı. Sekmenin **son
        // bilinen bellek değeri** burada silinmiyor — o `esleme.rs`
        // defterinde ve panelin atılmış sekme için gösterdiği tek sayı o.
        self.cerceveler.lock().unwrap().remove(&id);
        if let Some(w) = self.app.get_webview(&etiket(id)) {
            w.close().map_err(|e| MuirenHata::Motor(e.to_string()))?;
        }
        Ok(())
    }

    fn var_mi(&self, id: SekmeId) -> bool {
        self.app.get_webview(&etiket(id)).is_some()
    }

    /// Sayfanın dikey kaydırma **oranını** ister (0.0–1.0).
    ///
    /// Piksel değil oran: atılmış sekme geri geldiğinde pencere boyutu ya da
    /// sayfanın kendisi değişmiş olabiliyor; 4200 piksel o sayfada artık
    /// yok olabilir ama "%38" her zaman bir yere denk geliyor.
    ///
    /// `ExecuteScript` sayfanın JS bağlamında koşuyor ama bu bir **ayrıcalıklı
    /// kanal değil**: biz sayfaya soruyoruz, sayfa bize bir şey gönderemiyor
    /// (`docs/IPC.md`). Dönen değer JSON sayı; ayrıştırma başarısızsa hiçbir
    /// şey yapılmıyor.
    fn kaydirma_iste(&self, id: SekmeId) -> Sonuc<()> {
        let Some(dinleyici) = self.dinleyici.lock().unwrap().clone() else {
            return Ok(());
        };
        self.ham(id, move |core| unsafe {
            let betik = windows::core::HSTRING::from(
                // `scrollHeight - clientHeight` sıfır olabilir (sayfa
                // kaydırılmıyor); sıfıra bölmek NaN üretip JSON'u bozardı.
                "(function(){var e=document.documentElement||document.body;\
                 var t=e.scrollHeight-e.clientHeight;\
                 return t>0?(e.scrollTop||window.scrollY||0)/t:0;})()",
            );
            let _ = core.ExecuteScript(
                windows::core::PCWSTR(betik.as_ptr()),
                &ExecuteScriptCompletedHandler::create(Box::new(move |hr, sonuc| {
                    if hr.is_err() {
                        return Ok(());
                    }
                    // `sonuc` JSON: sayfanın döndürdüğü sayı. Ayrıştırma
                    // başarısızsa hiçbir şey yapılmıyor — sayfanın
                    // gönderdiği metne güvenilmiyor (CLAUDE.md #8).
                    if let (Ok(oran), Some(d)) = (sonuc.trim().parse::<f32>(), dinleyici.upgrade())
                    {
                        if oran.is_finite() {
                            d.kaydirma_okundu(id, oran.clamp(0.0, 1.0));
                        }
                    }
                    Ok(())
                })),
            );
        })
    }

    /// Veri temizleme, **var olan herhangi bir** sekme webview'i üzerinden
    /// yapılıyor.
    ///
    /// Sebebi: profil webview'e değil ortama ait ve `ICoreWebView2_13::Profile`
    /// bir `ICoreWebView2` üzerinden alınıyor. Hangi sekmeden alındığı fark
    /// etmiyor; hepsi aynı profili paylaşıyor (gizli sekmeler hariç — onların
    /// verisi zaten bellekte ve kapanınca gidiyor).
    ///
    /// Hiç webview yoksa (bütün sekmeler atılmış) silinecek bir oturum da
    /// çoğunlukla yok; yine de sessiz kalmıyoruz, çünkü kullanıcı "sildim"
    /// sanıp silinmemiş bir çerezle dolaşmamalı.
    fn veri_temizle(&self, kalemler: super::MotorTemizligi) -> Sonuc<()> {
        if kalemler.bos_mu() {
            return Ok(());
        }

        let mut turler = 0u32;
        if kalemler.cerezler {
            turler |= COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES.0 as u32;
        }
        if kalemler.onbellek {
            turler |= COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE.0 as u32;
        }
        if kalemler.site_verisi {
            // `ALL_DOM_STORAGE` localStorage, IndexedDB, WebSQL, CacheStorage
            // ve dosya sistemini birlikte kapsıyor; `SERVICE_WORKERS` ayrıca
            // ekleniyor çünkü onlar olmadan site çevrimdışı kopyasıyla
            // açılmaya devam ediyor ve kullanıcı "silinmedi" diyor.
            turler |= COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_DOM_STORAGE.0 as u32;
            turler |= COREWEBVIEW2_BROWSING_DATA_KINDS_SERVICE_WORKERS.0 as u32;
        }
        if kalemler.otomatik_doldurma {
            // `PASSWORD_AUTOSAVE` **bilerek yok**: Muiren şifre saklamıyor
            // (CLAUDE.md kapsam dışı) ve kullanıcının başka bir yöneticiye
            // ait verisini silmek bizim işimiz değil.
            turler |= COREWEBVIEW2_BROWSING_DATA_KINDS_GENERAL_AUTOFILL.0 as u32;
        }

        // Herhangi bir sekme webview'i: profil hepsinde ortak.
        let hedef = self
            .app
            .webviews()
            .into_iter()
            .find(|(etiket_adi, _)| etiket_adi.starts_with(super::SEKME_ONEKI))
            .map(|(_, w)| w);
        let Some(webview) = hedef else {
            return Err(MuirenHata::Motor("açık sekme yok".into()));
        };

        webview
            .with_webview(move |pw| unsafe {
                let Ok(core) = pw.controller().CoreWebView2() else {
                    return;
                };
                let Ok(v13) = core.cast::<ICoreWebView2_13>() else {
                    // Eski Runtime: `Profile` yok. Bir yetenek yokluğu ama
                    // sessiz kalmıyor — kullanıcı sildiğini sanmamalı.
                    log::error!("muiren: veri temizlenemedi — ICoreWebView2_13 yok");
                    return;
                };
                let Ok(profil) = v13.Profile() else { return };
                let Ok(profil2) = profil.cast::<ICoreWebView2Profile2>() else {
                    log::error!("muiren: veri temizlenemedi — ICoreWebView2Profile2 yok");
                    return;
                };
                let sonuc = profil2.ClearBrowsingData(
                    COREWEBVIEW2_BROWSING_DATA_KINDS(turler as i32),
                    &ClearBrowsingDataCompletedHandler::create(Box::new(|hr| {
                        if hr.is_err() {
                            log::error!("muiren: veri temizleme başarısız: {hr:?}");
                        }
                        Ok(())
                    })),
                );
                if let Err(e) = sonuc {
                    log::error!("muiren: veri temizleme çağrısı başarısız: {e}");
                }
            })
            .map_err(|e| MuirenHata::Motor(e.to_string()))
    }

    fn kaydirma_uygula(&self, id: SekmeId, oran: f32) -> Sonuc<()> {
        if !(0.0..=1.0).contains(&oran) || oran <= 0.0 {
            // Sayfanın başı zaten varsayılan; boşuna betik çalıştırmıyoruz.
            return Ok(());
        }
        self.ham(id, move |core| unsafe {
            let betik = windows::core::HSTRING::from(format!(
                "(function(){{var e=document.documentElement||document.body;\
                 var t=e.scrollHeight-e.clientHeight;\
                 if(t>0)window.scrollTo(0,t*{oran});}})()"
            ));
            let _ = core.ExecuteScript(
                windows::core::PCWSTR(betik.as_ptr()),
                &ExecuteScriptCompletedHandler::create(Box::new(|_, _| Ok(()))),
            );
        })
    }

    /// Süreç → sekme eşlemesini ister.
    ///
    /// Çağrı **ortam** düzeyinde: sayfaya inmiyor, betik çalıştırmıyor,
    /// uyuyan bir sekmeyi uyandırmıyor (CLAUDE.md #5).
    ///
    /// Ortama **kabuk** webview'i üzerinden ulaşılıyor, herhangi bir sekme
    /// üzerinden değil — `veri_temizle`den ayrıldığı yer burası ve sebebi
    /// sıklık: bu çağrı gözcünün her turunda tekrarlanıyor ve seçilecek
    /// sekme uyuyan bir sekme olabilir. Kabuk hiç uyutulmuyor. Ortam
    /// paylaşımı zaten garanti: bütün webview'ler aynı bayrak dizesini ve
    /// aynı kullanıcı veri klasörünü kullanıyor (CLAUDE.md #16), yani tek
    /// bir tarayıcı sürecini paylaşıyorlar ve `GetProcessExtendedInfos` o
    /// sürecin tamamını sayıyor.
    fn surec_bilgisi_iste(&self) -> Sonuc<()> {
        let Some(webview) = self.app.get_webview(super::KABUK) else {
            // Kabuk henüz doğmadı (açılışın ilk anları). Eski anlık görüntü
            // **temizleniyor** — duran bir liste, artık var olmayan
            // süreçleri sekmelere yazdırırdı.
            self.surecler.lock().unwrap().clear();
            return Ok(());
        };

        let cerceveler = self.cerceveler.clone();
        let surecler = self.surecler.clone();
        webview
            .with_webview(move |pw| unsafe {
                let Ok(core) = pw.controller().CoreWebView2() else {
                    return;
                };
                let ortam = core
                    .cast::<ICoreWebView2_2>()
                    .and_then(|v2| v2.Environment())
                    .and_then(|o| o.cast::<ICoreWebView2Environment13>());
                let Ok(ortam) = ortam else {
                    // Eski Runtime. Yetenek zaten kapalı ve panel "yaklaşık"
                    // diyor; her turda bir hata satırı yazmanın karşılığı yok.
                    return;
                };

                // Ters tablo **burada**, geri çağrının içinde değil:
                // kilit ana iş parçacığında ve kısa tutuluyor.
                let ters: HashMap<u32, SekmeId> = cerceveler
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|(sekme, cerceve)| (*cerceve, *sekme))
                    .collect();

                // Kabuğun kendi çerçevesi. Ayrıca tutulmasının sebebi somut:
                // kabuk da bir render sürecinde koşuyor ve o süreç listeye
                // giriyor. Tanınmasaydı her turda "sahibi bulunamayan render
                // süreci" sayılır, eşleme **hiçbir zaman** tam çıkmaz ve
                // `olcum_yaklasik` kalıcı olarak true kalırdı.
                let mut kabuk_cerceve = 0u32;
                if let Ok(v20) = core.cast::<ICoreWebView2_20>() {
                    let _ = v20.FrameId(&mut kabuk_cerceve);
                }

                let sonuc = ortam.GetProcessExtendedInfos(
                    &GetProcessExtendedInfosCompletedHandler::create(Box::new(
                        move |hr, koleksiyon| {
                            if hr.is_err() {
                                return Ok(());
                            }
                            let Some(koleksiyon) = koleksiyon else {
                                return Ok(());
                            };
                            *surecler.lock().unwrap() =
                                surecleri_topla(&koleksiyon, &ters, kabuk_cerceve);
                            Ok(())
                        },
                    )),
                );
                if let Err(e) = sonuc {
                    log::error!("muiren: süreç bilgisi istenemedi: {e}");
                }
            })
            .map_err(|e| MuirenHata::Motor(e.to_string()))
    }

    fn surec_haritasi(&self) -> Vec<SurecKaydi> {
        self.surecler.lock().unwrap().clone()
    }

    /// `GetAvailableCoreWebView2BrowserVersionString` — kurulu çalışma
    /// zamanının sürümü.
    ///
    /// Serbest bir fonksiyon: bir webview, bir denetleyici, hatta bir ortam
    /// bile gerektirmiyor. Bu yüzden ölçüm modu raporu yazarken hiçbir sekmeye
    /// dokunmadan sorabiliyor (CLAUDE.md #5).
    fn calisma_zamani_surumu(&self) -> Option<String> {
        // SAFETY: çıktı işaretçisi başarı durumunda `take_pwstr` ile
        // devralınıp serbest bırakılıyor; hata durumunda null kalıyor.
        unsafe {
            let mut ham = windows::core::PWSTR::null();
            if GetAvailableCoreWebView2BrowserVersionString(windows::core::PCWSTR::null(), &mut ham)
                .is_err()
                || ham.is_null()
            {
                return None;
            }
            let dize = take_pwstr(ham);
            if dize.trim().is_empty() {
                None
            } else {
                Some(dize)
            }
        }
    }
}

/// `GetProcessExtendedInfos` sonucunu [`SurecKaydi`] listesine çevirir.
///
/// Burada **hesap yok**, çeviri var: bellek rakamı bu listeye hiç girmiyor,
/// birleştirmeyi saf taraf yapıyor (`memory/esleme.rs`).
///
/// # Safety
/// Ana iş parçacığında, geçerli bir koleksiyonla çağrılıyor.
unsafe fn surecleri_topla(
    koleksiyon: &ICoreWebView2ProcessExtendedInfoCollection,
    ters: &HashMap<u32, SekmeId>,
    kabuk_cerceve: u32,
) -> Vec<SurecKaydi> {
    let mut sayi = 0u32;
    if koleksiyon.Count(&mut sayi).is_err() {
        return Vec::new();
    }

    let mut kayitlar = Vec::with_capacity(sayi as usize);
    for i in 0..sayi {
        let Ok(bilgi) = koleksiyon.GetValueAtIndex(i) else {
            continue;
        };
        let Ok(surec) = bilgi.ProcessInfo() else {
            continue;
        };
        let mut pid = 0i32;
        if surec.ProcessId(&mut pid).is_err() || pid <= 0 {
            continue;
        }
        let mut tur = Default::default();
        let render = surec.Kind(&mut tur).is_ok() && tur == COREWEBVIEW2_PROCESS_KIND_RENDERER;

        let mut sekmeler: Vec<SekmeId> = Vec::new();
        let mut kabuk = false;
        if render {
            if let Ok(cerceveler) = bilgi.AssociatedFrameInfos() {
                if let Ok(gezgin) = cerceveler.GetIterator() {
                    let mut var = windows::core::BOOL(0);
                    while gezgin.HasCurrent(&mut var).is_ok() && var.as_bool() {
                        if let Ok(cerceve) = gezgin.GetCurrent() {
                            match cerceve_sahibi(&cerceve, ters, kabuk_cerceve) {
                                // Aynı sekmenin birden çok çerçevesi aynı
                                // süreçte olabilir; sekmeyi iki kez yazmak
                                // `esleme.rs` içindeki payı ikiye bölerdi.
                                Some(Sahip::Sekme(sekme)) => {
                                    if !sekmeler.contains(&sekme) {
                                        sekmeler.push(sekme);
                                    }
                                }
                                Some(Sahip::Kabuk) => kabuk = true,
                                None => {}
                            }
                        }
                        let mut ilerledi = windows::core::BOOL(0);
                        if gezgin.MoveNext(&mut ilerledi).is_err() || !ilerledi.as_bool() {
                            break;
                        }
                    }
                }
            }
        }

        kayitlar.push(SurecKaydi {
            pid: pid as u32,
            render,
            sekmeler,
            kabuk,
        });
    }
    kayitlar
}

/// Bir çerçevenin sahibi.
///
/// İki tür sahip var ve ikisi de "tanıdım" sayılıyor — ayrımı yapan tek şey
/// bulunan belleğin nereye yazılacağı: sekmeye mi, ortak gidere mi
/// (`memory/esleme.rs`).
enum Sahip {
    Sekme(SekmeId),
    /// Kabuğun kendi arayüzü. Sekme değil, Muiren'in kendi maliyeti.
    Kabuk,
}

/// Bir çerçevenin ait olduğu sekme — ya da kabuk.
///
/// Alt çerçeveler (iframe) tabloda yok — orada yalnız sekmelerin **ana**
/// çerçeveleri var. Bu yüzden zincir yukarı yürütülüyor: gömülü bir
/// oynatıcının ayrı süreci, onu gösteren sayfanın maliyetinin parçası ve
/// "tanımadım" deyip ortak gidere yazmak, site izolasyonu açıkken sekme
/// başına rakamı sistematik olarak eksik gösterirdi.
///
/// Kabuğun çerçevesi de aynı zincirde aranıyor: kabuğun arayüzü de bir render
/// sürecinde koşuyor ve tanınmasaydı o süreç her turda "sahipsiz" sayılır,
/// eşleme hiçbir zaman tam çıkmazdı (`docs/Bellek.md`).
///
/// # Safety
/// Ana iş parçacığında, geçerli bir çerçeve bilgisiyle çağrılıyor.
unsafe fn cerceve_sahibi(
    cerceve: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2FrameInfo,
    ters: &HashMap<u32, SekmeId>,
    kabuk_cerceve: u32,
) -> Option<Sahip> {
    let mut su = cerceve.cast::<ICoreWebView2FrameInfo2>().ok()?;
    // Derinlik sınırlı: bozuk bir zincirde döngüye girmemek için. İç içe
    // yirmi çerçeve gerçek bir sayfada da görülmüyor.
    for _ in 0..20 {
        let mut kimlik = 0u32;
        if su.FrameId(&mut kimlik).is_ok() {
            if let Some(sekme) = ters.get(&kimlik) {
                return Some(Sahip::Sekme(*sekme));
            }
            // Sıfır kontrolü şart: `FrameId` alınamadığında `kabuk_cerceve`
            // sıfır kalıyor ve sıfırla eşleşen her çerçeve kabuk sanılırdı.
            if kabuk_cerceve != 0 && kimlik == kabuk_cerceve {
                return Some(Sahip::Kabuk);
            }
        }
        let Ok(ust) = su.ParentFrameInfo() else {
            return None; // üst düzey çerçeve: zincirin sonu
        };
        su = ust.cast::<ICoreWebView2FrameInfo2>().ok()?;
    }
    None
}
