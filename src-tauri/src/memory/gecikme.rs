//! Uyanma gecikmesi defteri — **saf** (`docs/Bellek.md`, "Başarı kriterleri").
//!
//! Kabul tablosunun iki satırı uzun süre ölçülemiyordu:
//!
//! ```text
//! Uyuyan sekmeye tıklayıp etkileşime hazır olma   < 300 ms
//! Atılmış sekmeye tıklayıp ilk boya               < 2 sn
//! ```
//!
//! Gecikmeyi sayan bir yer yoktu, dolayısıyla Faz 2 bu iki satır yüzünden
//! kapanamıyordu — ve `docs/Roadmap.md` "hızlı olduğunu hissediyorum"u kabul
//! etmiyor.
//!
//! Bu dosya `esik.rs` ile aynı ayrımda duruyor: **karar ve hesap burada, saf;
//! saat dışarıda.** Kronometreyi başlatan/durduran anlar `Instant` olarak
//! *parametre* geliyor, bu dosya hiç `Instant::now()` çağırmıyor. Bedeli bir
//! satır fazla yazmak, kazancı ölçüm mantığının `--no-default-features`
//! derlemesinde de test edilebilmesi.
//!
//! Sayılar ortalama olarak değil **dağılım** olarak veriliyor. Ortalama kasten
//! yok: tek bir 8 saniyelik uyanma otuz iyi örneğin ortalamasını bozup tabloyu
//! olduğundan kötü gösterir, medyan + p95 ise hem tipik hâli hem kuyruğu ayrı
//! ayrı söylüyor.

use std::collections::HashMap;
use std::time::Instant;

use serde::Serialize;

use crate::tabs::SekmeId;

/// Kaç örnek saklanıyor. Halka tampon: en yenisi girerken en eskisi düşüyor.
///
/// 64 seçildi çünkü hem bir ölçüm oturumuna (50 sekme) yetiyor hem de defter
/// birkaç KB'ta kalıyor — bellek iddiası olan bir programda ölçüm aracının
/// kendisi bir bellek kalemi olmamalı.
pub const KAPASITE: usize = 64;

/// Bu süreyi aşan kronometre **kaydedilmeden atılıyor**.
///
/// Sebebi somut: ağı kopan bir kullanıcının tek bir 5 dakikalık örneği
/// medyanı kalıcı olarak zehirler ve panel bir daha hiç doğru bir sayı
/// göstermezdi. Ölçülemeyen bir uyanmayı hiç saymamak, yanlış saymaktan iyi.
pub const AZAMI_MS: u32 = 30_000;

/// `docs/Bellek.md` kabul tablosundaki hedefler.
///
/// **Bunlar politika eşiği değil** — CLAUDE.md #6 (bellek eşiği koda gömülmez)
/// buraya işlemiyor: hiçbir karara girmiyorlar, uyutma/atma kararını
/// değiştirmiyorlar, yalnız panelde ölçülen sayının yanında "hedef buydu" diye
/// görünüyorlar. Makineye göre değişen bir şey de değiller; dokümanda yazan
/// sabit iddianın kendisi.
pub const HEDEF_UYUYAN_MS: u32 = 300;
pub const HEDEF_ATILMIS_MS: u32 = 2_000;

/// Sekmenin uyanmadan önce hangi durumda olduğu.
///
/// İkisi **farklı şeyler ölçüyor** ve karıştırılmaları tabloyu anlamsız
/// kılardı: biri render sürecinin uyanması, diğeri sayfanın ağdan yeniden
/// yüklenmesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kaynak {
    /// `Uyuyan` → `Etkin`. Kronometre `Resume` çağrısından **önce** başlıyor,
    /// sayfanın JS ana iş parçacığı bizim betiğimizi çalıştırınca duruyor.
    /// Ölçülen şey gerçekten "etkileşime hazır": render süreci uyandı ve
    /// kullanıcının tuşunu işleyecek iş parçacığı boşta.
    Uyuyan,
    /// `Atilmis` → `Etkin`. Kronometre webview yaratılmadan önce başlıyor,
    /// `NavigationCompleted` ile duruyor — yani **ilk boya değil, ondan
    /// sonrası**. Gerçek ilk boyanın üst sınırı: bu sayı eşiği geçiyorsa
    /// kriter kesin olarak sağlanıyor.
    Atilmis,
}

/// Tek bir kaynağın dağılımı. `n == 0` olan dağılım hiç üretilmiyor
/// ([`Defter::ozet`] `None` döndürüyor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dagilim {
    pub n: u32,
    pub medyan_ms: u32,
    pub p95_ms: u32,
    pub en_kotu_ms: u32,
}

/// Panelin ve ölçüm raporunun beslendiği özet (`docs/IPC.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GecikmeOzeti {
    /// `None` = henüz ölçülmedi. **Sıfır göstermek yok**: hiç sekme
    /// uyandırmamış kullanıcıya "medyan 0 ms" demek, ölçülmemiş bir şeyi
    /// ölçülmüş gibi göstermek olurdu.
    pub uyuyan: Option<Dagilim>,
    pub atilmis: Option<Dagilim>,
    pub hedef_uyuyan_ms: u32,
    pub hedef_atilmis_ms: u32,
}

/// Açık kronometreler + kapanmış örnekler.
///
/// Tek yapı, çünkü ikisi arasındaki geçiş kuralları (eşleşmeyen kaynak
/// sayılmaz, geç gelen cevap atılır) tam olarak bu dosyanın işi. Sürücüye
/// dağılmış olsalardı biri unutulduğunda ölçüm sessizce yanlışlanırdı.
#[derive(Debug, Default)]
pub struct Defter {
    bekleyen: HashMap<SekmeId, (Kaynak, Instant)>,
    uyuyan: Vec<u32>,
    atilmis: Vec<u32>,
}

impl Defter {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Kronometreyi başlatır.
    ///
    /// Aynı sekme için açık bir kronometre varsa **eziliyor**: kullanıcı
    /// yükleme bitmeden başka sekmeye geçip geri dönerse eskisi artık
    /// ölçülebilir bir şeye karşılık gelmiyor.
    pub fn basla(&mut self, id: SekmeId, kaynak: Kaynak, simdi: Instant) {
        self.bekleyen.insert(id, (kaynak, simdi));
    }

    /// Kronometreyi durdurur ve örneği deftere yazar.
    ///
    /// Örnek yazılmadan dönebileceği **üç** hâl var ve üçü de kasıtlı: açık
    /// kronometre yok, kaynak eşleşmiyor (uyuyan sekme beklenirken gezinme
    /// olayı geldi), süre [`AZAMI_MS`] üstünde.
    ///
    /// Dönüş kaydedilen örnek — çağıran tarafın günlüğe yazabilmesi için.
    pub fn bitir(&mut self, id: SekmeId, kaynak: Kaynak, simdi: Instant) -> Option<u32> {
        let (bekleyen_kaynak, baslangic) = *self.bekleyen.get(&id)?;
        // Eşleşmeyen kaynak kronometreyi **kapatmıyor**: asıl cevap hâlâ
        // gelebilir ve onu düşürmek ölçümü sessizce eksiltirdi.
        if bekleyen_kaynak != kaynak {
            return None;
        }
        self.bekleyen.remove(&id);

        let ms = simdi.saturating_duration_since(baslangic).as_millis();
        let ms = u32::try_from(ms).unwrap_or(u32::MAX);
        if ms > AZAMI_MS {
            return None;
        }

        let liste = match kaynak {
            Kaynak::Uyuyan => &mut self.uyuyan,
            Kaynak::Atilmis => &mut self.atilmis,
        };
        if liste.len() == KAPASITE {
            liste.remove(0);
        }
        liste.push(ms);
        Some(ms)
    }

    /// Açık kronometreyi örnek üretmeden kapatır.
    ///
    /// İki yerden çağrılıyor: başarısız gezinme (CLAUDE.md #7 — başarısız
    /// gezinme hiçbir yere yazılmıyor, gecikme tablosu da bir yer) ve sekme
    /// kapanması. Çağrılmasaydı kronometre haritada kalır, aynı kimlik
    /// yeniden görüldüğünde saçma bir örnek üretirdi.
    pub fn iptal(&mut self, id: SekmeId) {
        self.bekleyen.remove(&id);
    }

    /// Ölçüm oturumu başlatmak için defteri boşaltır (`gecikme_sifirla`).
    pub fn temizle(&mut self) {
        self.bekleyen.clear();
        self.uyuyan.clear();
        self.atilmis.clear();
    }

    pub fn ozet(&self) -> GecikmeOzeti {
        GecikmeOzeti {
            uyuyan: dagilim(&self.uyuyan),
            atilmis: dagilim(&self.atilmis),
            hedef_uyuyan_ms: HEDEF_UYUYAN_MS,
            hedef_atilmis_ms: HEDEF_ATILMIS_MS,
        }
    }
}

/// Örneklerden dağılım çıkarır. Boş liste `None`.
///
/// Yüzdelik **en yakın sıra** yöntemiyle: sıralanmış listede `ceil(p × n)`.
/// İnterpolasyon yok — 12 örnekte iki komşu değerin ortasını üretmek, elde
/// olmayan bir ölçümü varmış gibi göstermek olurdu.
fn dagilim(ornekler: &[u32]) -> Option<Dagilim> {
    if ornekler.is_empty() {
        return None;
    }
    let mut sirali = ornekler.to_vec();
    sirali.sort_unstable();
    Some(Dagilim {
        n: sirali.len() as u32,
        medyan_ms: yuzdelik(&sirali, 50),
        p95_ms: yuzdelik(&sirali, 95),
        // Kuyruğun kendisi: p95 bile gizleyebiliyor ve kullanıcının "bazen
        // takılıyor" dediği an tam olarak burası.
        en_kotu_ms: *sirali.last().unwrap(),
    })
}

/// `sirali` artan sıralı ve boş değil.
fn yuzdelik(sirali: &[u32], yuzde: u32) -> u32 {
    let n = sirali.len();
    // ceil(yuzde × n / 100), en az 1 — tek örnekte sıfırıncı sıraya düşüp
    // "p95 = 0 ms" demek ölçülmemiş bir iddia olurdu.
    let sira = (yuzde as usize * n).div_ceil(100);
    sirali[sira.clamp(1, n) - 1]
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::time::Duration;

    fn ms(t: Instant, n: u64) -> Instant {
        t + Duration::from_millis(n)
    }

    #[test]
    fn ornek_yokken_dagilim_yok() {
        let o = Defter::yeni().ozet();
        assert!(o.uyuyan.is_none());
        assert!(o.atilmis.is_none());
        // Hedefler örnek olmasa da geliyor: panel "hedef 300 ms" yazabilsin.
        assert_eq!(o.hedef_uyuyan_ms, HEDEF_UYUYAN_MS);
        assert_eq!(o.hedef_atilmis_ms, HEDEF_ATILMIS_MS);
    }

    #[test]
    fn basit_olcum_kaydediliyor() {
        let mut d = Defter::yeni();
        let t = Instant::now();
        d.basla(1, Kaynak::Uyuyan, t);
        assert_eq!(d.bitir(1, Kaynak::Uyuyan, ms(t, 120)), Some(120));
        let o = d.ozet().uyuyan.unwrap();
        assert_eq!((o.n, o.medyan_ms, o.en_kotu_ms), (1, 120, 120));
    }

    #[test]
    fn iki_kaynak_ayri_defterde() {
        let mut d = Defter::yeni();
        let t = Instant::now();
        d.basla(1, Kaynak::Uyuyan, t);
        d.bitir(1, Kaynak::Uyuyan, ms(t, 100));
        d.basla(2, Kaynak::Atilmis, t);
        d.bitir(2, Kaynak::Atilmis, ms(t, 900));
        assert_eq!(d.ozet().uyuyan.unwrap().medyan_ms, 100);
        assert_eq!(d.ozet().atilmis.unwrap().medyan_ms, 900);
    }

    #[test]
    fn kaynak_eslesmezse_ornek_uretilmiyor() {
        // Uyuyan sekme uyandırıldı ama arada bir gezinme olayı geldi: o olay
        // bu kronometreyi durdurmamalı, yoksa "uyanma" diye sayfa yüklemesi
        // ölçülürdü.
        let mut d = Defter::yeni();
        let t = Instant::now();
        d.basla(1, Kaynak::Uyuyan, t);
        assert_eq!(d.bitir(1, Kaynak::Atilmis, ms(t, 50)), None);
        assert!(d.ozet().atilmis.is_none());
        // Kronometre AÇIK kalıyor: asıl cevap hâlâ gelebilir.
        assert_eq!(d.bitir(1, Kaynak::Uyuyan, ms(t, 80)), Some(80));
    }

    #[test]
    fn baslamamis_kronometre_bitirilemiyor() {
        let mut d = Defter::yeni();
        assert_eq!(d.bitir(7, Kaynak::Uyuyan, Instant::now()), None);
    }

    #[test]
    fn iptal_edilen_kronometre_ornek_uretmiyor() {
        // CLAUDE.md #7: başarısız gezinme hiçbir yere yazılmıyor.
        let mut d = Defter::yeni();
        let t = Instant::now();
        d.basla(1, Kaynak::Atilmis, t);
        d.iptal(1);
        assert_eq!(d.bitir(1, Kaynak::Atilmis, ms(t, 300)), None);
        assert!(d.ozet().atilmis.is_none());
    }

    #[test]
    fn cok_uzun_olcum_atiliyor() {
        let mut d = Defter::yeni();
        let t = Instant::now();
        d.basla(1, Kaynak::Atilmis, t);
        assert_eq!(d.bitir(1, Kaynak::Atilmis, ms(t, AZAMI_MS as u64 + 1)), None);
        assert!(d.ozet().atilmis.is_none());
        // Tam sınır kabul ediliyor.
        d.basla(2, Kaynak::Atilmis, t);
        assert!(d.bitir(2, Kaynak::Atilmis, ms(t, AZAMI_MS as u64)).is_some());
    }

    #[test]
    fn yeniden_baslamak_eskisini_eziyor() {
        // Kullanıcı yükleme bitmeden başka sekmeye geçip geri döndü.
        let mut d = Defter::yeni();
        let t = Instant::now();
        d.basla(1, Kaynak::Atilmis, t);
        d.basla(1, Kaynak::Atilmis, ms(t, 500));
        assert_eq!(d.bitir(1, Kaynak::Atilmis, ms(t, 700)), Some(200));
    }

    #[test]
    fn halka_tampon_en_eskiyi_dusuruyor() {
        let mut d = Defter::yeni();
        let t = Instant::now();
        // KAPASITE + 1 örnek: ilki (çok yavaş olan) düşmeli.
        d.basla(0, Kaynak::Uyuyan, t);
        d.bitir(0, Kaynak::Uyuyan, ms(t, 9_000));
        for i in 1..=KAPASITE as u64 {
            d.basla(i, Kaynak::Uyuyan, t);
            d.bitir(i, Kaynak::Uyuyan, ms(t, 100));
        }
        let o = d.ozet().uyuyan.unwrap();
        assert_eq!(o.n, KAPASITE as u32);
        assert_eq!(o.en_kotu_ms, 100, "9 saniyelik ilk örnek düşmeliydi");
    }

    #[test]
    fn medyan_ve_p95_bilinen_kumede_dogru() {
        let mut d = Defter::yeni();
        let t = Instant::now();
        // 1..=20 ms. Medyan (en yakın sıra, %50 → 10. eleman) = 10,
        // p95 (19. eleman) = 19, en kötü = 20.
        for i in 1..=20u64 {
            d.basla(i, Kaynak::Uyuyan, t);
            d.bitir(i, Kaynak::Uyuyan, ms(t, i));
        }
        let o = d.ozet().uyuyan.unwrap();
        assert_eq!((o.n, o.medyan_ms, o.p95_ms, o.en_kotu_ms), (20, 10, 19, 20));
    }

    #[test]
    fn tek_ornekte_p95_o_ornegin_kendisi() {
        let mut d = Defter::yeni();
        let t = Instant::now();
        d.basla(1, Kaynak::Uyuyan, t);
        d.bitir(1, Kaynak::Uyuyan, ms(t, 250));
        let o = d.ozet().uyuyan.unwrap();
        assert_eq!((o.medyan_ms, o.p95_ms), (250, 250));
    }

    #[test]
    fn temizle_bekleyeni_de_siliyor() {
        // Ölçüm oturumu başlarken açık kalmış bir kronometrenin ilk örneği
        // kirletmesi, tam da temizlemekle kaçınılan şey.
        let mut d = Defter::yeni();
        let t = Instant::now();
        d.basla(1, Kaynak::Uyuyan, t);
        d.bitir(1, Kaynak::Uyuyan, ms(t, 10));
        d.basla(2, Kaynak::Atilmis, t);
        d.temizle();
        assert!(d.ozet().uyuyan.is_none());
        assert_eq!(d.bitir(2, Kaynak::Atilmis, ms(t, 50)), None);
    }
}
