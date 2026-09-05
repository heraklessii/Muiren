//! Gözcü — periyodik döngü.
//!
//! Kendi iş parçacığında koşuyor ve **sebebi somut**: politikanın çalışması
//! gereken asıl an, arayüzün kapalı olduğu an. Kullanıcı oyunda ya da pencere
//! simge durumundayken 40 sekmenin uyanık beklemesinin hiçbir karşılığı yok.
//! Arayüzün çizilmiş olup olmaması bu kararı etkilemiyor (CLAUDE.md, "karar
//! veren kod backend'de").
//!
//! Buradaki iş sırası:
//!
//! ```text
//! her N saniyede:
//!     baski   = olcum::sistem()          → esik::BellekBaskisi
//!     ozetler = surucu.sekme_listesi()   → kilit kısa tutuluyor
//!     eylemler = esik::karar(...)        → SAF
//!     her eylem: surucu'ya uygula
//!     bellek özetini yayınla
//! ```
//!
//! Karar burada **verilmiyor**, `esik.rs` veriyor. Buranın işi ölçmek,
//! sormak ve uygulamak.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tauri::{Emitter, Manager, Runtime};

use super::esik::{self, BellekAyarlari, BellekBaskisi, Eylem};
use super::{olcum, BellekOzeti};
use crate::bridge::muifly::Kaynak;
use crate::olaylar;
use crate::tabs::durum::{Durum, Olay, Sebep};
use crate::tabs::surucu::Surucu;

/// Oyun modunda uyanık sekme üst sınırı (`docs/Bellek.md`, "Oyun modu").
const OYUN_UYANIK_SINIRI: u32 = 2;

/// İlk turun beklemesi.
///
/// Döngü **önce uyuyup sonra ölçtüğü** için ilk özet ancak bir periyot sonra
/// (varsayılan 20 sn) çıkıyordu. Bellek göstergesi artık gezinme çubuğunda
/// sürekli duruyor (`src/components/Nabiz.tsx`) ve o boşluk, açılıştan sonra
/// yirmi saniye boyunca **eksik bir gösterge** olarak görünüyor: kullanıcının
/// gördüğü şey "ölçülüyor" değil, hiçbir şey.
///
/// Politika açısından bu turu öne almak zararsız: açılışta oturumdan gelen
/// sekmelerin hepsi `Atilmis` doğuyor (`tabs/oturum.rs`) ve boşta kalma
/// süreleri sıfırdan başlıyor, yani eşik hiçbir eylem üretmiyor. Turun tek
/// yaptığı ölçüp yayınlamak.
const ILK_BEKLEME_SN: u64 = 3;

/// Gözcü iş parçacığını başlatır.
///
/// Sürücüyü **zayıf** tutuyor: güçlü tutsaydı uygulama kapanırken sürücü
/// düşmez ve süreç sonlanmazdı.
pub fn baslat<R: Runtime>(surucu: &Arc<Surucu<R>>) {
    let zayif = Arc::downgrade(surucu);
    let sonuc = std::thread::Builder::new()
        .name("muiren-gozcu".into())
        .spawn(move || {
            let mut ilk = true;
            loop {
                // Periyot her turda yeniden okunuyor: kullanıcı ayarlardan
                // değiştirdiğinde bir sonraki tur yeni değerle uyuyor.
                let periyot = match zayif.upgrade() {
                    Some(s) => {
                        let p = s.ayarlar().gozcu_periyodu_sn.max(1);
                        // Pencere simge durumundayken periyot kısalıyor
                        // (`docs/Bellek.md`, "Gözcü"). Bu turun bedeli de aynı
                        // anda düşüyor: gizliyken süreç tablosu hiç taranmıyor
                        // (`yayinla_ozet`), yani daha sık koşan tur daha ucuz bir
                        // tur. Tersi olsaydı — iki katı sıklıkta tam ölçüm —
                        // bedeli tam da kullanıcının oyunda olduğu ana denk
                        // gelirdi.
                        if s.pencere_gizli() {
                            (p / 2).max(1)
                        } else {
                            p
                        }
                    }
                    None => return, // sürücü düştü: uygulama kapanıyor
                };
                // İlk tur öne alınıyor; sonrakiler ayarın periyodunda.
                let bekleme = if ilk {
                    ilk = false;
                    periyot.min(ILK_BEKLEME_SN)
                } else {
                    periyot
                };
                std::thread::sleep(Duration::from_secs(bekleme));

                let Some(surucu) = zayif.upgrade() else {
                    return;
                };
                tur(&surucu);
            }
        });

    if let Err(e) = sonuc {
        // Gözcü kurulamazsa tarayıcı çalışmaya devam ediyor ama tezini
        // yerine getirmiyor; bu sessiz kalmamalı.
        log::error!("muiren: bellek gözcüsü başlatılamadı: {e}");
    }
}

/// Bir tur: ölç, sor, uygula, yayınla.
///
/// `pub` çünkü elle tetiklenebiliyor (`hepsini_uyut` sonrası panelin
/// tazelenmesi) ve testten çağrılabilir olması iyi.
pub fn tur<R: Runtime>(surucu: &Surucu<R>) {
    let ozetler = surucu.sekme_listesi();
    let ayarlar = surucu.ayarlar();
    let yetenekler = surucu.yetenekler();
    let oyun_modu = surucu.oyun_modu();

    // **Denetim.** Oyun modu kapalıyken bizim küçülttüğümüz pencere küçük
    // kalmamalı. Normal yolda geri getirme `oyun_modu(false)` içinde oluyor;
    // bu satır o yolun düştüğü hâli kapatıyor (algılama iş parçacığı ölürse,
    // bayrak ile mod bir sebeple ayrışırsa). Kullanıcının penceresini
    // kendiliğinden küçülten bir özelliğin geri getirmesi garantili olmak
    // zorunda (`docs/Bellek.md`, "Oyun modu" 4. adım).
    if !oyun_modu {
        oyun_penceresi(surucu, false);
    }
    let pencere_gizli = surucu.pencere_gizli();

    let sistem = olcum::sistem();
    // Ölçülen baskı + kullanıcının o anki durumu. Birleştirme **saf tarafta**
    // (`esik::BellekBaskisi::etkin`) çünkü `max` yerine atama yazmak, bellek
    // gerçekten tükenmişken pencereyi küçültenin politikasını gevşetirdi ve
    // bunu ancak bir test yakalar.
    //
    // Ölçülen değer ayrıca tutuluyor: karara **etkin** baskı giriyor ama
    // kullanıcıya gösterilen sebep ölçülene bakıyor. İkisini tek değişkende
    // birleştirseydik, pencereyi küçülten kullanıcı sekmelerinin yanında
    // "sistem baskısı" rozetini görür ve makinesinde olmayan bir sorun
    // arardı.
    let olculen = BellekBaskisi::yuzdeden(sistem.bos_yuzde());
    let baski = BellekBaskisi::etkin(olculen, oyun_modu, pencere_gizli);

    let mut bellek_ayarlari = BellekAyarlari::ayarlardan(&ayarlar);
    bellek_ayarlari.askiya_alma_var = yetenekler.askiya_alma;
    bellek_ayarlari.bellek_hedefi_var = yetenekler.bellek_hedefi;
    // Koruma kuralı #7 (`docs/Kopruler.md`). Liste turun başında bir kez
    // kopyalanıyor: `esik.rs` saf kalmak zorunda ve oraya bir kilit
    // geçirilemez.
    bellek_ayarlari.muiwatch_sekmeleri = surucu.muiwatch_sekmeleri();

    // Baskının sertleşmesi yukarıda, `BellekBaskisi::etkin` içinde. Uyanık üst
    // sınırı oyun moduna **özel**: gizli pencerede düşürülmüyor, çünkü aradaki
    // fark kullanıcının ne söylediği. "Oyundayım" bir talep; simge durumuna
    // almak "şu an bakmıyorum" demek, "sekmelerim gitsin" demek değil.
    if oyun_modu {
        bellek_ayarlari.uyanik_ust_sinir = OYUN_UYANIK_SINIRI;
    }

    let eylemler = esik::karar(&ozetler, baski, &bellek_ayarlari);
    // Sıra önemli: **gerçek** baskı, gizli pencereden önce geliyor. İkisi
    // birdense kullanıcının bilmesi gereken şey makinesinin belleğinin
    // dolduğu; pencereyi küçültmüş olması onun yanında ikincil bir ayrıntı.
    let sebep = if oyun_modu {
        Sebep::OyunModu
    } else if olculen >= BellekBaskisi::Yuksek {
        Sebep::SistemBaskisi
    } else if pencere_gizli {
        Sebep::PencereGizli
    } else {
        Sebep::BostaKaldi
    };

    for eylem in eylemler {
        uygula(surucu, eylem, sebep);
    }

    if pencere_gizli {
        // Süreç tablosu **taranmıyor** ve özet yayınlanmıyor: bellek panelini
        // görmeyen bir kullanıcı için Toolhelp32 anlık görüntüsü almanın
        // karşılığı yok. Panelin son bildiği değer yerinde kalıyor ve pencere
        // geri geldiğinde `pencere_gizli` bir tur koşturup tazeliyor.
        //
        // Duran şey **gösterim**, politika değil: karar yukarıda zaten
        // verildi ve uygulandı. Atlanan iki iş de yalnız paneli besliyordu
        // (süreç ölçümü ve olay yayını); eşik kararının ikisine de ihtiyacı
        // yok — girdisi boşta kalma süresi ve sistem baskısı
        // (`docs/Bellek.md`, "Ölçüm").
        return;
    }

    yayinla_ozet(surucu, baski, sistem);
}

/// Tek bir eylemi uygular.
///
/// Uyutma ve atma `durum_degistir` kapısından geçiyor — kullanıcı elle
/// uyuttuğunda da aynı kapı. İki kopya olsaydı biri durum makinesini,
/// diğeri olay yayınını atlardı.
fn uygula<R: Runtime>(surucu: &Surucu<R>, eylem: Eylem, sebep: Sebep) {
    match eylem {
        Eylem::Uyut(id) => {
            if surucu.durum_degistir(id, Olay::Uyutuldu, sebep).is_ok() {
                // Uyuyan sekmenin "hedefi düşürüldü" kaydı anlamını yitirdi:
                // uyandığında yeniden düşürülebilmeli.
                surucu.hedef_dusurme_unut(id);
            }
        }
        Eylem::At(id) => {
            let _ = surucu.durum_degistir(id, Olay::Atildi, sebep);
            surucu.hedef_dusurme_unut(id);
        }
        Eylem::HedefDusur(id) => {
            // Aynı sekmeye her turda yeniden göndermenin karşılığı yok;
            // tekrarı eleyen durum burada, `esik.rs` saf kalsın diye.
            if surucu.hedef_dusurulsun_mu(id) {
                let _ = surucu.bellek_hedefi_dusur(id);
            }
        }
        Eylem::Uyandir(id) => {
            // `esik.rs` bugün bu eylemi **üretmiyor** (gerekçesi orada). Yine
            // de gövde doğru: `Resume` çağrısı `durum_degistir` kapısının
            // içinde, uyutma/atma ile aynı yerde.
            let _ = surucu.durum_degistir(id, Olay::Etkinlestirildi, Sebep::Etkilesim);
        }
    }
}

/// Bellek özetini hesaplayıp yayınlar.
fn yayinla_ozet<R: Runtime>(surucu: &Surucu<R>, baski: BellekBaskisi, sistem: olcum::SistemBellek) {
    // Liste eylemler UYGULANDIKTAN sonra yeniden okunuyor: panelin gösterdiği
    // sayılar bu turun sonucunu içersin.
    let ozetler = surucu.sekme_listesi();
    let surecler = olcum::surecler();

    // Süreç → sekme eşlemesi. Motorun anlık görüntüsü **bir tur geriden**
    // gelebiliyor (`Motor::surec_bilgisi_iste` ateşle-unut) ve bu kabul
    // edilmiş: süreç tablosu 10 saniyede kendini tekrarlıyor, gözcüyü
    // WebView2'nin döngüsüne bağlamanın bedeli ise her turda ödeniyor olurdu.
    let harita = surucu.surec_haritasi();
    let durumlar: Vec<(crate::tabs::SekmeId, bool)> =
        ozetler.iter().map(|o| (o.id, o.durum.uyanik())).collect();
    let esleme = surucu
        .bellek_defteri()
        .lock()
        .unwrap()
        .tur(&harita, &surecler.pid_mb, &durumlar);

    let mut etkin = 0;
    let mut arkaplan = 0;
    let mut uyuyan = 0;
    let mut atilmis = 0;
    for o in &ozetler {
        match o.durum {
            Durum::Etkin => etkin += 1,
            Durum::Arkaplan => arkaplan += 1,
            Durum::Uyuyan => uyuyan += 1,
            Durum::Atilmis => atilmis += 1,
        }
    }

    let ozet = BellekOzeti {
        baski,
        toplam_mb: surecler.webview_mb,
        kabuk_mb: surecler.kabuk_mb,
        sistem_toplam_mb: sistem.toplam_mb,
        sistem_bos_mb: sistem.bos_mb,
        etkin,
        arkaplan,
        uyuyan,
        atilmis,
        tahmini_kazanc_mb: super::kazanc(&esleme, surecler.webview_mb, etkin + arkaplan),
        // İki koşul birden: eşleme tam ve hiçbir pasif sekme ölçümsüz
        // değil. Yarısıyla "kesin" demek, panelin tek büyük rakamını
        // olduğundan güvenilir gösterirdi (`docs/Bellek.md`, "Ölçüm").
        olcum_yaklasik: !(esleme.tam && esleme.kazanc_bilinmeyen == 0),
        sekme_mb: esleme.sekmeler,
        ortak_mb: esleme.ortak_mb,
        surec_sayisi: surecler.surec_sayisi,
    };

    surucu.bellek_ozeti_yaz(ozet.clone());
    let _ = surucu.app().emit(olaylar::BELLEK_OZETI, ozet);

    // Bir sonraki turun eşlemesi **burada** isteniyor, turun başında değil:
    // pencere gizliyken bu fonksiyona hiç gelinmiyor ve o zaman istenmemesi
    // gereken şey tam da bu. Süreç tablosunu taramayan bir tur, motora da
    // soru sormuyor.
    surucu.surec_bilgisi_iste();
}

/// Elle "hepsini uyut": korumalı olmayan her uyanık sekmeyi uyutur.
///
/// Dönüş uyutulan sekme sayısı (`docs/IPC.md`). Karar yine `esik.rs` içindeki
/// koruma kurallarından geçiyor — "hepsini" derken ses çalan sekme de
/// uyutulsaydı kullanıcı müziğini kaybederdi.
pub fn hepsini_uyut<R: Runtime>(surucu: &Surucu<R>) -> u32 {
    let ozetler = surucu.sekme_listesi();
    let ayarlar = surucu.ayarlar();
    let mut bellek_ayarlari = BellekAyarlari::ayarlardan(&ayarlar);
    let yetenekler = surucu.yetenekler();
    bellek_ayarlari.askiya_alma_var = yetenekler.askiya_alma;
    // "Hepsini uyut" derken Muiwatch oturumu da uyusaydı senkron sessizce
    // koparlardı (koruma kuralı #7).
    bellek_ayarlari.muiwatch_sekmeleri = surucu.muiwatch_sekmeleri();

    let mut sayi = 0;
    for o in &ozetler {
        if !o.durum.uyanik() || !esik::uyutulabilir(o, &bellek_ayarlari) {
            continue;
        }
        let olay = if yetenekler.askiya_alma {
            Olay::Uyutuldu
        } else if esik::atilabilir(o, &bellek_ayarlari) {
            Olay::Atildi
        } else {
            continue;
        };
        if surucu.durum_degistir(o.id, olay, Sebep::Elle).is_ok() {
            surucu.hedef_dusurme_unut(o.id);
            sayi += 1;
        }
    }
    sayi
}

/// Oyun modunu açar/kapatır ve hemen bir tur koşturur.
///
/// Oyun bitince sekmeler **kendiliğinden uyandırılmıyor** (`docs/Bellek.md`):
/// kullanıcı hangisine tıklarsa o uyanıyor. Oyundan çıkan birinin 30 sekmesi
/// birden yüklenmeye başlasa, tam da kaçındığımız ani bellek sıçraması olurdu.
/// `kaynak` arayüzde görünüyor (`docs/IPC.md`, `muiren://oyun-modu`): elle
/// açılan modu kullanıcı kendisi kapatır, algılanan modu "yanlış algıladı"
/// diye kapatır ve ikincisi ayarlara gitmesi gereken bir geri bildirim.
/// Pencere simge durumuna alındı ya da geri geldi.
///
/// `oyun_modu` ile aynı kalıp ve aynı iki sebep: durum değişmediyse hiçbir şey
/// yapılmıyor (`Resized` olayı simge durumundayken de tekrar tekrar gelebiliyor
/// ve her seferinde bir tur koşturmanın karşılığı yok), değiştiyse **hemen** bir
/// tur koşuluyor.
///
/// Hemen koşmak, kısaltılmış periyodun asıl işini yapan yer: politikanın
/// pencereyi küçülten kullanıcıyı bir sonraki tura kadar beklemesi için bir
/// sebep yok. Pencere geri geldiğinde de aynı tur, gizliyken atlanan bellek
/// özetini tazeliyor — panel açıldığında eski bir sayı göstermesin.
///
/// **Olay yayınlanmıyor.** Arayüz bunu zaten biliyor (pencereyi kendisi
/// küçülttü) ve simge durumundaki bir pencereye olay göndermenin karşılığı yok.
/// `oyun-modu` olayının var olma sebebi farklıydı: orada kararı **algılama**
/// veriyor ve arayüzün haberi olmuyor.
pub fn pencere_gizli<R: Runtime>(surucu: &Arc<Surucu<R>>, gizli: bool) {
    if surucu
        .pencere_gizli_bayragi()
        .swap(gizli, Ordering::Relaxed)
        == gizli
    {
        return;
    }
    tur(surucu);
}

pub fn oyun_modu<R: Runtime>(surucu: &Arc<Surucu<R>>, acik: bool, kaynak: Kaynak) {
    // Zaten aynı durumdaysak olay yayınlanmıyor: algılama döngüsü saniyede
    // bir aynı cevabı verdiğinde arayüz gereksiz bildirim göstermesin.
    if surucu.oyun_modu_bayragi().swap(acik, Ordering::Relaxed) == acik {
        return;
    }
    let _ = surucu
        .app()
        .emit(olaylar::OYUN_MODU, OyunModuOlayi { acik, kaynak });
    // Pencere turdan ÖNCE küçülüyor: `pencere_gizli` bayrağı bu turun
    // baskısına girsin. Sonra çevrilseydi 4. adımın etkisi bir tur gecikirdi.
    oyun_penceresi(surucu, acik);
    tur(surucu);
}

/// Oyun modunun 4. adımı: kabuk penceresini simge durumuna alır ya da geri
/// getirir (`docs/Bellek.md`, "Oyun modu").
///
/// Üç kural burada duruyor ve üçü de `docs/Bellek.md` içinde gerekçeli:
///
/// - **`minimize()`, `hide()` değil.** Gizlenen pencerenin görev çubuğu
///   düğmesi de kayboluyor; tam ekran bir oyun ekranı kaplarken kullanıcının
///   tarayıcıya dönecek görünür bir yolu kalmıyor.
/// - **Kullanıcının küçülttüğü pencere bizim değil.** Zaten simge
///   durumundaysa bayrak kalkmıyor, dolayısıyla oyun bitince o pencere
///   kendiliğinden geri gelmiyor.
/// - **Geri getirme koşulsuz.** Ayar bu arada kapatılmış olabilir; küçültülmüş
///   bırakılmış bir pencere, ayarı kapatan kullanıcının tek kaybı olurdu.
fn oyun_penceresi<R: Runtime>(surucu: &Surucu<R>, acik: bool) {
    let bayrak = surucu.oyun_kucultu_bayragi();
    if acik && (!surucu.ayarlar().oyun_pencere_gizle || bayrak.load(Ordering::Relaxed)) {
        return;
    }
    if !acik && !bayrak.load(Ordering::Relaxed) {
        // Küçülten biz değiliz: dokunulmuyor.
        return;
    }
    let Some(pencere) = surucu.app().get_window(crate::motor::ANA_PENCERE) else {
        return;
    };

    if acik {
        if pencere.is_minimized().unwrap_or(false) {
            return; // kullanıcı zaten küçültmüş
        }
        if let Err(e) = pencere.minimize() {
            log::warn!("muiren: oyun modunda pencere küçültülemedi: {e}");
            return;
        }
        bayrak.store(true, Ordering::Relaxed);
        // Bayrağı burada çeviriyoruz, `Resized` olayını beklemeden: gözcü
        // bir sonraki satırda baskıyı hesaplıyor ve olay ona yetişmiyor.
        surucu
            .pencere_gizli_bayragi()
            .store(true, Ordering::Relaxed);
    } else {
        if let Err(e) = pencere.unminimize() {
            // Bayrak **duruyor**: bir sonraki tur yeniden deniyor. Sıfırlamak,
            // pencereyi küçültülmüş bırakıp bir daha dönmemek olurdu.
            log::warn!("muiren: oyun modu bitti, pencere geri getirilemedi: {e}");
            return;
        }
        bayrak.store(false, Ordering::Relaxed);
        surucu
            .pencere_gizli_bayragi()
            .store(false, Ordering::Relaxed);
    }
}

#[derive(serde::Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct OyunModuOlayi {
    acik: bool,
    /// `elle` ya da `algilama`.
    ///
    /// `muifly` diye bir kaynak **yok** ve olmayacak: Faz 4'te Muifly'a
    /// bakıldı, `muifly://durum` süreç içi bir Tauri olayı ve süreçler arası
    /// bir uç yok (`bridge/muifly.rs`). Algılama bizim.
    kaynak: Kaynak,
}
