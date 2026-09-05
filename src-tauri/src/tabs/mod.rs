//! Sekme kaydı ve deposu.
//!
//! Mimarinin en önemli cümlesi (`docs/Architecture.md`, `docs/Sekmeler.md`):
//!
//! > **Sekme bir veri kaydıdır. Webview ise o kaydın pahalı, isteğe bağlı, her
//! > an yok edilebilir bir eki.**
//!
//! Bu modül kaydı tutuyor; webview'leri `motor/` tutuyor. Ayrım olmadan "500
//! sekme" mümkün değil: [`Sekme`] birkaç yüz bayt, webview 30–60 MB taban
//! maliyet.
//!
//! Alt modüller:
//!
//! - [`agac`] — sıra ve ağaç kararları, **saf**.
//! - [`durum`] — dört durumlu makine, **saf**.
//! - [`oturum`] — diske yazma/okuma.
//! - [`surucu`] — depo + motor + olay yayını; kararları yukarıdaki saf
//!   modüllere soruyor.

pub mod agac;
pub mod durum;
pub mod grup;
pub mod oturum;
pub mod surucu;

use std::collections::VecDeque;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use agac::Dugum;
pub use durum::Durum;
pub use grup::{Grup, GrupRengi};

/// Uygulama ömrü boyunca tekil, **asla yeniden kullanılmayan** sekme kimliği.
///
/// Yeniden kullanılsaydı: gözcü iş parçacığı ile arayüz arasında her zaman
/// birkaç yüz milisaniye gecikme var; gözcünün "sekme 7'yi uyut" kararı
/// kullanıcının o arada açtığı yeni 7 numaralı sekmeye çarpardı. Bu tür
/// hatalar tekrarlanamaz ve haftalarca bulunamaz (`docs/Sekmeler.md`).
pub type SekmeId = u64;
pub type GrupId = u64;

/// Kabuğun kendi yeni sekme sayfası. Ağ isteği yok, öneri akışı yok,
/// **webview yok** — sayfayı kabuk çiziyor, dolayısıyla yeni sekmenin render
/// maliyeti sıfır (`docs/Sekmeler.md`, "Yeni sekme sayfası").
pub const YENI_SEKME: &str = "muiren://yeni";

/// Bu adresi motor değil kabuk açıyor.
pub fn kabuk_sayfasi(url: &str) -> bool {
    url.starts_with("muiren://")
}

/// Adres çubuğundan gezinilebilen şemalar. Listede olmayan her şey **arama**.
///
/// `javascript:` ve `data:` bilerek dışarıda: ikisi de kullanıcıya "şunu
/// adres çubuğuna yapıştır" dedirten saldırıların taşıyıcısı. Hata vermek
/// yerine aramaya düşüyorlar — kullanıcının yapıştırdığı metin bir arama
/// sonucu üretiyor, sayfada kod çalıştırmıyor.
const IZINLI_SEMALAR: [&str; 3] = ["http", "https", "file"];

/// Girdinin başındaki şema (`https`, `file`, `javascript`…).
///
/// `konak:port` yazımını şema **saymıyor**: `localhost:3000` geçerli bir şema
/// deseni ve ayırt edilmezse geliştiricinin en çok yazdığı adres aramaya
/// giderdi. Ayıran şey iki nokta üst üsteden sonra yalnız rakam olması.
pub fn sema_oneki(metin: &str) -> Option<String> {
    let iki_nokta = metin.find(':')?;
    let sema = &metin[..iki_nokta];
    let mut harfler = sema.chars();
    if !harfler.next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    if !harfler.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
        return None;
    }
    // `konak:port` mu?
    let kalan = &metin[iki_nokta + 1..];
    let rakamlar = kalan.len() - kalan.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    if rakamlar > 0 {
        let sonrasi = kalan[rakamlar..].chars().next();
        if matches!(sonrasi, None | Some('/') | Some('?') | Some('#')) {
            return None;
        }
    }
    Some(sema.to_ascii_lowercase())
}

/// Nokta ile ayrılmış dört sayı — `192.168.1.5`.
fn ipv4_mi(konak: &str) -> bool {
    let mut parca = 0;
    for p in konak.split('.') {
        if p.is_empty() || p.len() > 3 || !p.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        parca += 1;
    }
    parca == 4
}

/// Kullanıcının yazdığı şey adres mi, arama mı.
///
/// Aynı ayrım arayüzde de var (`src/lib/url.ts`, `adresMi`) ama oradaki kopya
/// **yalnız canlı geri bildirim** için: kullanıcı yazarken kilit/vurgu
/// çiziliyor. Motora ne gideceğine karar veren yer burası — kabuk kapalıyken
/// de (oturum geri yükleme, köprü, kısayol) doğru çalışmak zorunda
/// (CLAUDE.md, "karar veren kod backend'de").
///
/// `?` ile başlayan girdi **zorla arama**: `localhost` gibi belirsiz
/// durumlarda kullanıcının tek çıkış yolu bu.
pub fn adres_mi(girdi: &str) -> bool {
    let metin = girdi.trim();
    if metin.is_empty() || metin.starts_with('?') {
        return false;
    }
    if let Some(sema) = sema_oneki(metin) {
        return IZINLI_SEMALAR.contains(&sema.as_str());
    }
    // Boşluk varsa arama. `ornek com` bir adres değil.
    if metin.chars().any(char::is_whitespace) {
        return false;
    }

    let otorite = metin.split(['/', '?', '#']).next().unwrap_or("");
    let konak = match otorite.rsplit_once('@') {
        Some((_, k)) => k,
        None => otorite,
    };
    // Port yalnız rakamsa atılıyor; `ornek.com:sayfa` zaten adres değil.
    let konak = match konak.rsplit_once(':') {
        Some((k, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => k,
        _ => konak,
    };

    if konak.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if ipv4_mi(konak) {
        return true;
    }

    let etiketler: Vec<&str> = konak.split('.').collect();
    if etiketler.len() < 2 {
        return false;
    }
    // Üst düzey alan adı harf ve en az iki karakter; `1.2` ya da `sürüm.3`
    // adres değil.
    let son = etiketler[etiketler.len() - 1];
    son.chars().count() >= 2 && son.chars().all(char::is_alphabetic)
}

/// Sekme başlığının üst sınırı. `document.title` saldırganın yazdığı bir dize;
/// sınırsız uzunlukta bir başlık sekme çubuğunu ve oturum dosyasını şişirir.
const BASLIK_SINIRI: usize = 200;

/// Sayfadan gelen metni arayüze basılabilir hâle getirir.
///
/// `document.title` **güvenilmez** (CLAUDE.md #8). İki şey yapılıyor:
///
/// - 200 karakterde kırpma.
/// - Satır sonu, sıfır genişlikli karakter ve iki yönlü metin kontrol
///   karakterlerinin (`U+202E` vb.) temizlenmesi. Bunlar temizlenmezse sekme
///   çubuğunda metin ters akıyor ve sahte bir alan adı gösterilebiliyor.
///
/// Aynısı favicon adresi, indirme dosya adı ve `window.open` ile gelen ad için
/// de geçerli — hepsi buradan geçiyor.
pub fn baslik_temizle(ham: &str) -> String {
    let temiz: String = ham
        .chars()
        .filter(|c| {
            // Kontrol karakterleri (satır sonu, sekme, ESC ...) düşüyor.
            if c.is_control() {
                return false;
            }
            match *c {
                // İki yönlü metin denetimi: LRM/RLM, LRE..RLO, LRI..PDI.
                '\u{200E}' | '\u{200F}' => false,
                '\u{202A}'..='\u{202E}' => false,
                '\u{2066}'..='\u{2069}' => false,
                // Sıfır genişlikli birleştirici/ayırıcı ve BOM.
                '\u{200B}'..='\u{200D}' => false,
                '\u{FEFF}' => false,
                _ => true,
            }
        })
        .collect();

    let kirpilmis: String = temiz.trim().chars().take(BASLIK_SINIRI).collect();
    kirpilmis.trim_end().to_string()
}

/// Bir sekmenin bütün hâli. Birkaç yüz bayt.
#[derive(Debug, Clone)]
pub struct Sekme {
    pub id: SekmeId,
    /// Son **başarılı** gezinme. Başarısız gezinme buraya yazılmıyor
    /// (CLAUDE.md #7). Oturuma ve geçmişe giden alan bu.
    pub url: String,
    /// Devam eden ya da başarısız olmuş gezinmenin adresi.
    ///
    /// Adres çubuğu bunu gösteriyor: bir sayfa açılamadığında kullanıcının
    /// adres çubuğunda eski adresi görmesi kafa karıştırıcı olurdu. Ama bu
    /// alan oturuma ve geçmişe **girmiyor** — orası yalnız `url`.
    pub bekleyen: Option<String>,
    /// Sayfadan geldi → güvenilmez. [`baslik_temizle`] geçmeden atanmıyor.
    pub baslik: String,
    /// Faz 3'te `favicon/` deposundaki dosyanın kimliği olacak. Faz 1'de
    /// doldurulmuyor: uzak bir adresi doğrudan kabuğa basmak, açılan her
    /// sitenin kabuk isteği yaptırabilmesi demek olurdu.
    pub favicon: Option<String>,
    pub gecmis: Vec<String>,
    pub gecmis_konum: usize,
    /// 0.0–1.0. Atılmış sekmeyi geri getirirken kullanılıyor.
    ///
    /// Piksel değil **oran**: sekme geri geldiğinde pencere boyutu ya da
    /// sayfanın kendisi değişmiş olabiliyor; 4200 piksel o sayfada artık
    /// olmayabilir ama "%38" her zaman bir yere denk geliyor.
    pub kaydirma: f32,
    /// Bu sekme atılmışlıktan yeni döndü ve yüklenince kaydırılacak.
    ///
    /// Geçici: diske yazılmıyor ve oturumdan gelmiyor. Bir bayrak olmadan
    /// "kullanıcı aynı adrese elle gitti" ile "atılmış sekme geri geldi"
    /// ayırt edilemiyor ve ilkinde sayfa istenmediği hâlde kayardı.
    pub kaydirma_bekliyor: Option<f32>,
    /// Gizli sekme (`docs/Roadmap.md` Faz 3).
    ///
    /// Webview `SetIsInPrivateModeEnabled` ile doğuyor: çerezler, oturum
    /// depolaması ve önbellek bellekte kalıyor, sekme kapanınca gidiyor.
    /// Muiren tarafında üç kapı ayrıca kapalı — **geçmişe yazılmıyor**,
    /// **oturum dosyasına girmiyor**, **favicon deposuna dokunmuyor**.
    /// Motorun gizliliği yetmezdi: üçü de bizim diskimize yazan yollar.
    pub gizli: bool,
    /// "Bu sekme hangi sekmeden açıldı." Ağacın tek kaynağı.
    pub ebeveyn: Option<SekmeId>,
    pub grup: Option<GrupId>,
    pub sabit: bool,
    /// Kullanıcı bu sekmeyi "uyutma" diye işaretledi (koruma kuralı #5).
    pub uyutma_istisnasi: bool,
    pub durum: Durum,
    pub son_etkinlik: Instant,
    pub ses_caliyor: bool,
    pub sessiz: bool,
    /// Doldurulmuş form var → **atılmaz**, uyutulabilir (koruma kuralı #4).
    pub form_dolu: bool,
    /// Sayfa tam ekran bir öğe gösteriyor (koruma kuralı #8).
    pub tam_ekran: bool,
    pub yukleniyor: bool,
    pub geri_var: bool,
    pub ileri_var: bool,
}

impl Sekme {
    fn yeni(id: SekmeId, url: String, ebeveyn: Option<SekmeId>, gizli: bool) -> Self {
        Sekme {
            id,
            baslik: String::new(),
            favicon: None,
            gecmis: vec![url.clone()],
            gecmis_konum: 0,
            url,
            bekleyen: None,
            kaydirma: 0.0,
            kaydirma_bekliyor: None,
            gizli,
            ebeveyn,
            grup: None,
            sabit: false,
            uyutma_istisnasi: false,
            durum: Durum::Atilmis,
            son_etkinlik: Instant::now(),
            ses_caliyor: false,
            sessiz: false,
            form_dolu: false,
            tam_ekran: false,
            yukleniyor: false,
            geri_var: false,
            ileri_var: false,
        }
    }

    /// Adres çubuğunun göstereceği adres: devam eden gezinme varsa o, yoksa
    /// son başarılı adres.
    pub fn gosterilen_adres(&self) -> String {
        self.bekleyen.clone().unwrap_or_else(|| self.url.clone())
    }

    /// Sekme çubuğunda gösterilecek ad. Başlık yoksa adresin alan adı.
    pub fn gorunen_ad(&self) -> String {
        if !self.baslik.is_empty() {
            return self.baslik.clone();
        }
        let adres = self.gosterilen_adres();
        if adres == YENI_SEKME {
            return String::new();
        }
        url::Url::parse(&adres)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or(adres)
    }
}

/// Arayüze giden sekme özeti.
///
/// `docs/IPC.md`: alanlar `camelCase`. Uyuyan sekmenin özetini üretmek onu
/// **uyandırmıyor** — bütün alanlar zaten bellekteki kayıttan geliyor
/// (CLAUDE.md #5).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SekmeOzeti {
    pub id: SekmeId,
    pub url: String,
    pub baslik: String,
    pub gorunen_ad: String,
    pub favicon: Option<String>,
    pub durum: Durum,
    pub ebeveyn: Option<SekmeId>,
    /// Ağaçtaki derinlik. Sekme çubuğu girintiyi buradan çiziyor; hesabı
    /// arayüzde yapmak, arayüze ağaç mantığı koymak olurdu (CLAUDE.md #2).
    pub derinlik: u32,
    pub sabit: bool,
    pub uyutma_istisnasi: bool,
    pub ses_caliyor: bool,
    pub sessiz: bool,
    /// Doldurulmuş form var → **atılmaz**, uyutulabilir (koruma kuralı #4).
    /// Arayüz bunu ipucunda gösteriyor: sekmenin neden atılmadığı görünsün.
    pub form_dolu: bool,
    /// Gizli sekme. Arayüz sekme çubuğunda ayrı bir işaret çiziyor —
    /// kullanıcının hangi sekmenin gizli olduğunu **görmesi** gerekiyor,
    /// yoksa gizli sandığı sekmede geçmişe yazan bir arama yapıyor.
    pub gizli: bool,
    pub grup: Option<GrupId>,
    /// Grubu katlanmış mı. Arayüz sekmeyi çizmiyor; **backend yine de
    /// gönderiyor** çünkü katlama bir görüntü tercihi ve arayüzün elinde tam
    /// liste olmadan "grubu aç" düğmesi kaç sekme olduğunu söyleyemez.
    pub katli: bool,
    /// Bu sekmeye uygulanacak uyku eşiği, grubundan geliyorsa.
    ///
    /// **Eşik kararına `esik.rs` üzerinden giriyor** ve o dosya saf kalmak
    /// zorunda: grup tablosunu oraya taşımak yerine, etkin değer burada
    /// hesaplanıp özetle birlikte gidiyor. `None` ise genel ayar geçerli.
    pub grup_uyku_esigi_sn: Option<u64>,
    /// Sayfa tam ekran bir öğe gösteriyor (video, oyun) → korumalı
    /// (koruma kuralı #8).
    pub tam_ekran: bool,
    pub yukleniyor: bool,
    pub geri_var: bool,
    pub ileri_var: bool,
    pub etkin: bool,
    /// Kaç saniyedir dokunulmadı. Bellek paneli ve ipuçları için.
    pub bosta_sn: u64,
}

/// Kapatılan sekmelerin geri alma listesi. `Ctrl+Shift+T`.
///
/// Kayıt zaten küçük olduğu için 25 tanesini bellekte tutmanın maliyeti yok.
/// Uygulama kapanınca liste siliniyor — çöpü diske yazmıyoruz
/// (`docs/Sekmeler.md`).
const GERI_ALMA_SINIRI: usize = 25;

/// Sekmelerin tek kaynağı. Motor çağrısı **yok**: bu tip kilit altında
/// tutuluyor ve kilidi elinde tutarken webview yaratmak, arayüzü gözcünün
/// turuna kilitlemek demek olurdu.
#[derive(Debug, Default)]
pub struct Depo {
    sekmeler: Vec<Sekme>,
    etkin: Option<SekmeId>,
    sayac: SekmeId,
    kapatilanlar: VecDeque<Sekme>,
    // --- gruplar (Faz 5) ---
    gruplar: Vec<Grup>,
    /// Grup kimlikleri de **yeniden kullanılmıyor**, sekme kimlikleriyle aynı
    /// gerekçe: gözcü ile arayüz arasındaki gecikmede silinen bir grubun
    /// kimliği yeni bir gruba çarpardı.
    grup_sayaci: GrupId,
}

impl Depo {
    pub fn yeni() -> Self {
        Depo::default()
    }

    pub fn bos_mu(&self) -> bool {
        self.sekmeler.is_empty()
    }

    pub fn sayi(&self) -> usize {
        self.sekmeler.len()
    }

    pub fn etkin(&self) -> Option<SekmeId> {
        self.etkin
    }

    pub fn bul(&self, id: SekmeId) -> Option<&Sekme> {
        self.sekmeler.iter().find(|s| s.id == id)
    }

    pub fn bul_mut(&mut self, id: SekmeId) -> Option<&mut Sekme> {
        self.sekmeler.iter_mut().find(|s| s.id == id)
    }

    pub fn hepsi(&self) -> &[Sekme] {
        &self.sekmeler
    }

    /// Toplu güncelleme için (veri temizleme favicon kimliklerini düşürüyor).
    /// Tek tek `bul_mut` çağırmak listeyi her sekme için baştan tarardı.
    pub fn hepsi_mut(&mut self) -> &mut [Sekme] {
        &mut self.sekmeler
    }

    pub fn dugumler(&self) -> Vec<Dugum> {
        self.sekmeler
            .iter()
            .map(|s| Dugum {
                id: s.id,
                ebeveyn: s.ebeveyn,
                sabit: s.sabit,
            })
            .collect()
    }

    /// Yeni sekme. Konumu [`agac::ekle_konumu`] belirliyor.
    ///
    /// Sekme `Atilmis` doğuyor; webview'i sürücü, gerekiyorsa, ayrıca açıyor.
    /// Bu sıra önemli: "sekme var ama webview'i yok" normal bir durum, tersi
    /// değil.
    pub fn ac(&mut self, url: String, ebeveyn: Option<SekmeId>, gizli: bool) -> SekmeId {
        self.sayac += 1;
        let id = self.sayac;
        let konum = agac::ekle_konumu(&self.dugumler(), ebeveyn);
        self.sekmeler
            .insert(konum, Sekme::yeni(id, url, ebeveyn, gizli));
        id
    }

    /// Oturumdan gelen hazır kaydı ekler. Id sayacı en büyük id'nin üstünde
    /// tutuluyor ki geri yüklenen bir kimlik yeniden dağıtılmasın.
    pub fn ekle_ham(&mut self, sekme: Sekme) {
        self.sayac = self.sayac.max(sekme.id);
        self.sekmeler.push(sekme);
    }

    /// Kapatır ve yeni etkin sekmenin id'sini döndürür.
    pub fn kapat(&mut self, id: SekmeId) -> Option<SekmeId> {
        let dugumler = self.dugumler();
        let yeni_etkin = agac::kapatinca_etkin(&dugumler, id);

        let Some(i) = self.sekmeler.iter().position(|s| s.id == id) else {
            return self.etkin;
        };

        let mut dugumler = self.dugumler();
        agac::kapatinca_ebeveyn_devri(&mut dugumler, id);
        for d in &dugumler {
            if let Some(s) = self.bul_mut(d.id) {
                s.ebeveyn = d.ebeveyn;
            }
        }

        let kapanan = self.sekmeler.remove(i);
        // Yeni sekme sayfası geri alınmıyor: kaybedilen bir şey yok.
        if kapanan.url != YENI_SEKME {
            self.kapatilanlar.push_front(kapanan);
            self.kapatilanlar.truncate(GERI_ALMA_SINIRI);
        }

        if self.etkin == Some(id) {
            self.etkin = yeni_etkin;
        }
        self.etkin
    }

    /// Son kapatılan sekmenin kaydını çıkarır. Sürücü onu yeniden açıyor.
    pub fn geri_al(&mut self) -> Option<Sekme> {
        self.kapatilanlar.pop_front()
    }

    /// Etkin sekmeyi değiştirir; eski etkinin id'sini döndürür.
    ///
    /// Eski sekmeyi **uyutma kararı burada yok** — onu gözcü veriyor, çünkü
    /// "3 saniyeliğine başka sekmeye baktı" ile "40 dakikadır açmadı" farklı
    /// şeyler ve bu farkı bilen tek yer gözcünün zamanlayıcısı
    /// (`docs/Architecture.md`).
    pub fn etkinlestir(&mut self, id: SekmeId) -> Option<SekmeId> {
        let eski = self.etkin;
        if self.bul(id).is_none() {
            return eski;
        }
        self.etkin = Some(id);
        if let Some(s) = self.bul_mut(id) {
            s.son_etkinlik = Instant::now();
            s.durum = durum::gecis(s.durum, durum::Olay::Etkinlestirildi);
        }
        if let Some(e) = eski.filter(|e| *e != id) {
            if let Some(s) = self.bul_mut(e) {
                s.durum = durum::gecis(s.durum, durum::Olay::OdakGitti);
                s.son_etkinlik = Instant::now();
            }
        }
        eski
    }

    pub fn tasi(&mut self, id: SekmeId, hedef: usize) {
        let yeni_sira = agac::tasi(&self.dugumler(), id, hedef);
        let mut sirali: Vec<Sekme> = Vec::with_capacity(self.sekmeler.len());
        for d in &yeni_sira {
            if let Some(i) = self.sekmeler.iter().position(|s| s.id == d.id) {
                sirali.push(self.sekmeler.remove(i));
            }
        }
        sirali.append(&mut self.sekmeler);
        self.sekmeler = sirali;
    }

    /// Sabitler/çözer ve sekmeyi sabit bloğunun ucuna taşır.
    pub fn sabitle(&mut self, id: SekmeId, sabit: bool) {
        let Some(s) = self.bul_mut(id) else { return };
        s.sabit = sabit;
        // Sabitlenen sekmenin ağaç bağı kopuyor: sabit bir sekme kalıcı
        // olsun diye sabitlendi, bir başkasının çocuğu olarak dolaşması
        // beklenmiyor.
        if sabit {
            s.ebeveyn = None;
        }
        let sinir = agac::sabit_siniri(&self.dugumler());
        let hedef = if sabit {
            sinir.saturating_sub(1)
        } else {
            sinir
        };
        self.tasi(id, hedef);
    }

    pub fn durum_ata(&mut self, id: SekmeId, yeni: Durum) {
        if let Some(s) = self.bul_mut(id) {
            s.durum = yeni;
        }
    }

    fn derinlik(&self, sekme: &Sekme) -> u32 {
        let mut derinlik = 0;
        let mut sirada = sekme.ebeveyn;
        // Üst sınır: bozuk bir ebeveyn zinciri (dosyadan gelen oturum) sonsuz
        // döngüye çevirmesin.
        while let Some(e) = sirada {
            derinlik += 1;
            if derinlik > 32 {
                break;
            }
            sirada = self.bul(e).and_then(|s| s.ebeveyn);
        }
        derinlik
    }

    /// Arayüze giden tam liste.
    ///
    /// `docs/IPC.md`: `sekme-degisti` olayı kısmi güncelleme değil tam liste
    /// gönderiyor. Delta daha ucuz ama sıra değişimlerinde arayüzle backend
    /// **sessizce** ayrışabiliyor; kayıtlar küçük olduğu için doğruluk burada
    /// ucuz.
    pub fn ozetler(&self) -> Vec<SekmeOzeti> {
        let simdi = Instant::now();
        self.sekmeler
            .iter()
            .map(|s| SekmeOzeti {
                id: s.id,
                url: s.gosterilen_adres(),
                baslik: s.baslik.clone(),
                gorunen_ad: s.gorunen_ad(),
                favicon: s.favicon.clone(),
                durum: s.durum,
                ebeveyn: s.ebeveyn,
                derinlik: self.derinlik(s),
                sabit: s.sabit,
                uyutma_istisnasi: s.uyutma_istisnasi,
                ses_caliyor: s.ses_caliyor,
                sessiz: s.sessiz,
                form_dolu: s.form_dolu,
                gizli: s.gizli,
                grup: s.grup,
                katli: s
                    .grup
                    .is_some_and(|g| self.grup_bul(g).is_some_and(|x| x.katli)),
                grup_uyku_esigi_sn: s
                    .grup
                    .and_then(|g| self.grup_bul(g).and_then(|x| x.uyku_esigi_sn)),
                tam_ekran: s.tam_ekran,
                yukleniyor: s.yukleniyor,
                geri_var: s.geri_var,
                ileri_var: s.ileri_var,
                etkin: self.etkin == Some(s.id),
                bosta_sn: simdi.saturating_duration_since(s.son_etkinlik).as_secs(),
            })
            .collect()
    }

    pub fn ozet(&self, id: SekmeId) -> Option<SekmeOzeti> {
        self.ozetler().into_iter().find(|o| o.id == id)
    }

    // ------------------------------------------------------------- gruplar

    pub fn gruplar(&self) -> &[Grup] {
        &self.gruplar
    }

    pub fn grup_bul(&self, id: GrupId) -> Option<&Grup> {
        self.gruplar.iter().find(|g| g.id == id)
    }

    /// Oturumdan gelen grupları yükler.
    ///
    /// Sayaç en büyük kimliğin üstünde tutuluyor: `ekle_ham` ile aynı gerekçe
    /// — geri yüklenen bir kimlik yeniden dağıtılmamalı.
    pub fn gruplari_yukle(&mut self, gruplar: Vec<Grup>) {
        for g in gruplar {
            self.grup_sayaci = self.grup_sayaci.max(g.id);
            self.gruplar.push(g);
        }
    }

    pub fn grup_ac(&mut self, ad: &str) -> GrupId {
        self.grup_sayaci += 1;
        let id = self.grup_sayaci;
        self.gruplar.push(Grup::yeni(id, ad));
        id
    }

    /// Grubu siler. Sekmeler **kalıyor**, yalnız gruptan çıkıyorlar.
    ///
    /// Sekmeleri de kapatmak "grubu kapat" olurdu ve o ayrı bir eylem;
    /// ikisini tek düğmede birleştirmek, basılmaya korkulan bir düğme
    /// üretiyor.
    pub fn grup_sil(&mut self, id: GrupId) {
        self.gruplar.retain(|g| g.id != id);
        for s in &mut self.sekmeler {
            if s.grup == Some(id) {
                s.grup = None;
            }
        }
    }

    /// Sekmeyi gruba alır ya da (`None` ile) gruptan çıkarır.
    ///
    /// Var olmayan bir gruba atama **sessizce yok sayılmıyor**: sekme
    /// gruptan çıkıyor. Aksi hâlde arayüz ile depo, kimsenin göremediği bir
    /// grup kimliği üzerinde ayrışırdı.
    pub fn grup_ata(&mut self, sekme: SekmeId, grup: Option<GrupId>) {
        let gecerli = grup.filter(|g| self.gruplar.iter().any(|x| x.id == *g));
        if let Some(s) = self.bul_mut(sekme) {
            s.grup = gecerli;
        }
    }

    pub fn grup_guncelle(
        &mut self,
        id: GrupId,
        ad: Option<&str>,
        renk: Option<GrupRengi>,
        katli: Option<bool>,
        uyku_esigi_sn: Option<Option<u64>>,
    ) {
        let Some(g) = self.gruplar.iter_mut().find(|g| g.id == id) else {
            return;
        };
        if let Some(a) = ad {
            g.ad = grup::ad_temizle(a);
        }
        if let Some(r) = renk {
            g.renk = r;
        }
        if let Some(k) = katli {
            g.katli = k;
        }
        if let Some(e) = uyku_esigi_sn {
            g.uyku_esigi_sn = grup::esik_duzelt(e);
        }
    }

    /// Grubun sekmeleri, çubuktaki sıralarıyla.
    pub fn grubun_sekmeleri(&self, id: GrupId) -> Vec<SekmeId> {
        self.sekmeler
            .iter()
            .filter(|s| s.grup == Some(id))
            .map(|s| s.id)
            .collect()
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn semasiz_alan_adi_adres() {
        assert!(adres_mi("ornek.com"));
        assert!(adres_mi("www.ornek.com.tr/yol?a=1"));
        assert!(adres_mi("ornek.com:8080/yol"));
    }

    #[test]
    fn semali_adres() {
        assert!(adres_mi("https://ornek.com"));
        assert!(adres_mi("http://ornek.com"));
        assert!(adres_mi("file:///C:/bir.html"));
    }

    #[test]
    fn localhost_ve_ip_adres() {
        assert!(adres_mi("localhost"));
        assert!(adres_mi("localhost:1420"));
        assert!(adres_mi("127.0.0.1:8080/yol"));
    }

    #[test]
    fn bosluklu_ve_tek_kelime_arama() {
        assert!(!adres_mi("rust kitabı"));
        assert!(!adres_mi("muiren"));
        assert!(!adres_mi("1.2"));
        assert!(!adres_mi(""));
    }

    #[test]
    fn soru_isareti_zorla_arama() {
        // `localhost` gibi belirsiz bir girdiyi aratmanın tek yolu.
        assert!(!adres_mi("?localhost"));
        assert!(!adres_mi("?ornek.com"));
    }

    #[test]
    fn tehlikeli_sema_adres_degil() {
        // Adres sayılmıyor, yani aramaya düşüyor: sayfada kod çalışmıyor.
        assert!(!adres_mi("javascript:alert(1)"));
        assert!(!adres_mi("data:text/html,<script>x</script>"));
    }

    #[test]
    fn sema_oneki_konak_portu_ayiriyor() {
        assert_eq!(sema_oneki("https://ornek.com").as_deref(), Some("https"));
        assert_eq!(
            sema_oneki("javascript:alert(1)").as_deref(),
            Some("javascript")
        );
        assert_eq!(sema_oneki("localhost:3000"), None);
        assert_eq!(sema_oneki("localhost:3000/yol"), None);
        assert_eq!(sema_oneki("ornek.com"), None);
    }

    #[test]
    fn baslik_kirpiliyor() {
        let uzun = "a".repeat(500);
        assert_eq!(baslik_temizle(&uzun).chars().count(), BASLIK_SINIRI);
    }

    #[test]
    fn baslik_satir_sonlarini_atiyor() {
        assert_eq!(baslik_temizle("bir\nİki\tuc"), "birİkiuc");
    }

    #[test]
    fn baslik_ters_akis_karakterini_atiyor() {
        // U+202E (RIGHT-TO-LEFT OVERRIDE) sekme çubuğunda metni ters akıtıp
        // sahte bir alan adı gösterebiliyor. Ana saldırı bu.
        let ham = "banka.com\u{202E}moc.nagladlas";
        assert!(!baslik_temizle(ham).contains('\u{202E}'));
    }

    #[test]
    fn baslik_sifir_genislikli_karakteri_atiyor() {
        assert_eq!(baslik_temizle("go\u{200B}ogle"), "google");
    }

    #[test]
    fn baslik_bosluklari_kirpiyor() {
        assert_eq!(baslik_temizle("   Muiren   "), "Muiren");
    }

    #[test]
    fn turkce_karakterler_korunuyor() {
        assert_eq!(baslik_temizle("Şişli Ğ İçerik ı"), "Şişli Ğ İçerik ı");
    }

    #[test]
    fn id_yeniden_kullanilmiyor() {
        let mut d = Depo::yeni();
        let a = d.ac("https://bir.example".into(), None, false);
        d.kapat(a);
        let b = d.ac("https://iki.example".into(), None, false);
        assert_ne!(a, b, "kapanan sekmenin kimliği yeniden dağıtıldı");
    }

    #[test]
    fn cocuk_ebeveynin_yanina_aciliyor() {
        let mut d = Depo::yeni();
        let a = d.ac("https://a.example".into(), None, false);
        let _b = d.ac("https://b.example".into(), None, false);
        let c = d.ac("https://c.example".into(), Some(a), false);
        assert_eq!(d.hepsi()[1].id, c);
    }

    #[test]
    fn etkinlestirme_eskiyi_arkaplana_dusuruyor() {
        let mut d = Depo::yeni();
        let a = d.ac("https://a.example".into(), None, false);
        let b = d.ac("https://b.example".into(), None, false);
        d.etkinlestir(a);
        let eski = d.etkinlestir(b);
        assert_eq!(eski, Some(a));
        assert_eq!(d.bul(a).unwrap().durum, Durum::Arkaplan);
        assert_eq!(d.bul(b).unwrap().durum, Durum::Etkin);
    }

    #[test]
    fn kapatilan_sekme_geri_alinabiliyor() {
        let mut d = Depo::yeni();
        let a = d.ac("https://a.example".into(), None, false);
        d.kapat(a);
        assert_eq!(d.geri_al().map(|s| s.url), Some("https://a.example".into()));
    }

    #[test]
    fn yeni_sekme_sayfasi_geri_alma_listesine_girmiyor() {
        let mut d = Depo::yeni();
        let a = d.ac(YENI_SEKME.into(), None, false);
        d.kapat(a);
        assert!(d.geri_al().is_none());
    }

    #[test]
    fn sabitlenen_sekme_basa_gidiyor() {
        let mut d = Depo::yeni();
        let _a = d.ac("https://a.example".into(), None, false);
        let _b = d.ac("https://b.example".into(), None, false);
        let c = d.ac("https://c.example".into(), None, false);
        d.sabitle(c, true);
        assert_eq!(d.hepsi()[0].id, c);
    }

    #[test]
    fn derinlik_hesaplaniyor() {
        let mut d = Depo::yeni();
        let a = d.ac("https://a.example".into(), None, false);
        let b = d.ac("https://b.example".into(), Some(a), false);
        let c = d.ac("https://c.example".into(), Some(b), false);
        let ozetler = d.ozetler();
        let bul = |id| ozetler.iter().find(|o| o.id == id).unwrap().derinlik;
        assert_eq!((bul(a), bul(b), bul(c)), (0, 1, 2));
    }

    #[test]
    fn gorunen_ad_baslik_yoksa_alan_adi() {
        let mut d = Depo::yeni();
        let a = d.ac("https://ornek.com/bir/iki?x=1".into(), None, false);
        assert_eq!(d.bul(a).unwrap().gorunen_ad(), "ornek.com");
    }
}
