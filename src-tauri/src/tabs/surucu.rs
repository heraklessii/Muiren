//! Sekme sürücüsü — deponun, motorun ve olayların buluştuğu yer.
//!
//! `commands/` buraya ince sarmalayıcılarla bağlanıyor; iş mantığı orada
//! **yok** (CLAUDE.md #1). Sebep somut: aynı karara bellek gözcüsünün
//! döngüsünden de gelinebiliyor ve iki kopya birbirinden sessizce ayrışır.
//!
//! Kararlar burada verilmiyor, **sorulup uygulanıyor**: sıra
//! [`agac`](super::agac), durum [`durum`](super::durum), eşik (Faz 2)
//! `memory/esik.rs`. Bu dosyanın işi kilidi kısa tutmak, motoru çağırmak ve
//! olayı yayınlamak.
//!
//! **Kilit kuralı:** depo kilidi elde tutulurken motor çağrılmıyor. Motor
//! çağrıları ana iş parçacığına iş gönderiyor; kilidi tutarak beklemek arayüzü
//! gözcünün turuna kilitler.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

use super::durum::{Durum, Olay, Sebep};
use super::{
    durum, kabuk_sayfasi, oturum, Depo, Grup, GrupId, GrupRengi, SekmeId, SekmeOzeti, YENI_SEKME,
};
use crate::bridge::{Kopru, KopruDurumu, Kopruler, Yuk};
use crate::engel::{EngelOzeti, EngelSebebi, Engelleyici};
use crate::favicon;
use crate::hata::{MuirenHata, Sonuc};
use crate::history::depo::Depo as GecmisDepo;
use crate::kisayol::Kisayol;
use crate::memory::gecikme::{self, GecikmeOzeti};
use crate::memory::{esleme, gozcu, BellekOzeti};
use crate::motor::{
    Asama, BellekSeviyesi, Dikdortgen, Dinleyici, IndirmeKarari, Kurulu, Motor, SurecKaydi,
    Yetenekler,
};
use crate::olaylar;
use crate::settings::{self, IndirmePolitikasi, Settings};
use crate::theme::{paket, TemaOzeti};

/// Oturum yazımı için bekleme süresi. Sekme açma/kapama/gezinme sırasında
/// saniyede birkaç kez tetiklenebiliyor; her seferinde diske yazmak gereksiz
/// (`docs/Sekmeler.md`: 2 sn debounce).
const OTURUM_BEKLEME: Duration = Duration::from_secs(2);
/// Hiçbir şey değişmese bile güvenlik yazımı.
const OTURUM_GUVENLIK: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DurumOlayi {
    id: SekmeId,
    durum: Durum,
    sebep: Sebep,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GezinmeOlayi {
    id: SekmeId,
    asama: Asama,
    url: String,
    basarili: bool,
}

pub struct Surucu<R: Runtime> {
    app: AppHandle<R>,
    motor: Kurulu<R>,
    depo: Mutex<Depo>,
    ayarlar: Mutex<Settings>,
    /// Arayüzün bildirdiği içerik alanı. Sekme webview'leri buraya
    /// yerleşiyor.
    alan: Mutex<Dikdortgen>,
    oturum_yolu: PathBuf,
    ayarlar_yolu: PathBuf,
    kirli: AtomicBool,

    // --- bellek politikası (Faz 2) ---
    /// Gözcünün baskı altında bellek hedefini düşürdüğü sekmeler.
    ///
    /// Tekrarı eleyen durum burada, `memory/esik.rs` içinde değil: o dosya
    /// saf kalmak zorunda. Sekme uyuduğunda ya da atıldığında kayıt
    /// siliniyor — uyandığında yeniden düşürülebilmeli.
    hedefi_dusurulenler: Mutex<HashSet<SekmeId>>,
    /// Oyun modu. `AtomicBool` çünkü gözcü iş parçacığı her turda okuyor ve
    /// kilit almasının karşılığı yok.
    oyun_modu: AtomicBool,
    /// Pencere simge durumunda mı — yani **kullanıcı tarayıcıya bakmıyor mu**.
    ///
    /// Gözcünün çalışması gereken asıl an bu (`docs/Bellek.md`, "Gözcü"):
    /// kullanıcı bakmıyorsa 14 sekmenin uyanık kalmasının hiçbir karşılığı
    /// yok. `oyun_modu` ile aynı gerekçeyle `AtomicBool`.
    pencere_gizli: AtomicBool,
    /// Pencereyi **oyun modu** küçülttü mü (`docs/Bellek.md`, 4. adım).
    ///
    /// Ayrı tutulmasının sebebi geri getirme: kullanıcının kendi küçülttüğü
    /// pencere, oyun bitince kendiliğinden geri gelmemeli. Bayrak yalnız biz
    /// küçülttüğümüzde kalkıyor ve yalnız biz geri getirdiğimizde iniyor.
    oyun_kucultu: AtomicBool,
    /// Son bellek özeti. `bellek_ozeti` komutu bunu döndürüyor: komut kendi
    /// ölçümünü yapsaydı panel her açılışta süreç tablosunu tarardı.
    son_ozet: Mutex<Option<BellekOzeti>>,
    /// Uyanma gecikmesi defteri (`memory/gecikme.rs`).
    ///
    /// Kronometrenin başladığı/durduğu anlar bu dosyada dağınık duruyor
    /// (uyandırma, gezinme bitişi, betik cevabı, sekme kapanışı) ama hesabın
    /// tamamı deftere ait — `esik.rs` ile aynı ayrım. Buradan geçen tek şey
    /// `Instant::now()`.
    gecikme: Mutex<gecikme::Defter>,
    /// Süreç → sekme bellek defteri (`memory/esleme.rs`).
    ///
    /// Gözcünün turunda değil burada duruyor, çünkü tur `pub` ve birden çok
    /// yerden tetikleniyor (pencere gizlenince, oyun modu değişince, elle).
    /// Defter turun içinde yaşasaydı her tetiklemede sıfırdan doğar ve
    /// atılmış sekmelerin son bilinen değeri kaybolurdu — yani panelin o
    /// sekmeler için gösterebileceği tek sayı.
    bellek_defteri: Mutex<esleme::Defter>,

    // --- geçmiş ve yer imleri (Faz 3) ---
    /// `Option` çünkü veritabanı açılamayabilir (disk dolu, dosya kilitli).
    /// O durumda tarayıcı **çalışmaya devam ediyor**, yalnız geçmiş
    /// tutulmuyor: bozuk bir veritabanı yüzünden açılmayan tarayıcı kabul
    /// edilemez (`docs/Depolama.md`, `oku` asla hata döndürmüyor kuralıyla
    /// aynı gerekçe).
    gecmis: Option<GecmisDepo>,

    // --- temalar (Faz 4) ---
    temalar_dizini: PathBuf,

    // --- köprüler (Faz 3–5) ---
    /// `Arc` çünkü Muifly'ın izleme döngüsü kendi iş parçacığında sürücüden
    /// bağımsız yaşayabilmeli (`bridge::muifly::izlemeyi_baslat` zayıf
    /// referans tutuyor).
    kopruler: Arc<Kopruler>,
    /// Bir kez motorun kendi indirmesine izin verilen adresler.
    ///
    /// `IndirmePolitikasi::Sor` altında akış şu: motor indirmeyi iptal
    /// ediyor, arayüz soruyor, kullanıcı "Muiren indirsin" derse adres
    /// buraya giriyor ve sekme aynı adrese yeniden gönderiliyor. İkinci
    /// `DownloadStarting` bu kümeyi görüp izin veriyor ve **kaydı
    /// siliyor** — kalıcı bir istisna listesi değil, tek atımlık bir bilet.
    /// Kalıcı olsaydı kullanıcının bir kez verdiği izin, aynı adresin
    /// sonraki bütün indirmelerinde sessizce geçerli olurdu.
    indirme_biletleri: Mutex<HashSet<String>>,
    /// Favicon deposunun dizini (`favicon/`).
    favicon_dizini: PathBuf,
    /// Kabukta tam ekran bir örtü açık mı (ayarlar, sekme arama).
    ///
    /// Açıkken sekme webview'i gizli tutuluyor; `goster` çağrıları bu
    /// bayrağa bakıyor, yoksa örtü açıkken gelen bir gezinme sayfayı
    /// örtünün üstüne geri getirirdi.
    ortu_acik: AtomicBool,

    // --- engelleme (Faz 4) ---
    /// Pop-up politikası ve istek filtresi. Ayarlar yazıldığında yeniden
    /// derleniyor (`engel::Engelleyici::guncelle`).
    engelleyici: Engelleyici,
}

impl<R: Runtime> Surucu<R> {
    pub fn yeni(app: AppHandle<R>, motor: Kurulu<R>, veri_dizini: PathBuf) -> Arc<Self> {
        let ayarlar_yolu = veri_dizini.join("ayarlar.json");
        let ayarlar = settings::oku(&ayarlar_yolu);

        let engelleyici = Engelleyici::yeni(&ayarlar);

        let gecmis = match GecmisDepo::ac(&veri_dizini.join("muiren.db")) {
            Ok(d) => Some(d),
            Err(e) => {
                log::error!("muiren: geçmiş veritabanı açılamadı: {e}");
                None
            }
        };

        Arc::new(Surucu {
            app,
            motor,
            depo: Mutex::new(Depo::yeni()),
            ayarlar: Mutex::new(ayarlar),
            alan: Mutex::new(Dikdortgen::BOS),
            oturum_yolu: veri_dizini.join("oturum.json"),
            ayarlar_yolu,
            kirli: AtomicBool::new(false),
            hedefi_dusurulenler: Mutex::new(HashSet::new()),
            oyun_modu: AtomicBool::new(false),
            pencere_gizli: AtomicBool::new(false),
            oyun_kucultu: AtomicBool::new(false),
            son_ozet: Mutex::new(None),
            gecikme: Mutex::new(gecikme::Defter::yeni()),
            bellek_defteri: Mutex::new(esleme::Defter::default()),
            gecmis,
            temalar_dizini: paket::dizin(&veri_dizini),
            kopruler: Arc::new(Kopruler::yeni()),
            indirme_biletleri: Mutex::new(HashSet::new()),
            favicon_dizini: favicon::dizin(&veri_dizini),
            ortu_acik: AtomicBool::new(false),
            engelleyici,
        })
    }

    pub fn motor(&self) -> &Kurulu<R> {
        &self.motor
    }

    pub fn yetenekler(&self) -> Yetenekler {
        self.motor.yetenekler()
    }

    pub fn ayarlar(&self) -> Settings {
        self.ayarlar.lock().unwrap().clone()
    }

    /// Düzeltilmiş ayarları döndürüyor (`docs/IPC.md`).
    pub fn ayarlar_yaz(&self, ham: Settings) -> Sonuc<Settings> {
        let duzeltilmis = settings::duzelt(ham);
        settings::yaz(&self.ayarlar_yolu, &duzeltilmis)?;
        *self.ayarlar.lock().unwrap() = duzeltilmis.clone();
        // Filtre listesi **burada** yeniden derleniyor, her istekte değil:
        // `WebResourceRequested` bir sayfada yüzlerce kez tetikleniyor ve
        // her seferinde metin ayrıştırmak sayfa yüklemesini ayrıştırıcıya
        // bağlardı (`engel/mod.rs`).
        self.engelleyici.guncelle(&duzeltilmis);
        Ok(duzeltilmis)
    }

    /// Engelleme özeti — ayarlar ekranı bunu gösteriyor (kaç kural derlendi,
    /// hangi satırlar anlaşılmadı).
    pub fn engel_ozeti(&self) -> EngelOzeti {
        self.engelleyici.ozet()
    }

    /// Sekmede bu sayfada kaç istek engellendi. Adres çubuğundaki rozet.
    pub fn engel_sayaci(&self, id: SekmeId) -> u32 {
        self.engelleyici.sayac(id)
    }

    // ---------------------------------------------------------------- açılış

    /// Oturumu diskten yükler ve gereken webview'leri açar.
    ///
    /// Sabitlenmemiş sekmeler `Atilmis` doğuyor: 200 sekmelik bir oturum
    /// saniyeler içinde açılıyor ve sıfır render belleği tüketiyor.
    pub fn baslat(self: &Arc<Self>) -> Sonuc<()> {
        {
            let mut depo = self.depo.lock().unwrap();
            oturum::yukle(&mut depo, oturum::oku(&self.oturum_yolu));
            if depo.bos_mu() {
                let id = depo.ac(YENI_SEKME.into(), None, false);
                depo.etkinlestir(id);
            }
        }

        // Webview'i olması gereken sekmeler: sabitlenmişler ve etkin sekme.
        // Oturumdan gelen hiçbir sekme gizli değil (`oturum::yukle`), o
        // yüzden burada bayrak sabit `false` değil sekmeden okunuyor:
        // bir gün oturum gizli sekme taşırsa bu satır sessizce yanlış
        // olmasın.
        let acilacaklar: Vec<(SekmeId, String, bool)> = {
            let depo = self.depo.lock().unwrap();
            depo.hepsi()
                .iter()
                .filter(|s| s.durum.webview_gerekli() && !kabuk_sayfasi(&s.url))
                .map(|s| (s.id, s.url.clone(), s.gizli))
                .collect()
        };
        for (id, url, gizli) in acilacaklar {
            // Bir sekmenin açılamaması diğerlerini engellemiyor: kullanıcı
            // tıkladığında yeniden denenecek.
            let _ = self.motor.sekme_ac(id, &url, gizli);
        }

        let etkin = self.depo.lock().unwrap().etkin();
        if let Some(id) = etkin {
            self.sekme_etkinlestir(id)?;
        }
        self.yayinla_liste();
        self.oturum_dongusu_baslat();
        // Bellek gözcüsü — projenin tezi (`docs/Bellek.md`). Oturum
        // yüklendikten SONRA başlıyor: boş bir depoyla ilk turu koşmanın
        // karşılığı yok.
        gozcu::baslat(self);
        // Geçmiş budaması: açılışta bir kez, arka planda.
        self.budamayi_baslat();
        // Oyun algılama döngüsü. Ayar kapalıyken de dönüyor ve hiçbir şey
        // yapmıyor; ayarı her turda yeniden soruyor (`bridge/muifly.rs`).
        self.oyun_algilamayi_baslat();
        // Köprü durumu bir kez yayınlanıyor: arayüz açılışta hangi kardeş
        // uygulamanın kurulu olduğunu bilmeli, yoksa menüleri boş çiziyor.
        let _ = self.app.emit(olaylar::KOPRU_DURUMU, self.kopru_durumu());
        Ok(())
    }

    /// Oturumu yazan arka plan iş parçacığı.
    ///
    /// Kendi iş parçacığında çünkü yazımın arayüzün çizimine bağlı olmaması
    /// gerekiyor: pencere simge durumundayken de sekmeler kaydedilmeli.
    fn oturum_dongusu_baslat(self: &Arc<Self>) {
        let surucu = Arc::downgrade(self);
        std::thread::Builder::new()
            .name("muiren-oturum".into())
            .spawn(move || {
                let mut son_yazim = Instant::now();
                loop {
                    std::thread::sleep(OTURUM_BEKLEME);
                    // Sürücü düştüyse (uygulama kapandı) döngü de bitiyor.
                    let Some(s) = surucu.upgrade() else { return };
                    let zorunlu = son_yazim.elapsed() >= OTURUM_GUVENLIK;
                    if s.kirli.swap(false, Ordering::AcqRel) || zorunlu {
                        let _ = s.oturum_yaz();
                        son_yazim = Instant::now();
                    }
                }
            })
            .ok();
    }

    fn oturum_isaretle(&self) {
        self.kirli.store(true, Ordering::Release);
    }

    /// Diske hemen yazar. Uygulama kapanırken çağrılıyor.
    pub fn oturum_yaz(&self) -> Sonuc<()> {
        let anlik = oturum::topla(&self.depo.lock().unwrap());
        oturum::yaz(&self.oturum_yolu, &anlik)
    }

    // ----------------------------------------------------------------- sekme

    pub fn sekme_listesi(&self) -> Vec<SekmeOzeti> {
        self.depo.lock().unwrap().ozetler()
    }

    /// `gizli` sekmeyi InPrivate kipinde açıyor (`docs/Roadmap.md` Faz 3).
    ///
    /// Bayrak **ebeveynden miras alınmıyor**: gizli bir sekmedeki bağlantı
    /// yeni sekmede de gizli açılsın diye çağıran bunu açıkça veriyor
    /// (`yeni_pencere` öyle yapıyor). Miras kuralı burada olsaydı,
    /// kullanıcının menüden açtığı normal sekme de gizli doğardı.
    pub fn sekme_ac(
        self: &Arc<Self>,
        url: Option<String>,
        ebeveyn: Option<SekmeId>,
        arkaplanda: bool,
        gizli: bool,
    ) -> Sonuc<SekmeId> {
        let hedef = match url {
            Some(u) if !u.trim().is_empty() => self.coz_adres(&u)?,
            _ => YENI_SEKME.to_string(),
        };

        let id = {
            let mut depo = self.depo.lock().unwrap();
            depo.ac(hedef.clone(), ebeveyn, gizli)
        };

        if arkaplanda {
            if !kabuk_sayfasi(&hedef) {
                // Arka plan sekmesi doğrudan `Arkaplan` durumuna giriyor:
                // webview var, gizli. Gözcü onu bir sonraki turda
                // değerlendirecek.
                self.motor.sekme_ac(id, &hedef, gizli)?;
                self.depo.lock().unwrap().durum_ata(id, Durum::Arkaplan);
            }
            self.yayinla_liste();
            self.oturum_isaretle();
        } else {
            self.sekme_etkinlestir(id)?;
        }
        Ok(id)
    }

    pub fn sekme_kapat(self: &Arc<Self>, id: SekmeId) -> Sonuc<Option<SekmeId>> {
        let vardi = {
            let depo = self.depo.lock().unwrap();
            depo.bul(id).is_some()
        };
        if !vardi {
            return Err(MuirenHata::SekmeYok(id));
        }

        let _ = self.motor.sekme_kapat(id);
        // Açık kronometre sekmeyle birlikte düşüyor: kapanmış bir sekmenin
        // ölçülecek uyanması yok.
        self.gecikme.lock().unwrap().iptal(id);
        // Muiwatch kaydı sekmeyle birlikte düşüyor. Düşmeseydi kapanmış bir
        // sekmenin kimliği koruma listesinde kalırdı; kimlikler yeniden
        // dağıtılmadığı için (`tabs::SekmeId`) zararsız ama liste sonsuza
        // kadar büyürdü.
        self.kopruler.muiwatch.oturumlar.birak(id);
        let yeni_etkin = self.depo.lock().unwrap().kapat(id);

        if let Some(y) = yeni_etkin {
            self.sekme_etkinlestir(y)?;
        } else {
            self.yayinla_liste();
        }
        self.oturum_isaretle();
        Ok(yeni_etkin)
    }

    /// Tıklanan sekme etkin oluyor.
    ///
    /// `Uyuyan` ise `Resume`, `Atilmis` ise URL'den yeniden yükleme. İkisi de
    /// arayüze aynı görünüyor; farkı yalnız burası biliyor.
    pub fn sekme_etkinlestir(self: &Arc<Self>, id: SekmeId) -> Sonuc<()> {
        let (eski, url, onceki_durum, gizli) = {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul(id) else {
                return Err(MuirenHata::SekmeYok(id));
            };
            let onceki = s.durum;
            let url = s.gosterilen_adres();
            // Atılmış gizli sekme geri gelirken de InPrivate doğmalı;
            // bayrak kayıtta duruyor ve webview ondan yeniden kuruluyor.
            let gizli = s.gizli;
            let eski = depo.etkinlestir(id);
            (eski, url, onceki, gizli)
        };

        if let Some(e) = eski.filter(|e| *e != id) {
            // Kaydırma konumu, webview HÂLÂ AYAKTAYKEN isteniyor. Atılma
            // anında sormak yarış demek: `ExecuteScript` eşzamansız ve cevap
            // geldiğinde webview yıkılmış oluyor. Bir tur geriden gelen oran
            // kabul edilebilir (bir ekranlık kayma), sayfayı bekletmek değil.
            let _ = self.motor.kaydirma_iste(e);
            let _ = self.motor.gizle(e);
        }

        if kabuk_sayfasi(&url) {
            // Yeni sekme sayfasını kabuk çiziyor; sekmenin webview'i yok ve
            // olmayacak. Render maliyeti sıfır.
            let _ = self.motor.at(id);
        } else {
            let alan = *self.alan.lock().unwrap();
            if !self.motor.var_mi(id) {
                // Atılmış sekme geri geliyor: kaydırma konumu yüklenme
                // bitince uygulanacak. Bayrak olmadan "kullanıcı aynı adrese
                // elle gitti" ile bu ayırt edilemiyor ve ilkinde sayfa
                // istenmediği hâlde kayardı.
                if let Some(s) = self.depo.lock().unwrap().bul_mut(id) {
                    if s.kaydirma > 0.0 {
                        s.kaydirma_bekliyor = Some(s.kaydirma);
                    }
                }
                // Kronometre webview YARATILMADAN önce başlıyor: ölçülen şey
                // kullanıcının beklediği süre, motorun içindeki bir parça
                // değil. `Atilmis` kaynağı yalnız gerçekten atılmışlıktan
                // dönerken sayılıyor — yeni açılan bir sekmenin ilk yüklemesi
                // "uyanma" değil (`memory/gecikme.rs`).
                if onceki_durum == Durum::Atilmis {
                    self.gecikme.lock().unwrap().basla(
                        id,
                        gecikme::Kaynak::Atilmis,
                        Instant::now(),
                    );
                }
                self.motor.sekme_ac(id, &url, gizli)?;
            } else if onceki_durum == Durum::Uyuyan {
                self.gecikme
                    .lock()
                    .unwrap()
                    .basla(id, gecikme::Kaynak::Uyuyan, Instant::now());
                self.motor.uyandir(id)?;
                // Kronometreyi durduracak olan cevap: sayfanın JS ana iş
                // parçacığı bu betiği çalıştırdığında `kaydirma_okundu`
                // geliyor. `Resume`in dönmesi ölçüm için yetmezdi — o yalnız
                // çağrının kabul edildiğini söylüyor, render sürecinin
                // kullanıcının tuşunu işleyecek hâle geldiğini değil.
                //
                // Ayrı bir "yoklama" metodu açılmadı: bu tam olarak
                // `kaydirma_iste`nin yaptığı `ExecuteScript` turu ve motor
                // yüzeyine ölçüm için ikinci bir tur eklemek, `yok.rs` ile
                // birlikte iki dosyada bakılacak yeni bir kapı demekti
                // (CLAUDE.md #4). Yan etkisi de zararsız: uyanan sekmenin
                // kaydırma oranı tazeleniyor.
                let _ = self.motor.kaydirma_iste(id);
            }
            // Örtü açıkken göstermiyoruz: sayfa ayarlar ekranının üstüne
            // çıkardı (`ortu_gorunur`). Örtü kapanınca oradan gösteriliyor.
            if !self.ortu_acik() {
                self.motor.goster(id, alan)?;
            }
        }

        self.yayinla_liste();
        self.yayinla_durum(id, Durum::Etkin, Sebep::Etkilesim);
        self.oturum_isaretle();
        Ok(())
    }

    pub fn sekme_tasi(&self, id: SekmeId, hedef: usize) -> Sonuc<()> {
        let mut depo = self.depo.lock().unwrap();
        if depo.bul(id).is_none() {
            return Err(MuirenHata::SekmeYok(id));
        }
        depo.tasi(id, hedef);
        drop(depo);
        self.yayinla_liste();
        self.oturum_isaretle();
        Ok(())
    }

    pub fn sekme_sabitle(&self, id: SekmeId, sabit: bool) -> Sonuc<()> {
        let mut depo = self.depo.lock().unwrap();
        if depo.bul(id).is_none() {
            return Err(MuirenHata::SekmeYok(id));
        }
        depo.sabitle(id, sabit);
        drop(depo);
        self.yayinla_liste();
        self.oturum_isaretle();
        Ok(())
    }

    /// Sessize alınan sekme **koruma kaybediyor**: arka planda müzik dinlemek
    /// birinci sınıf kullanım ama sesi kapatılmış bir sekmeyi ses çalıyor diye
    /// uyanık tutmanın karşılığı yok (`docs/Bellek.md`).
    pub fn sekme_sessize_al(&self, id: SekmeId, sessiz: bool) -> Sonuc<()> {
        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else {
                return Err(MuirenHata::SekmeYok(id));
            };
            s.sessiz = sessiz;
        }
        let _ = self.motor.ses_kes(id, sessiz);
        self.yayinla_sekme(id);
        Ok(())
    }

    pub fn sekme_uyutma_istisnasi(&self, id: SekmeId, istisna: bool) -> Sonuc<()> {
        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else {
                return Err(MuirenHata::SekmeYok(id));
            };
            s.uyutma_istisnasi = istisna;
        }
        self.yayinla_sekme(id);
        self.oturum_isaretle();
        Ok(())
    }

    pub fn sekme_geri_al(self: &Arc<Self>) -> Sonuc<Option<SekmeId>> {
        let Some(eski) = self.depo.lock().unwrap().geri_al() else {
            return Ok(None);
        };
        // Gizli sekme gizli olarak geri geliyor. Alternatifi, kullanıcının
        // Ctrl+Shift+T ile kapattığı gizli sekmeyi normal bir sekmede
        // açmak olurdu; o adres o anda geçmişe yazılırdı.
        let id = self.sekme_ac(Some(eski.url.clone()), eski.ebeveyn, false, eski.gizli)?;
        if let Some(s) = self.depo.lock().unwrap().bul_mut(id) {
            s.baslik = eski.baslik;
            s.kaydirma = eski.kaydirma;
            s.uyutma_istisnasi = eski.uyutma_istisnasi;
        }
        self.yayinla_liste();
        Ok(Some(id))
    }

    // -------------------------------------------------------------- gezinme

    /// Ham kullanıcı metnini adrese çevirir.
    ///
    /// Ayrımı [`adres_mi`] veriyor: **saf ve testli** (`tabs/mod.rs`). Aynı
    /// ayrımın arayüzdeki kopyası (`src/lib/url.ts`) yalnız canlı geri
    /// bildirim çiziyor; motora ne gideceğine karar veren yer burası, çünkü
    /// bu kapıdan kabuk kapalıyken de geçiliyor (oturum, köprü, kısayol).
    ///
    /// Şemasız yazılan adres `https`e tamamlanıyor: kullanıcı `ornek.com`
    /// yazdığında oraya gitmesi gerekiyor, o metni aratmak değil.
    fn coz_adres(&self, girdi: &str) -> Sonuc<String> {
        let girdi = girdi.trim();
        if girdi.is_empty() {
            return Ok(YENI_SEKME.to_string());
        }
        if kabuk_sayfasi(girdi) {
            return Ok(girdi.to_string());
        }

        if super::adres_mi(girdi) {
            let tam = if super::sema_oneki(girdi).is_some() {
                girdi.to_string()
            } else {
                format!("https://{girdi}")
            };
            let u = url::Url::parse(&tam)?;
            return match u.scheme() {
                "http" | "https" | "file" => Ok(u.to_string()),
                _ => Err(MuirenHata::GecersizAdres(girdi.to_string())),
            };
        }

        // Arama. `?` öneki "bu bir adres değil" demenin yolu; şablona
        // girmeden atılıyor.
        let sorgu = girdi.strip_prefix('?').unwrap_or(girdi).trim();
        let sablon = self.ayarlar.lock().unwrap().arama_url.clone();
        Ok(sablon.replace("%s", &urlencode(sorgu)))
    }

    pub fn gezin(self: &Arc<Self>, id: SekmeId, girdi: String) -> Sonuc<()> {
        let hedef = self.coz_adres(&girdi)?;

        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else {
                return Err(MuirenHata::SekmeYok(id));
            };
            // `url` alanı son BAŞARILI gezinme; buraya yazılmıyor
            // (CLAUDE.md #7). Adres çubuğu `bekleyen`i gösteriyor.
            s.bekleyen = Some(hedef.clone());
            s.yukleniyor = !kabuk_sayfasi(&hedef);
        }

        if kabuk_sayfasi(&hedef) {
            let _ = self.motor.at(id);
            let mut depo = self.depo.lock().unwrap();
            if let Some(s) = depo.bul_mut(id) {
                s.url = hedef;
                s.bekleyen = None;
                s.baslik = String::new();
                s.durum = Durum::Etkin;
            }
            drop(depo);
            self.yayinla_liste();
            self.oturum_isaretle();
            return Ok(());
        }

        let alan = *self.alan.lock().unwrap();
        if self.motor.var_mi(id) {
            self.motor.gezin(id, &hedef)?;
        } else {
            let gizli = self.depo.lock().unwrap().bul(id).is_some_and(|s| s.gizli);
            self.motor.sekme_ac(id, &hedef, gizli)?;
        }
        let etkin = self.depo.lock().unwrap().etkin() == Some(id);
        if etkin && !self.ortu_acik() {
            self.motor.goster(id, alan)?;
        }
        self.yayinla_sekme(id);
        Ok(())
    }

    pub fn geri(&self, id: SekmeId) -> Sonuc<()> {
        self.motor.geri(id)
    }

    pub fn ileri(&self, id: SekmeId) -> Sonuc<()> {
        self.motor.ileri(id)
    }

    pub fn yenile(&self, id: SekmeId) -> Sonuc<()> {
        self.motor.yenile(id)
    }

    pub fn durdur(&self, id: SekmeId) -> Sonuc<()> {
        self.motor.durdur(id)
    }

    /// Kabuk tam ekran bir örtü açtı/kapattı (ayarlar, sekme arama).
    ///
    /// **Neden gerekiyor:** sekme webview'i ayrı bir native pencere ve
    /// kabuğun **üstünde** duruyor (`lib.rs`, pencere düzeni). Kabuğun
    /// çizdiği tam ekran bir örtü, sayfanın altında kalıyor ve kullanıcı
    /// ayarlar ekranını hiç göremiyor. CSS `z-index` burada işe yaramıyor:
    /// iki ayrı pencerenin sırası CSS'in görebildiği bir şey değil.
    ///
    /// Çözüm sayfayı **gizlemek**, uyutmak değil: `hide()` yalnız native
    /// pencereyi gizliyor, sekme durumu ve sayfanın kendisi olduğu gibi
    /// duruyor. Örtü kapanınca aynı alana geri geliyor.
    ///
    /// > **Bilinen sınır:** adres çubuğu öneri listesi bu kapıdan
    /// > geçmiyor. Öneriler kullanıcı yazarken açılıyor ve her tuşta sayfayı
    /// > gizleyip göstermek, düzeltmeye çalıştığından daha rahatsız edici
    /// > bir titreme üretirdi. Liste içerik alanının üst şeridine taşıyor ve
    /// > orada sayfanın altında kalıyor.
    pub fn ortu_gorunur(&self, ortu_acik: bool) -> Sonuc<()> {
        self.ortu_acik.store(ortu_acik, Ordering::Release);
        let Some(id) = self.depo.lock().unwrap().etkin() else {
            return Ok(());
        };
        if ortu_acik {
            let _ = self.motor.gizle(id);
            // Örtünün kendi girdileri var (ayarlar alanları, sekme arama
            // kutusu). Sayfa gizlenince odak kendiliğinden kabuğa geçmiyor;
            // `kisayol_uygula` ile aynı gerekçe.
            let _ = self.motor.kabuk_odakla();
            Ok(())
        } else {
            let alan = *self.alan.lock().unwrap();
            // Kabuk sayfasındaysak gösterilecek bir webview yok.
            if self.motor.var_mi(id) {
                self.motor.goster(id, alan)
            } else {
                Ok(())
            }
        }
    }

    fn ortu_acik(&self) -> bool {
        self.ortu_acik.load(Ordering::Acquire)
    }

    /// Arayüz kabuğun altında kalan alanı bildiriyor (yan panel açıldığında,
    /// pencere boyutu değiştiğinde, tam ekrana geçildiğinde).
    pub fn icerik_alani(&self, dikdortgen: Dikdortgen) -> Sonuc<()> {
        // Kabuğun kendi CSS görünüm alanını öğrenmenin tek ucuz yolu: bu
        // dikdörtgen arayüzde `getBoundingClientRect` ile ölçülüyor, yani
        // birimi CSS pikseli. Kabuk pencereye oturmadığında ilk buradan
        // görünüyor (`MUIREN_LOG=debug`).
        log::debug!(
            "içerik alanı: {}x{} @({},{}) CSS px",
            dikdortgen.genislik,
            dikdortgen.yukseklik,
            dikdortgen.x,
            dikdortgen.y
        );
        *self.alan.lock().unwrap() = dikdortgen;
        self.motor.alan_ayarla(dikdortgen)
    }

    // --------------------------------------------------------- elle bellek

    /// Uyanma gecikmesi dağılımı (`docs/IPC.md`, `gecikme_ozeti`).
    pub fn gecikme_ozeti(&self) -> GecikmeOzeti {
        self.gecikme.lock().unwrap().ozet()
    }

    /// Defteri boşaltır — bir ölçüm oturumuna temiz başlamak için.
    ///
    /// Gerekçe `docs/olcumler/` protokolünde: rapor edilen dağılım, o
    /// oturumda ölçülenlerin dağılımı olmalı. Kullanıcının sabah yaptığı
    /// gezinmeler ölçüm oturumunun medyanına karışırsa rapor, ölçüldüğü
    /// söylenen şeyi ölçmemiş olur.
    pub fn gecikme_sifirla(&self) {
        self.gecikme.lock().unwrap().temizle();
    }

    pub fn sekme_uyut(&self, id: SekmeId) -> Sonuc<()> {
        self.durum_degistir(id, Olay::Uyutuldu, Sebep::Elle)
    }

    pub fn sekme_at(&self, id: SekmeId) -> Sonuc<()> {
        self.durum_degistir(id, Olay::Atildi, Sebep::Elle)
    }

    /// Durum geçişini uygular: saf makineye sorar, motora der, olay yayınlar.
    ///
    /// Gözcü de (Faz 2) bu kapıdan geçecek — uyutma/atma tek yerde olsun diye.
    pub fn durum_degistir(&self, id: SekmeId, olay: Olay, sebep: Sebep) -> Sonuc<()> {
        let (onceki, sonraki) = {
            let depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul(id) else {
                return Err(MuirenHata::SekmeYok(id));
            };
            (s.durum, durum::gecis(s.durum, olay))
        };
        if onceki == sonraki {
            return Ok(());
        }

        match sonraki {
            Durum::Uyuyan | Durum::Atilmis => {
                // Uyanma kronometresi açıkken sekme geri uyutuluyor/atılıyorsa
                // ölçülecek bir uyanma kalmadı. İptal edilmeseydi geç gelen
                // betik cevabı, arada geçen bütün uyku süresini "uyanma
                // gecikmesi" diye deftere yazardı.
                self.gecikme.lock().unwrap().iptal(id);
            }
            _ => {}
        }

        match sonraki {
            Durum::Uyuyan => {
                // Uyumadan önce kaydırma soruluyor: uyuyan sekme sonradan
                // atılabiliyor ve o an sormak için çok geç oluyor.
                let _ = self.motor.kaydirma_iste(id);
                // `TrySuspend` yalnız görünmeyen bir webview'de çalışıyor.
                let _ = self.motor.gizle(id);
                self.motor.uyut(id)?;
            }
            Durum::Atilmis => self.motor.at(id)?,
            // Uyanma yönü de bu kapıdan geçiyor. Eksik olsaydı, kapı kaydı
            // `Etkin` yapıp motora hiçbir şey demez ve sekme "uyandı"
            // görünürken webview'i askıda kalırdı — belirtisi tıklanınca boş
            // kalan bir sekme.
            //
            // `sekme_etkinlestir` buraya **uğramıyor**; kullanıcının tıkladığı
            // sekmenin `Resume`u orada, gösterme ve alan hesabıyla birlikte
            // duruyor. Yani çift `Resume` yok. Bu dal gözcü tarafındaki
            // `Eylem::Uyandir` için: bugün üretilmiyor ama üretildiği gün
            // sessizce yarım çalışmasın.
            Durum::Etkin if onceki == Durum::Uyuyan => self.motor.uyandir(id)?,
            _ => {}
        }

        self.depo.lock().unwrap().durum_ata(id, sonraki);
        self.yayinla_durum(id, sonraki, sebep);
        self.yayinla_liste();
        Ok(())
    }

    // ------------------------------------------------------- geçmiş / yer imi

    /// Geçmiş deposu. `None` ise veritabanı açılamamış; komutlar boş liste
    /// döndürüyor ve tarayıcı çalışmaya devam ediyor.
    pub fn gecmis(&self) -> Option<&GecmisDepo> {
        self.gecmis.as_ref()
    }

    /// Başarılı bir gezinmeyi geçmişe yazar.
    ///
    /// Üç şey **yazılmıyor**: kabuk sayfaları (`muiren://yeni`), başarısız
    /// gezinmeler (CLAUDE.md #7) ve `about:` benzeri şemalar. İlk ikisi
    /// geçmişi çöple dolduruyor, üçüncüsü kullanıcının gitmediği bir yer.
    fn gecmise_yaz(&self, id: SekmeId, url: &str, baslik: &str) {
        if kabuk_sayfasi(url) || !(url.starts_with("http://") || url.starts_with("https://")) {
            return;
        }
        // Dördüncü yazılmayan: **gizli sekmeler**. Motorun InPrivate kipi
        // çerezleri ve önbelleği bellekte tutuyor ama geçmiş bizim
        // veritabanımız — motor onu bilmiyor. Bu satır olmadan "gizli"
        // iddiası ilk gezinmede çökerdi.
        if self.depo.lock().unwrap().bul(id).is_some_and(|s| s.gizli) {
            return;
        }
        let Some(depo) = self.gecmis.as_ref() else {
            return;
        };
        if let Err(e) = depo.ziyaret_ekle(url, baslik) {
            // Geçmişe yazamamak gezinmeyi durdurmuyor.
            log::warn!("muiren: geçmişe yazılamadı: {e}");
        }
    }

    /// Saklama süresini geçmiş kayıtları budar.
    ///
    /// Açılışta bir kez, **arka planda** koşuyor: 90 günlük bir budama on
    /// binlerce satır silebiliyor ve bunu açılış yolunda yapmak pencerenin
    /// görünmesini geciktirirdi (`docs/Depolama.md`).
    fn budamayi_baslat(self: &Arc<Self>) {
        let surucu = Arc::downgrade(self);
        std::thread::Builder::new()
            .name("muiren-budama".into())
            .spawn(move || {
                let Some(s) = surucu.upgrade() else { return };
                let gun = s.ayarlar().gecmis_saklama_gun;
                if let Some(depo) = s.gecmis.as_ref() {
                    match depo.buda(gun) {
                        Ok(n) if n > 0 => log::info!("muiren: {n} geçmiş kaydı budandı"),
                        Err(e) => log::warn!("muiren: geçmiş budanamadı: {e}"),
                        _ => {}
                    }
                }
            })
            .ok();
    }

    // ---------------------------------------------------------------- tema

    /// Yerleşik + kurulu bütün temalar.
    ///
    /// Yerleşikler de `paket::yerlesik_ozet` üzerinden, yani **aynı**
    /// doğrulamadan geçerek üretiliyor. İstisna olan yol, bir gün
    /// doğrulamayı atlayan yol oluyor (`docs/Temalar.md`).
    pub fn tema_listesi(&self) -> Vec<TemaOzeti> {
        let mut liste: Vec<TemaOzeti> = crate::theme::yerlesikler()
            .into_iter()
            .map(|(ad, j)| paket::yerlesik_ozet(ad, j))
            .collect();
        liste.extend(paket::kurulular(&self.temalar_dizini));
        liste
    }

    fn tema_bul(&self, ad: &str) -> Option<TemaOzeti> {
        self.tema_listesi().into_iter().find(|t| t.ad == ad)
    }

    /// Ayarlardaki temanın doğrulanmış hâli.
    ///
    /// Tema silinmiş ya da bozulmuşsa **varsayılana düşüyor**: kullanıcının
    /// tarayıcısı, kaybolmuş bir tema yüzünden jetonsuz açılmamalı.
    pub fn tema_etkin(&self) -> Option<TemaOzeti> {
        let ad = self.ayarlar().tema;
        self.tema_bul(&ad).or_else(|| self.tema_bul("Mui"))
    }

    /// `.muitema` dosyasını kurar ve uygular.
    pub fn tema_yukle(&self, dosya_yolu: &str) -> Sonuc<TemaOzeti> {
        std::fs::create_dir_all(&self.temalar_dizini)?;
        let ozet = paket::kur(std::path::Path::new(dosya_yolu), &self.temalar_dizini)?;
        self.tema_uygula(&ozet.ad)
    }

    /// Temayı seçili hâle getirir, ayara yazar ve olayı yayınlar.
    pub fn tema_uygula(&self, ad: &str) -> Sonuc<TemaOzeti> {
        let ozet = self
            .tema_bul(ad)
            .ok_or_else(|| MuirenHata::Bicim(format!("tema bulunamadı: {ad}")))?;

        let mut ayarlar = self.ayarlar();
        ayarlar.tema = ozet.ad.clone();
        // `ayarlar_yaz` düzeltmeden geçiriyor: boş ad "Mui"ye dönüyor.
        self.ayarlar_yaz(ayarlar)?;

        let _ = self.app.emit(olaylar::TEMA_DEGISTI, ozet.clone());
        Ok(ozet)
    }

    pub fn tema_sil(&self, ad: &str) -> Sonuc<()> {
        paket::sil(ad, &self.temalar_dizini)?;
        // Silinen tema seçiliyse varsayılana dön; yoksa arayüz var olmayan
        // bir temayı seçili gösterirdi.
        if self.ayarlar().tema == ad {
            self.tema_uygula("Mui")?;
        }
        Ok(())
    }

    pub fn tema_disa_aktar(&self, ad: &str, hedef: &str) -> Sonuc<()> {
        // Yerleşik temalar da dışa aktarılabiliyor: kullanıcı onları
        // başlangıç noktası olarak kullanabilmeli.
        if crate::theme::yerlesikler().iter().any(|(a, _)| *a == ad) {
            let (_, jetonlar) = crate::theme::yerlesikler()
                .into_iter()
                .find(|(a, _)| *a == ad)
                .expect("az önce bulundu");
            std::fs::create_dir_all(self.temalar_dizini.join(ad))?;
            let json = serde_json::json!({
                "bicim": crate::theme::BICIM,
                "ad": ad,
                "yazar": "Mui",
                "surum": env!("CARGO_PKG_VERSION"),
                "jetonlar": jetonlar.iter().copied().collect::<std::collections::BTreeMap<_, _>>(),
            });
            std::fs::write(
                self.temalar_dizini.join(ad).join("tema.json"),
                serde_json::to_vec_pretty(&json)?,
            )?;
        }
        paket::disa_aktar(ad, &self.temalar_dizini, std::path::Path::new(hedef))
    }
    // --------------------------------------------------------------- gruplar

    pub fn grup_listesi(&self) -> Vec<Grup> {
        self.depo.lock().unwrap().gruplar().to_vec()
    }

    pub fn grup_ac(&self, ad: String) -> GrupId {
        let id = self.depo.lock().unwrap().grup_ac(&ad);
        self.yayinla_liste();
        self.oturum_isaretle();
        id
    }

    pub fn grup_sil(&self, id: GrupId) {
        self.depo.lock().unwrap().grup_sil(id);
        self.yayinla_liste();
        self.oturum_isaretle();
    }

    pub fn grup_ata(&self, sekme: SekmeId, grup: Option<GrupId>) {
        self.depo.lock().unwrap().grup_ata(sekme, grup);
        self.yayinla_liste();
        self.oturum_isaretle();
    }

    /// Grubun adını, rengini, katlanma durumunu ve uyku eşiğini günceller.
    ///
    /// Her alan `Option`: "değiştirme" ile "sıfırla" farklı şeyler.
    /// `uyku_esigi_sn` iki katmanlı — `None` dokunma, `Some(None)` grubu
    /// genel ayara döndür, `Some(Some(n))` eşiği n yap.
    pub fn grup_guncelle(
        &self,
        id: GrupId,
        ad: Option<String>,
        renk: Option<GrupRengi>,
        katli: Option<bool>,
        uyku_esigi_sn: Option<Option<u64>>,
    ) {
        self.depo
            .lock()
            .unwrap()
            .grup_guncelle(id, ad.as_deref(), renk, katli, uyku_esigi_sn);
        // Liste yayınlanıyor çünkü `SekmeOzeti.katli` ve
        // `grup_uyku_esigi_sn` her sekmede türetiliyor: grup değişince
        // sekmelerin özeti de değişiyor.
        self.yayinla_liste();
        self.oturum_isaretle();
    }

    // -------------------------------------------------------------- köprüler

    /// Köprü durumları (`docs/IPC.md`, `kopru_durumu`).
    pub fn kopru_durumu(&self) -> Vec<KopruDurumu> {
        self.kopruler.durumlar()
    }

    /// Kurulum önbelleklerini düşürüp durumu yeniden yayınlar.
    ///
    /// Kullanıcı Muiren açıkken bir kardeş uygulama kurabiliyor; ayarlar
    /// ekranındaki "yeniden ara" düğmesi buraya bağlı.
    pub fn kopru_tazele(&self) -> Vec<KopruDurumu> {
        self.kopruler.tazele();
        let durumlar = self.kopruler.durumlar();
        let _ = self.app.emit(olaylar::KOPRU_DURUMU, durumlar.clone());
        durumlar
    }

    /// İndirmeyi Muiget'e devreder.
    ///
    /// Ad **burada** temizleniyor: komut da bu kapıdan geçiyor ve arayüzden
    /// gelen ad da sonuçta sayfadan geliyor (CLAUDE.md #8).
    pub fn muiget_gonder(
        &self,
        url: String,
        dosya_adi: Option<String>,
        kaynak_sayfa: Option<String>,
    ) -> Sonuc<()> {
        self.kopruler.muiget.devret(Yuk::Indirme {
            url,
            dosya_adi: dosya_adi
                .as_deref()
                .and_then(crate::bridge::dosya_adi_temizle),
            kaynak_sayfa,
        })
    }

    /// Yerel medya dosyasını Muiply'a devreder.
    pub fn muiply_ac(&self, yol: String) -> Sonuc<()> {
        self.kopruler
            .muiply
            .devret(Yuk::DosyaYolu(PathBuf::from(yol)))
    }

    /// Sekmeyi bir Muiwatch oturumuna bağlar (koruma kuralı #7).
    ///
    /// Bağlama **başarısız devirde de** yapılıyor mu? Hayır: önce Muiwatch
    /// açılıyor, açılamazsa kayıt hiç girmiyor. Girseydi, hiç başlamamış bir
    /// oturum yüzünden bir sekme sonsuza kadar korumalı kalırdı.
    pub fn muiwatch_bagla(&self, id: SekmeId, oda: String) -> Sonuc<()> {
        if self.depo.lock().unwrap().bul(id).is_none() {
            return Err(MuirenHata::SekmeYok(id));
        }
        self.kopruler.muiwatch.devret(Yuk::Oda(oda.clone()))?;
        self.kopruler.muiwatch.oturumlar.bagla(id, oda);
        self.yayinla_sekme(id);
        Ok(())
    }

    pub fn muiwatch_birak(&self, id: SekmeId) {
        self.kopruler.muiwatch.oturumlar.birak(id);
        self.yayinla_sekme(id);
    }

    /// Koruma kuralı #7'nin girdisi. Gözcü turun başında bir kez okuyor.
    pub fn muiwatch_sekmeleri(&self) -> Vec<SekmeId> {
        self.kopruler.muiwatch.oturumlar.bagli_sekmeler()
    }

    /// Motorun kendi indirmesine bir kerelik izin verir ve indirmeyi yeniden
    /// tetikler.
    ///
    /// `IndirmePolitikasi::Sor` altında motor indirmeyi iptal etmişti;
    /// iptal edilmiş bir WebView2 indirmesi **sürdürülemiyor**, o yüzden aynı
    /// adrese yeniden gidiliyor. Bilet tek atımlık: ikinci
    /// `DownloadStarting` onu görüp tüketiyor.
    pub fn indirme_izin_ver(&self, id: SekmeId, url: String) -> Sonuc<()> {
        self.indirme_biletleri.lock().unwrap().insert(url.clone());
        if self.motor.var_mi(id) {
            self.motor.gezin(id, &url)
        } else {
            // Sekmenin webview'i yoksa (uyudu/atıldı) indirmeyi
            // tetikleyecek bir sayfa da yok. Bilet duruyor: sekme geri
            // geldiğinde kullanılabilir.
            Err(MuirenHata::SekmeYok(id))
        }
    }

    /// Veri temizleme (`docs/IPC.md`, `veri_temizle`).
    ///
    /// **İki taraf birden**: motorun profili (çerez, önbellek, site verisi) ve
    /// Muiren'in kendi depoları (geçmiş, favicon). Ayrım kullanıcıya
    /// görünmüyor ve görünmemeli — "çerezleri sil" dediğinde çerezlerin
    /// yarısının kalması diye bir şey yok.
    ///
    /// Dönüş, **ne yapıldığının raporu**. Motorun kendi silmesi eşzamansız
    /// olduğu için orası yalnız "başlatıldı" diyor; bizim depolarımızda
    /// gerçek sayılar var.
    pub fn veri_temizle(&self, istek: Temizlik) -> Sonuc<TemizlikRaporu> {
        let mut rapor = TemizlikRaporu::default();

        // Sıra önemli: favicon budaması geçmişten SONRA yapılıyor. Önce
        // yapılsaydı "hâlâ kullanılan" listesi silinmek üzere olan geçmiş
        // kayıtlarını da sayar ve ikonlar depoda kalırdı.
        if istek.gecmis {
            if let Some(depo) = self.gecmis.as_ref() {
                rapor.gecmis_kaydi = depo.gecmis_temizle(crate::history::Aralik::Hepsi)?;
            }
        }

        if istek.favicon {
            rapor.favicon_dosyasi = favicon::temizle(&self.favicon_dizini)?;
            // Sekme kayıtlarındaki kimlikler de düşüyor: dosya gitti,
            // kimlik duruyorsa arayüz her açılışta var olmayan bir ikonu
            // sorar ve sekme çubuğu boş kare gösterirdi.
            {
                let mut depo = self.depo.lock().unwrap();
                for s in depo.hepsi_mut() {
                    s.favicon = None;
                }
            }
            self.yayinla_liste();
        }

        let motor_kalemleri = istek.motor_kalemleri();
        if !motor_kalemleri.bos_mu() {
            self.motor.veri_temizle(motor_kalemleri)?;
            rapor.motor_baslatildi = true;
        }

        Ok(rapor)
    }

    /// Favicon'u `data:` adresi olarak verir (`docs/IPC.md`, `favicon_oku`).
    ///
    /// Kimlik doğrulanıyor: dize arayüzden geri geliyor ve bir dosya yoluna
    /// dönüşüyor (`favicon::kimlik_gecerli`).
    pub fn favicon_oku(&self, kimlik: &str) -> Option<String> {
        favicon::oku(&self.favicon_dizini, kimlik)
    }

    /// Oyun algılamasını başlatır.
    ///
    /// Döngü ayar kapalıyken de dönüyor ama hiçbir şey yapmıyor: ayar her
    /// turda yeniden soruluyor, böylece kullanıcı ayarı açtığında yeniden
    /// başlatma gerekmiyor (`bridge/muifly.rs`).
    fn oyun_algilamayi_baslat(self: &Arc<Self>) {
        let ayar_kaynagi = Arc::downgrade(self);
        let uygula_kaynagi = Arc::downgrade(self);
        let kopru = self.kopruler.muifly.clone();
        kopru.izlemeyi_baslat(
            move || {
                ayar_kaynagi
                    .upgrade()
                    .is_some_and(|s| s.ayarlar().oyun_algilama)
            },
            move |acik, kaynak| {
                if let Some(s) = uygula_kaynagi.upgrade() {
                    gozcu::oyun_modu(&s, acik, kaynak);
                }
            },
        );
    }

    // ------------------------------------------------------ bellek politikası

    /// Gözcünün ve komutların olay yayınlamak için ihtiyacı olan tutamak.
    pub fn app(&self) -> &AppHandle<R> {
        &self.app
    }

    pub fn oyun_modu(&self) -> bool {
        self.oyun_modu.load(Ordering::Relaxed)
    }

    pub fn oyun_modu_bayragi(&self) -> &AtomicBool {
        &self.oyun_modu
    }

    pub fn pencere_gizli(&self) -> bool {
        self.pencere_gizli.load(Ordering::Relaxed)
    }

    pub fn pencere_gizli_bayragi(&self) -> &AtomicBool {
        &self.pencere_gizli
    }

    /// Pencereyi oyun modu küçülttü mü (`memory/gozcu.rs` okuyup yazıyor).
    pub fn oyun_kucultu_bayragi(&self) -> &AtomicBool {
        &self.oyun_kucultu
    }

    /// Bu sekmenin bellek hedefi bu tur düşürülsün mü.
    ///
    /// İlk çağrıda `true` dönüp kaydediyor, sonrakilerde `false`: aynı sekmeye
    /// her turda `SetMemoryUsageTargetLevel` göndermenin karşılığı yok.
    pub fn hedef_dusurulsun_mu(&self, id: SekmeId) -> bool {
        self.hedefi_dusurulenler.lock().unwrap().insert(id)
    }

    /// Sekme uyudu/atıldı: hedef kaydı anlamını yitirdi.
    pub fn hedef_dusurme_unut(&self, id: SekmeId) {
        self.hedefi_dusurulenler.lock().unwrap().remove(&id);
    }

    pub fn bellek_hedefi_dusur(&self, id: SekmeId) -> Sonuc<()> {
        self.motor.bellek_hedefi(id, BellekSeviyesi::Dusuk)
    }

    pub fn bellek_ozeti_yaz(&self, ozet: BellekOzeti) {
        *self.son_ozet.lock().unwrap() = Some(ozet);
    }

    /// Motordan süreç → sekme eşlemesini ister (ateşle-unut).
    ///
    /// Gözcü motoru doğrudan çağırmıyor: `bellek_hedefi_dusur` ile aynı
    /// kalıp. Motor yüzeyinin tek müşterisi sürücü kalsın diye.
    pub fn surec_bilgisi_iste(&self) {
        if let Err(e) = self.motor.surec_bilgisi_iste() {
            // Sekme yokken de buraya geliniyor; hata seviyesi değil.
            log::debug!("muiren: süreç bilgisi istenemedi: {e}");
        }
    }

    /// Motorun son bilinen süreç → sekme anlık görüntüsü.
    /// Kurulu WebView2 Runtime sürümü (`docs/olcumler/README.md` başlığı).
    pub fn calisma_zamani_surumu(&self) -> Option<String> {
        self.motor.calisma_zamani_surumu()
    }

    pub fn surec_haritasi(&self) -> Vec<SurecKaydi> {
        self.motor.surec_haritasi()
    }

    /// Süreç ölçümlerini sekme başına rakama çeviren defter.
    pub fn bellek_defteri(&self) -> &Mutex<esleme::Defter> {
        &self.bellek_defteri
    }

    /// Son bellek özeti. Yoksa gözcü henüz ilk turunu koşmamış demektir; o
    /// durumda bir tur koşturmak yerine `None` dönüyor ve arayüz ilk olayı
    /// bekliyor — panel açılışında süreç tablosunu taramanın karşılığı yok.
    pub fn bellek_ozeti(&self) -> Option<BellekOzeti> {
        self.son_ozet.lock().unwrap().clone()
    }

    // ------------------------------------------------------------- kısayol

    /// Kısayolu uygular. Dönüş: tabloda karşılığı var mıydı.
    ///
    /// **İki kapı da buraya çıkıyor** (`docs/IPC.md`): kabuk odaktayken
    /// `kisayol_bas` komutu, sayfa odaktayken motorun hızlandırıcı kaydı.
    /// Tablo tek yerde (`crate::kisayol`), uygulama tek yerde burada.
    ///
    /// Arayüz işleri (adres çubuğuna odaklan, paneli aç) backend'den
    /// yapılamıyor; onlar `muiren://kisayol` olayıyla kabuğa gönderiliyor.
    pub fn kisayol_uygula(self: &Arc<Self>, tus: &str, ctrl: bool, shift: bool, alt: bool) -> bool {
        let Some(k) = crate::kisayol::coz(tus, ctrl, shift, alt) else {
            return false;
        };

        if k.arayuz_isi() {
            // Klavyeyi önce kabuğa al: bu kısayollar kabukta bir alana
            // odaklanıyor (adres çubuğu, sekme arama kutusu) ve sayfa
            // odaktayken kabuğun `focus()` çağrısı tuşları geri getirmiyor.
            // Bu satır olmadan Ctrl+L adres çubuğunu açıyor ama yazılan her
            // harf sayfaya gidiyor (`Motor::kabuk_odakla`).
            if k.odak_ister() {
                let _ = self.motor.kabuk_odakla();
            }
            let _ = self.app.emit(olaylar::KISAYOL, KisayolOlayi { is: k });
            return true;
        }

        let etkin = self.depo.lock().unwrap().etkin();
        match k {
            Kisayol::YeniSekme => {
                let _ = self.sekme_ac(None, None, false, false);
            }
            Kisayol::GizliSekme => {
                // Gizli sekme boş açılıyor: adresi kullanıcı yazacak.
                // O ana kadar hiçbir şey diske değmiyor.
                let _ = self.sekme_ac(None, None, false, true);
            }
            Kisayol::SekmeKapat => {
                if let Some(id) = etkin {
                    let _ = self.sekme_kapat(id);
                }
            }
            Kisayol::GeriAl => {
                let _ = self.sekme_geri_al();
            }
            Kisayol::SonrakiSekme => self.komsu_sekmeye_gec(1),
            Kisayol::OncekiSekme => self.komsu_sekmeye_gec(-1),
            Kisayol::SekmeNo(n) => {
                let hedef = self
                    .depo
                    .lock()
                    .unwrap()
                    .hepsi()
                    .get(n.saturating_sub(1) as usize)
                    .map(|s| s.id);
                if let Some(id) = hedef {
                    let _ = self.sekme_etkinlestir(id);
                }
            }
            Kisayol::SonSekme => {
                let hedef = self.depo.lock().unwrap().hepsi().last().map(|s| s.id);
                if let Some(id) = hedef {
                    let _ = self.sekme_etkinlestir(id);
                }
            }
            Kisayol::BuSekmeyiUyut => {
                if let Some(id) = etkin {
                    // Etkin sekme durum makinesi tarafından korunuyor
                    // (koruma kuralı #1): önce komşuya geçiliyor, sonra
                    // uyutuluyor. Aksi hâlde kısayol sessizce hiçbir şey
                    // yapmazdı.
                    self.komsu_sekmeye_gec(1);
                    let _ = self.sekme_uyut(id);
                }
            }
            Kisayol::HepsiniUyut => {
                gozcu::hepsini_uyut(self);
            }
            Kisayol::Yenile => {
                if let Some(id) = etkin {
                    let _ = self.yenile(id);
                }
            }
            Kisayol::Geri => {
                if let Some(id) = etkin {
                    let _ = self.geri(id);
                }
            }
            Kisayol::Ileri => {
                if let Some(id) = etkin {
                    let _ = self.ileri(id);
                }
            }
            Kisayol::Durdur => {
                if let Some(id) = etkin {
                    let _ = self.durdur(id);
                }
            }
            // `arayuz_isi()` yukarıda ele alındı.
            Kisayol::AdresOdak
            | Kisayol::SekmeArama
            | Kisayol::BellekPaneli
            | Kisayol::YanPanel
            | Kisayol::TamEkran => {}
        }
        true
    }

    /// Sıradaki/önceki sekmeye geçer. Liste başında/sonunda başa dönüyor.
    fn komsu_sekmeye_gec(self: &Arc<Self>, adim: isize) {
        let hedef = {
            let depo = self.depo.lock().unwrap();
            let hepsi = depo.hepsi();
            if hepsi.len() < 2 {
                return;
            }
            let simdiki = depo
                .etkin()
                .and_then(|id| hepsi.iter().position(|s| s.id == id))
                .unwrap_or(0) as isize;
            let n = hepsi.len() as isize;
            hepsi[(((simdiki + adim) % n + n) % n) as usize].id
        };
        let _ = self.sekme_etkinlestir(hedef);
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct KisayolOlayi {
    is: Kisayol,
}

impl<R: Runtime> Surucu<R> {
    // ---------------------------------------------------------------- olay

    fn yayinla_liste(&self) {
        let liste = self.sekme_listesi();
        let _ = self.app.emit(olaylar::SEKME_DEGISTI, liste);
    }

    fn yayinla_sekme(&self, id: SekmeId) {
        if let Some(o) = self.depo.lock().unwrap().ozet(id) {
            let _ = self.app.emit(olaylar::SEKME_GUNCELLENDI, o);
        }
    }

    fn yayinla_durum(&self, id: SekmeId, durum: Durum, sebep: Sebep) {
        let _ = self.app.emit(
            olaylar::SEKME_DURUM_DEGISTI,
            DurumOlayi { id, durum, sebep },
        );
    }
}

/// Arama sorgusu için asgari yüzde kodlaması.
///
/// Tam bir URL kodlayıcı değil ve olmasına gerek yok: burada tek iş, kullanıcı
/// sorgusunun sorgu dizesini bölmemesi. Ayrılmış karakterler ve ASCII olmayan
/// baytlar kodlanıyor.
fn urlencode(girdi: &str) -> String {
    let mut cikti = String::with_capacity(girdi.len());
    for bayt in girdi.as_bytes() {
        match bayt {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                cikti.push(*bayt as char)
            }
            b' ' => cikti.push('+'),
            _ => cikti.push_str(&format!("%{bayt:02X}")),
        }
    }
    cikti
}

/// Motordan gelen olaylar.
///
/// Motor `tabs`ı tanımıyor; bu trait ters yönde tek bağ. Sürücü zayıf
/// referansla veriliyor (`motor::Motor::dinleyici_ata`) — güçlü olsaydı
/// sürücü ↔ motor arasında bir döngü kurulur ve kapanışta ikisi de düşmezdi.
impl<R: Runtime> Dinleyici for Surucu<R> {
    fn baslik_degisti(&self, id: SekmeId, ham: String) {
        let kayit = {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else { return };
            // Sayfadan gelen metin ham basılmıyor (CLAUDE.md #8).
            s.baslik = super::baslik_temizle(&ham);
            (s.url.clone(), s.baslik.clone(), s.gizli)
        };

        // Geçmişteki başlığı tazele. `ziyaret_ekle` DEĞİL: başlık olayı
        // gezinmeden sonra geliyor ve tekrar ekleseydik her sayfa açılışı
        // sayacı iki artırırdı.
        //
        // Gizli sekme buraya da girmiyor: adres geçmişte zaten yok ama
        // `baslik_guncelle` aynı adrese normal bir sekmeden gelinmişse
        // o kaydı gizli sekmedeki başlıkla ezerdi.
        if let (Some(depo), false) = (self.gecmis.as_ref(), kayit.2) {
            let _ = depo.baslik_guncelle(&kayit.0, &kayit.1);
        }

        self.yayinla_sekme(id);
        self.oturum_isaretle();
    }

    fn adres_degisti(&self, id: SekmeId, url: String) {
        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else { return };
            s.bekleyen = Some(url);
        }
        self.yayinla_sekme(id);
    }

    fn gezinme(&self, id: SekmeId, asama: Asama, url: String, basarili: bool) {
        let mut gecmise_yazilacak: Option<(String, String)> = None;
        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else { return };
            match asama {
                Asama::Basladi => {
                    s.yukleniyor = true;
                    s.bekleyen = Some(url.clone());
                }
                Asama::Bitti => {
                    s.yukleniyor = false;
                    if basarili {
                        // Yalnız BAŞARILI gezinme kaydediliyor: geçmişe,
                        // oturuma ve sekme kaydına (CLAUDE.md #7).
                        s.url = url.clone();
                        s.bekleyen = None;
                        if s.gecmis.last().map(String::as_str) != Some(url.as_str()) {
                            s.gecmis.push(url.clone());
                            // Geri/ileri geçmişi 50 kayıtta duruyor
                            // (`docs/Sekmeler.md`).
                            if s.gecmis.len() > 50 {
                                s.gecmis.remove(0);
                            }
                            s.gecmis_konum = s.gecmis.len() - 1;
                        }
                        // Kilit ALTINDA yazmıyoruz: SQLite yazımı depo
                        // kilidini tutarken yapılırsa arayüz o süre boyunca
                        // sekme listesi alamıyor (dosyanın başındaki kilit
                        // kuralı). Kayıt kilit bırakıldıktan sonra gidiyor.
                        gecmise_yazilacak = Some((url.clone(), s.baslik.clone()));
                    } else {
                        // Başarısız gezinmede başlık eskisiyle kalmıyor.
                        s.baslik = String::new();
                    }
                }
            }
        }

        if matches!(asama, Asama::Bitti) {
            // Atılmışlıktan dönüşün kronometresi burada duruyor. Ölçülen şey
            // ilk boya DEĞİL, yüklemenin bitişi — yani gerçek ilk boyanın üst
            // sınırı (`memory/gecikme.rs`, `Kaynak::Atilmis`). WebView2 bize
            // bir ilk boya olayı vermiyor ve olmayan bir olayı varmış gibi
            // göstermektense dürüst bir üst sınır veriliyor.
            let mut defter = self.gecikme.lock().unwrap();
            if basarili {
                if let Some(ms) = defter.bitir(id, gecikme::Kaynak::Atilmis, Instant::now()) {
                    log::debug!("muiren: sekme {id} atılmışlıktan {ms} ms'de döndü");
                }
            } else {
                // CLAUDE.md #7: başarısız gezinme hiçbir yere yazılmıyor,
                // gecikme tablosu da bir yer. Sayılsaydı DNS hatasıyla anında
                // dönen bir sekme tabloyu olduğundan iyi gösterirdi.
                defter.iptal(id);
            }
        }

        if let Some((u, b)) = gecmise_yazilacak {
            self.gecmise_yaz(id, &u, &b);
        }

        let _ = self.app.emit(
            olaylar::GEZINME,
            GezinmeOlayi {
                id,
                asama,
                url,
                basarili,
            },
        );
        self.yayinla_sekme(id);
        if matches!(asama, Asama::Bitti) && basarili {
            // Atılmışlıktan dönen sekme kaldığı yere kayıyor. Bayrak
            // `take` ile tüketiliyor: kullanıcı sonra aynı adrese elle
            // giderse sayfa istenmediği hâlde kaymasın.
            let bekleyen = self
                .depo
                .lock()
                .unwrap()
                .bul_mut(id)
                .and_then(|s| s.kaydirma_bekliyor.take());
            if let Some(oran) = bekleyen {
                let _ = self.motor.kaydirma_uygula(id, oran);
            }
            self.oturum_isaretle();
        }
    }

    fn gecmis_degisti(&self, id: SekmeId, geri_var: bool, ileri_var: bool) {
        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else { return };
            s.geri_var = geri_var;
            s.ileri_var = ileri_var;
        }
        self.yayinla_sekme(id);
    }

    fn ses_degisti(&self, id: SekmeId, caliyor: bool) {
        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else { return };
            s.ses_caliyor = caliyor;
        }
        self.yayinla_sekme(id);
    }

    fn yeni_pencere(&self, ebeveyn: SekmeId, url: String, kullanici_baslatti: bool) {
        // Önce engelleme politikası: sayfanın kendi kendine açtığı pencere
        // (`kullanici_baslatti == false`) varsayılan olarak engelleniyor
        // (`engel/mod.rs`). Engellenen pencere **sessizce kaybolmuyor** —
        // olay yayınlanıyor ve arayüz "yine de aç" sunuyor.
        if let Some(sebep) = self
            .engelleyici
            .pencere_engelli_mi(&url, kullanici_baslatti)
        {
            let sayi = self.engelleyici.say(ebeveyn);
            let _ = self.app.emit(
                olaylar::ENGELLENDI,
                EngelOlayi {
                    id: ebeveyn,
                    url,
                    sebep,
                    sayi,
                },
            );
            return;
        }

        // `window.open` ve `target="_blank"` ayrı bir WebView2 penceresi
        // açardı; onu engelleyip kendi sekmemizi açıyoruz. Yeni sekme
        // ebeveynin ÇOCUĞU oluyor ve hemen sağına giriyor — ağaç sırasının
        // asıl kaynağı bu (`docs/Sekmeler.md`).
        let Some(surucu) = self.kendi_arc() else {
            return;
        };
        // Ayrı iş parçacığı ŞART: bu geri çağrı WebView2'nin olay döngüsünde,
        // yani ana iş parçacığında koşuyor. `sekme_ac` → `add_child` işi ana
        // iş parçacığına gönderip cevabını bekliyor; buradan doğrudan
        // çağrılsaydı uygulama kilitlenirdi.
        std::thread::spawn(move || {
            // Gizli sekmedeki bir bağlantı yeni sekmede de gizli açılıyor:
            // aksi hâlde `target="_blank"` bir bağlantı, kullanıcının hiç
            // istemediği bir anda gizliliği sessizce bırakırdı.
            let gizli = surucu
                .depo
                .lock()
                .unwrap()
                .bul(ebeveyn)
                .is_some_and(|s| s.gizli);
            let _ = surucu.sekme_ac(Some(url), Some(ebeveyn), true, gizli);
        });
    }

    fn kisayol(&self, _id: SekmeId, tus: String, ctrl: bool, shift: bool, alt: bool) {
        let Some(surucu) = self.kendi_arc() else {
            return;
        };
        // Ayrı iş parçacığı ŞART: bu geri çağrı WebView2'nin olay döngüsünde,
        // yani ana iş parçacığında koşuyor. `sekme_ac`/`sekme_etkinlestir`
        // işi ana iş parçacığına gönderip cevabını bekliyor; buradan doğrudan
        // çağrılsaydı uygulama kilitlenirdi (`yeni_pencere` ile aynı gerekçe).
        std::thread::spawn(move || {
            surucu.kisayol_uygula(&tus, ctrl, shift, alt);
        });
    }

    fn kaydirma_okundu(&self, id: SekmeId, oran: f32) {
        // Uyanma kronometresinin duracağı an: bu geri çağrı, sayfanın JS ana
        // iş parçacığının bizim betiğimizi ÇALIŞTIRDIĞI anlamına geliyor —
        // yani sekme gerçekten etkileşime hazır (`memory/gecikme.rs`).
        //
        // Bu olay üç yerden geliyor (uyanma, sekmeden çıkış, uyku öncesi) ve
        // yalnız ilkinde açık bir `Uyuyan` kronometresi bulunuyor; diğer
        // ikisinde `bitir` eşleşme bulamayıp `None` dönüyor.
        if let Some(ms) =
            self.gecikme
                .lock()
                .unwrap()
                .bitir(id, gecikme::Kaynak::Uyuyan, Instant::now())
        {
            log::debug!("muiren: sekme {id} uykudan {ms} ms'de döndü");
        }

        let mut depo = self.depo.lock().unwrap();
        let Some(s) = depo.bul_mut(id) else { return };
        s.kaydirma = oran.clamp(0.0, 1.0);
        drop(depo);
        // Oturuma yazılsın: atılmış sekme yeniden başlatmadan sonra da
        // kaldığı yerden gelsin.
        self.oturum_isaretle();
    }

    fn indirme_basladi(
        &self,
        id: SekmeId,
        url: String,
        dosya_adi: String,
        boyut: u64,
    ) -> IndirmeKarari {
        // **Bu gövde ucuz olmak zorunda.** WebView2 cevabı geri çağrı
        // bitmeden istiyor; buradaki her uzun iş sayfanın donması demek.
        // Yapılan: iki kilit okuması ve bir küme sorgusu.

        // Bilet: kullanıcı bu adres için "Muiren indirsin" demiş ve sekme
        // yeniden gönderilmişti. Tek atımlık — `remove` hem soruyor hem
        // tüketiyor.
        if self.indirme_biletleri.lock().unwrap().remove(&url) {
            return IndirmeKarari::Motorda;
        }

        let politika = self.ayarlar.lock().unwrap().indirme_politikasi;
        // Muiget kurulu değilse üç politika da aynı yere çıkıyor: motorun
        // kendi indirmesi. Karar #4'ün ikinci yarısı — "kullanıcı indirme
        // yapamaz duruma düşmüyor".
        if politika == IndirmePolitikasi::Motorda || !self.kopruler.muiget.kurulu() {
            return IndirmeKarari::Motorda;
        }

        // Ad motorun önerdiği tam yoldan geliyor ve kökeni sayfanın
        // `Content-Disposition` başlığı (CLAUDE.md #8).
        let temiz_ad = crate::bridge::dosya_adi_temizle(&dosya_adi);
        let kaynak_sayfa = self.depo.lock().unwrap().bul(id).map(|s| s.url.clone());

        match politika {
            IndirmePolitikasi::Muiget => {
                let kopruler = self.kopruler.clone();
                let app = self.app.clone();
                let (u, a, k) = (url.clone(), temiz_ad.clone(), kaynak_sayfa.clone());
                // Ayrı iş parçacığı ŞART: devir bir süreç doğurup yanıt
                // bekliyor (`bridge/muiget.rs`, 5 sn). Bunu WebView2'nin
                // olay döngüsünde yapmak sayfayı o süre boyunca dondururdu
                // (`yeni_pencere` ile aynı gerekçe).
                std::thread::spawn(move || {
                    let sonuc = kopruler.muiget.devret(Yuk::Indirme {
                        url: u.clone(),
                        dosya_adi: a.clone(),
                        kaynak_sayfa: k,
                    });
                    // Başarı da hata da arayüze gidiyor: köprü kırıldığında
                    // kullanıcı tarayıcıyı suçlamasın (`docs/Kopruler.md`).
                    let _ = app.emit(
                        olaylar::INDIRME_ONERISI,
                        IndirmeOlayi {
                            id,
                            url: u,
                            dosya_adi: a,
                            boyut,
                            sonuc: match &sonuc {
                                Ok(()) => IndirmeSonucu::Devredildi,
                                Err(_) => IndirmeSonucu::Basarisiz,
                            },
                        },
                    );
                });
                IndirmeKarari::Iptal
            }
            IndirmePolitikasi::Sor => {
                let _ = self.app.emit(
                    olaylar::INDIRME_ONERISI,
                    IndirmeOlayi {
                        id,
                        url,
                        dosya_adi: temiz_ad,
                        boyut,
                        sonuc: IndirmeSonucu::Soruluyor,
                    },
                );
                // İptal ediliyor ve bu bir tercih: kullanıcı "Muiren
                // indirsin" derse `indirme_izin_ver` aynı adrese yeniden
                // gidiyor. Alternatif (indirmeyi başlatıp sormak) dosyanın
                // iki kez inmesi olurdu.
                IndirmeKarari::Iptal
            }
            IndirmePolitikasi::Motorda => IndirmeKarari::Motorda,
        }
    }

    fn gezinme_izni(
        &self,
        id: SekmeId,
        url: &str,
        kullanici_baslatti: bool,
        yonlendirme: bool,
    ) -> bool {
        // Kabuk sayfaları politikanın dışında: `muiren://yeni` zaten motora
        // gitmiyor ama bir gün giderse kendi sayfamızı engellemeyelim.
        if kabuk_sayfasi(url) {
            return true;
        }
        let Some(sebep) = self
            .engelleyici
            .gezinme_engelli_mi(url, kullanici_baslatti, yonlendirme)
        else {
            // Yeni sayfaya geçiliyor: rozet **bu sayfada** kaç istek
            // engellendiğini gösteriyor, sekmenin ömrü boyunca kaçını
            // değil.
            if !yonlendirme {
                self.engelleyici.sayaci_sifirla(id);
            }
            return true;
        };

        let sayi = self.engelleyici.say(id);
        let _ = self.app.emit(
            olaylar::ENGELLENDI,
            EngelOlayi {
                id,
                url: url.to_string(),
                sebep,
                sayi,
            },
        );
        false
    }

    fn istek_izni(&self, id: SekmeId, url: &str) -> bool {
        // **Sıcak yol** (sayfa başına yüzlerce çağrı). Filtre kapalıysa
        // `istek_engelli_mi` tek bir `bool` okumasıyla dönüyor.
        if !self.engelleyici.istek_engelli_mi(url) {
            return true;
        }
        // Engellenen her alt kaynak için olay yayınlanmıyor — bir sayfada
        // yüzlerce olabiliyor ve arayüzü olay yağmuruna tutmanın karşılığı
        // yok. Sayaç artıyor; rozet onu okuyor.
        self.engelleyici.say(id);
        false
    }

    fn istek_suzgeci_acik(&self) -> bool {
        self.engelleyici.ozet().filtre_acik
    }

    fn tam_ekran_degisti(&self, id: SekmeId, tam_ekran: bool) {
        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else { return };
            s.tam_ekran = tam_ekran;
        }
        // Koruma kuralı #8 artık gerçekten çalışıyor: alan uzun süre
        // `SekmeOzeti` içinde vardı ama hiç doldurulmuyordu.
        self.yayinla_sekme(id);
    }

    fn favicon_geldi(&self, id: SekmeId, veri: Vec<u8>) {
        // Gizli sekmenin ikonu diske yazılmıyor. İkon "içerik" gibi
        // görünmüyor ama depoda kalan bir favicon, kullanıcının hangi siteye
        // gittiğinin diskteki kaydı — gizliliğin üçüncü sızıntı yolu
        // (`tabs::Sekme::gizli`). Bedeli: gizli sekmede ikon yerine durum
        // noktası kalıyor.
        if self.depo.lock().unwrap().bul(id).is_some_and(|s| s.gizli) {
            return;
        }
        // Disk yazımı depo kilidi ALTINDA yapılmıyor (dosyanın başındaki
        // kilit kuralı): favicon'lar gezinme sırasında geliyor ve o an
        // arayüzün sekme listesi alması gerekiyor.
        let Ok(kimlik) = favicon::yaz(&self.favicon_dizini, &veri) else {
            // Bozuk ya da PNG olmayan ikon sessizce düşüyor: sekme
            // çubuğunda ikon yok, o kadar.
            return;
        };
        {
            let mut depo = self.depo.lock().unwrap();
            let Some(s) = depo.bul_mut(id) else { return };
            if s.favicon.as_deref() == Some(kimlik.as_str()) {
                return;
            }
            s.favicon = Some(kimlik);
        }
        self.yayinla_sekme(id);
        self.oturum_isaretle();
    }
}

/// Veri temizleme isteği (`docs/IPC.md`, `veri_temizle`).
///
/// Kalemler ayrı ayrı çünkü kullanıcının niyeti ayrı: "bu siteden çıkmak
/// istiyorum" (çerez) ile "diskimi boşaltmak istiyorum" (önbellek) aynı şey
/// değil ve ikisini tek düğmede birleştirmek, ikisini de yapmaktan korkulan
/// bir düğme üretiyor.
#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Temizlik {
    /// Muiren'in geçmiş veritabanı. **Yer imleri dahil değil** — onlar
    /// kullanıcının kasten sakladığı şeyler ve "geçmişi temizle" ile
    /// gitmeleri kullanıcıyı kaybettiriyor.
    pub gecmis: bool,
    /// Muiren'in favicon deposu.
    pub favicon: bool,
    pub cerezler: bool,
    pub onbellek: bool,
    pub site_verisi: bool,
    pub otomatik_doldurma: bool,
}

impl Temizlik {
    fn motor_kalemleri(self) -> crate::motor::MotorTemizligi {
        crate::motor::MotorTemizligi {
            cerezler: self.cerezler,
            onbellek: self.onbellek,
            site_verisi: self.site_verisi,
            otomatik_doldurma: self.otomatik_doldurma,
        }
    }
}

/// Ne yapıldığının raporu. Arayüz "3 214 geçmiş kaydı silindi" diyor;
/// "temizlendi" demek, hiçbir şey silinmediğinde de doğru görünürdü.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemizlikRaporu {
    pub gecmis_kaydi: u64,
    pub favicon_dosyasi: u64,
    /// Motorun silmesi **eşzamansız**: burada yalnız "başlatıldı" var.
    /// Kesin sayı vermek, olmayan bir kesinlik iddia etmek olurdu.
    pub motor_baslatildi: bool,
}

/// `muiren://engellendi` yükü (`docs/IPC.md`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EngelOlayi {
    id: SekmeId,
    url: String,
    sebep: EngelSebebi,
    /// Bu sayfada engellenen toplam istek sayısı; adres çubuğundaki rozet.
    sayi: u32,
}

/// `muiren://indirme-onerisi` yükü (`docs/IPC.md`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct IndirmeOlayi {
    id: SekmeId,
    url: String,
    /// Temizlenmiş ad; `None` ise motorun önerdiği addan geriye bir şey
    /// kalmamış (`bridge::dosya_adi_temizle`).
    dosya_adi: Option<String>,
    /// Sunucunun bildirdiği boyut; bilinmiyorsa `0`.
    boyut: u64,
    sonuc: IndirmeSonucu,
}

/// İndirme olayının hangi anı olduğu.
///
/// Tek bir olayla üç durumun da anlatılması bilinçli: arayüzün üç ayrı
/// dinleyici yazması, birinin unutulmasıyla sessizce eksik kalırdı.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum IndirmeSonucu {
    /// `Sor` politikası: kullanıcı karar verecek.
    Soruluyor,
    /// Muiget kabul etti.
    Devredildi,
    /// Devir başarısız; motorun indirmesi de iptal edilmişti.
    Basarisiz,
}

/// [`Dinleyici`] uygulaması `Arc<Self>` isteyen metotları çağırabilsin diye.
///
/// Sürücü Tauri durumunda `Arc` olarak duruyor; oradan geri almak, ikinci bir
/// zayıf referans alanı tutmaktan basit.
impl<R: Runtime> Surucu<R> {
    fn kendi_arc(&self) -> Option<Arc<Surucu<R>>> {
        use tauri::Manager;
        self.app
            .try_state::<Arc<Surucu<R>>>()
            .map(|s| s.inner().clone())
    }
}
