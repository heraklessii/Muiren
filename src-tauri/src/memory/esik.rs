//! Eşik kararı — **saf**. Projenin tezi burada.
//!
//! Girdi: sekme özetleri + sistem baskısı + ayarlar. Çıktı: eylem listesi.
//! Sistem çağrısı yok, motor çağrısı yok, zaman okuma yok — boşta kalma süresi
//! `SekmeOzeti.bosta_sn` içinde hazır geliyor. Dolayısıyla bu dosyanın tamamı
//! test edilebilir ve testleri `--no-default-features` derlemesinde de koşuyor.
//!
//! Ayrım önemli: **ne zaman** uyunacağı burada, **nasıl** uyunacağı motorda.
//! Burası bir `TrySuspend` çağrısı görmüyor.
//!
//! ## Koruma kuralları eşiklerden ÖNCE geliyor
//!
//! `docs/Bellek.md` sekiz koruma kuralı sayıyor. Yedincisi (Muiwatch
//! oturumuna bağlı sekme) Faz 5'e ait ve burada **yok** — köprü yazılmadan
//! kontrol edilecek bir alan da yok. Kalan yedisi [`korumali`] içinde.
//!
//! Koruma iki seviyeli, çünkü kural #4 iki seviyeli: doldurulmuş formu olan
//! sekme **atılmaz** ama **uyutulabilir** (uyku sayfanın durumunu koruyor,
//! atma kaybediyor). Bu yüzden [`uyutulabilir`] ve [`atilabilir`] ayrı.

use serde::{Deserialize, Serialize};

use crate::tabs::{Durum, SekmeId, SekmeOzeti};

/// Sistemin bellek darlığı. Kaynağı `memory/olcum.rs`; eşikleri kısaltıyor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BellekBaskisi {
    /// Boş RAM > %40 — eşikler olduğu gibi.
    Dusuk,
    /// %20–40 — eşikler ÷ 2.
    Orta,
    /// %10–20 — eşikler ÷ 4, uyanık üst sınır zorlanıyor.
    Yuksek,
    /// < %10 — en eski uyuyanlar sırayla atılıyor.
    Kritik,
}

impl BellekBaskisi {
    /// Eşiklerin böleni (`docs/Bellek.md` baskı tablosu).
    pub fn bolen(self) -> u64 {
        match self {
            BellekBaskisi::Dusuk => 1,
            BellekBaskisi::Orta => 2,
            BellekBaskisi::Yuksek | BellekBaskisi::Kritik => 4,
        }
    }

    /// Boş fizik bellek yüzdesinden baskı seviyesi.
    ///
    /// Ölçüm `olcum.rs` içinde ama **eşik burada**: seviyeyi Win32 çağrısının
    /// yanında hesaplasaydık motorsuz derlemede test edilemezdi.
    pub fn yuzdeden(bos_yuzde: f64) -> BellekBaskisi {
        if bos_yuzde >= 40.0 {
            BellekBaskisi::Dusuk
        } else if bos_yuzde >= 20.0 {
            BellekBaskisi::Orta
        } else if bos_yuzde >= 10.0 {
            BellekBaskisi::Yuksek
        } else {
            BellekBaskisi::Kritik
        }
    }

    /// Ölçülen baskının, **kullanıcının o anki durumuyla** birlikte hâli.
    ///
    /// İki durum politikayı sertleştiriyor ve ikisi de ayar dosyasında
    /// durmuyor (`docs/Bellek.md`):
    ///
    /// - **Oyun modu** — kullanıcı ya da algılama "şimdi oyundayım" dedi.
    /// - **Pencere simge durumunda** — kullanıcı tarayıcıya bakmıyor. Gözcünün
    ///   çalışması gereken asıl an bu; bakılmayan 14 sekmenin uyanık kalmasının
    ///   karşılığı yok.
    ///
    /// İkisi de **yeni bir eşik getirmiyor**, var olan baskı tablosunu
    /// kullanıyor: en az `Yuksek`, yani eşikler ÷ 4. Ayrı bir "gizli eşiği"
    /// ayarı eklenmedi — ikinci bir sayı, kullanıcının ayarlar ekranında
    /// anlamlandırması gereken ikinci bir kavram demek ve baskı tablosu bu
    /// davranışı zaten tarif ediyor.
    ///
    /// Birleştirme `max` ile: `Kritik` ölçülmüşse pencere gizlendi diye
    /// `Yuksek`e **düşmüyor**. Atama olsaydı, bellek gerçekten tükenmişken
    /// pencereyi küçültmek politikayı gevşetirdi — sessiz ve tam ters yönde
    /// bir hata.
    pub fn etkin(olculen: BellekBaskisi, oyun_modu: bool, pencere_gizli: bool) -> BellekBaskisi {
        if oyun_modu || pencere_gizli {
            olculen.max(BellekBaskisi::Yuksek)
        } else {
            olculen
        }
    }
}

/// Gözcünün eşik kararına taşıdığı ayarlar.
///
/// `Settings`in tamamı değil: burası arama motorunu ya da temayı bilmemeli.
/// Ayrıca oyun modu ve motor yetenekleri gibi **o anki** durum da buraya
/// giriyor — ikisi de kararı değiştiriyor ama ayar dosyasında durmuyor.
#[derive(Debug, Clone, PartialEq)]
pub struct BellekAyarlari {
    pub uyku_esigi_sn: u64,
    pub atma_esigi_sn: u64,
    pub uyanik_ust_sinir: u32,
    /// Bu alan adlarındaki sekmeler korumalı (kural #6).
    pub istisna_alanlari: Vec<String>,
    /// `ICoreWebView2_3` yoksa `Uyuyan` durumu atlanıyor ve doğrudan atmaya
    /// düşülüyor (`docs/Setup.md` yetenek tablosu).
    pub askiya_alma_var: bool,
    /// `ICoreWebView2_19` yoksa `HedefDusur` eylemi hiç üretilmiyor.
    pub bellek_hedefi_var: bool,
    /// Muiwatch oturumuna bağlı sekmeler (koruma kuralı #7).
    ///
    /// Kimlik listesi olarak geliyor, kilit olarak değil: bu dosya saf kalmak
    /// zorunda ve `bridge::muiwatch::Oturumlar` bir `Mutex` tutuyor. Gözcü
    /// turun başında bir kez kopyalıyor.
    pub muiwatch_sekmeleri: Vec<SekmeId>,
}

impl BellekAyarlari {
    /// `Settings`ten okunabilen kısım. Yetenekler ve oyun modu çağıranın işi.
    pub fn ayarlardan(s: &crate::settings::Settings) -> Self {
        BellekAyarlari {
            uyku_esigi_sn: s.uyku_esigi_sn,
            atma_esigi_sn: s.atma_esigi_sn,
            uyanik_ust_sinir: s.uyanik_ust_sinir,
            istisna_alanlari: s.istisna_alanlari.clone(),
            askiya_alma_var: true,
            bellek_hedefi_var: true,
            muiwatch_sekmeleri: Vec::new(),
        }
    }
}

/// Gözcünün motora ileteceği iş.
///
/// Burada **karar** var, uygulama yok: "uyut" deniyor, `TrySuspend`
/// çağrılmıyor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eylem {
    Uyut(SekmeId),
    At(SekmeId),
    /// Uyutmadan önceki ara adım: sayfa çalışmaya devam ediyor ama motor
    /// önbelleklerini boşaltıyor (`ICoreWebView2_19`).
    HedefDusur(SekmeId),
    /// Politika bugün bunu **üretmiyor**: uyanma kararı kullanıcının ve
    /// `sekme_etkinlestir` üzerinden geçiyor. Yüzeyde duruyor çünkü
    /// `docs/Bellek.md` eylem listesinde var ve ileride bir kural (ör. köprü
    /// bir sekmeyi geri isterse) buna ihtiyaç duyabilir.
    Uyandir(SekmeId),
}

/// Sekmenin neden korunduğu. Arayüz bunu ipucunda gösteriyor: kullanıcı
/// sekmesinin neden uyumadığını göremezse özelliğin bozuk olduğunu sanıyor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum KorumaSebebi {
    /// Kural #1 — bakılan sekme.
    Etkin,
    /// Kural #2 — arka planda müzik/podcast birinci sınıf kullanım.
    SesCaliyor,
    /// Kural #3 — kullanıcı "bu hep dursun" demiş.
    Sabit,
    /// Kural #5 — sekme bazlı istisna.
    UyutmaIstisnasi,
    /// Kural #6 — alan adı ayarlardaki listede.
    AlanIstisnasi,
    /// Kural #7 — Muiwatch oturumuna bağlı.
    MuiwatchOturumu,
    /// Kural #8 — tam ekran / PiP.
    TamEkran,
}

/// Sekmenin alan adını kaba biçimde çıkarır.
///
/// Tam bir ayrıştırıcı değil ve olmasına gerek yok: burada tek iş, kullanıcının
/// yazdığı `"posta.sirket.com"` ile sekmenin adresini eşleştirmek. `url` crate'i
/// kullanılmıyor çünkü bu dosya saf kalmalı ve ayrıştırma hatası bir sekmeyi
/// korumasız bırakmamalı — hata durumunda boş dize dönüyor ve eşleşme olmuyor.
fn alan(url: &str) -> &str {
    let kalan = url.split_once("://").map(|(_, k)| k).unwrap_or(url);
    let host = kalan.split(['/', '?', '#']).next().unwrap_or("");
    // Kullanıcı adı ve port at.
    let host = host.rsplit('@').next().unwrap_or(host);
    host.split(':').next().unwrap_or(host)
}

/// Alan adı istisna listesinde mi.
///
/// Alt alan adları da kapsanıyor: `"sirket.com"` yazan kullanıcı
/// `"posta.sirket.com"` sekmesinin de korunmasını bekliyor. Karşılaştırma
/// ASCII küçültmeyle: alan adları zaten ASCII (IDN punycode'a çevrilmiş
/// geliyor, `src/lib/url.ts`).
fn istisna_mi(url: &str, liste: &[String]) -> bool {
    let a = alan(url).to_ascii_lowercase();
    if a.is_empty() {
        return false;
    }
    liste.iter().any(|ham| {
        let d = ham.trim().trim_start_matches('.').to_ascii_lowercase();
        !d.is_empty() && (a == d || a.ends_with(&format!(".{d}")))
    })
}

/// Sekme uyutmaya ve atmaya karşı korumalı mı, korumalıysa neden.
///
/// `docs/Bellek.md` koruma kuralları #1, #2, #3, #5, #6, #8. Kural #4 (form)
/// burada **yok**: o yalnız atmayı engelliyor, [`atilabilir`] içinde.
pub fn korumali(s: &SekmeOzeti, ayarlar: &BellekAyarlari) -> Option<KorumaSebebi> {
    if s.etkin || s.durum == Durum::Etkin {
        return Some(KorumaSebebi::Etkin);
    }
    // Sessize alınan sekme korumasını KAYBEDİYOR (`docs/Sekmeler.md`):
    // kullanıcı sesi kapatmışsa o sekmeyi dinlemiyor demektir.
    if s.ses_caliyor && !s.sessiz {
        return Some(KorumaSebebi::SesCaliyor);
    }
    if s.sabit {
        return Some(KorumaSebebi::Sabit);
    }
    if s.uyutma_istisnasi {
        return Some(KorumaSebebi::UyutmaIstisnasi);
    }
    // Kural #7 — Muiwatch oturumu (`docs/Kopruler.md`). Uyuyan sekme
    // `nav_event` alamıyor ve senkron **sessizce** kopuyor; kullanıcı bunu
    // "Muiwatch bozuk" diye okuyor. Sesten önce değil sonra bakılıyor:
    // sebeplerin sırası arayüzde gösterilen ipucunu belirliyor ve ses çalan
    // bir oturumda "ses çalıyor" daha anlaşılır bir cümle.
    if ayarlar.muiwatch_sekmeleri.contains(&s.id) {
        return Some(KorumaSebebi::MuiwatchOturumu);
    }
    if s.tam_ekran {
        return Some(KorumaSebebi::TamEkran);
    }
    if istisna_mi(&s.url, &ayarlar.istisna_alanlari) {
        return Some(KorumaSebebi::AlanIstisnasi);
    }
    None
}

/// Uyutulabilir mi (koruma kuralları #1, #2, #3, #5, #6, #8).
pub fn uyutulabilir(s: &SekmeOzeti, ayarlar: &BellekAyarlari) -> bool {
    korumali(s, ayarlar).is_none()
}

/// Atılabilir mi. Uyutmanın üstüne kural #4: **doldurulmuş form gider.**
pub fn atilabilir(s: &SekmeOzeti, ayarlar: &BellekAyarlari) -> bool {
    korumali(s, ayarlar).is_none() && !s.form_dolu
}

/// Bu turda yapılacak işler.
///
/// Sıra önemli: önce atma (en çok kazandıran), sonra uyutma, sonra hedef
/// düşürme. Aynı sekme için iki eylem üretilmiyor.
pub fn karar(
    sekmeler: &[SekmeOzeti],
    baski: BellekBaskisi,
    ayarlar: &BellekAyarlari,
) -> Vec<Eylem> {
    let bolen = baski.bolen();
    // `max(1)`: bölen eşiği sıfırlarsa her sekme anında uyurdu — kullanıcı
    // sekme değiştirdiği anda öncekini kaybederdi.
    let uyku_esigi = (ayarlar.uyku_esigi_sn / bolen).max(1);
    let atma_esigi = (ayarlar.atma_esigi_sn / bolen).max(uyku_esigi);

    // Grup bazlı eşik (Faz 5). Grup tablosu buraya **girmiyor**; etkin değer
    // `SekmeOzeti` içinde hazır geliyor (`tabs::Depo::ozetler`) ve bu dosya
    // saf kalıyor.
    //
    // Atma eşiği grup eşiğinden bağımsız DEĞİL: grubuna 1 dakika yazan
    // kullanıcı, o sekmelerin bir saat sonra atılmasını da bekliyor.
    // Ayarlardaki uyku/atma oranı korunarak ölçekleniyor — atma eşiğini
    // olduğu gibi bırakmak, "1 dakikada uyusun" diyen grubun sekmelerini
    // 59 dakika boyunca uyanık-ama-uyuyor durumunda tutardı.
    // Katlanmış grup eşiği **yarıya** indiriyor (`docs/Sekmeler.md`):
    //
    // > Kullanıcı grubu katlayarak "şu an bunlarla işim yok" demiş oluyor.
    // > Bu, bellek politikasının kullanıcı niyetinden sinyal aldığı tek yer.
    //
    // Grubun kendi eşiği varsa katlama onun **üstüne** biniyor, yerine
    // geçmiyor: iki ayrı sinyal ve ikisi de kullanıcıdan geliyor.
    let sekme_esikleri = |s: &SekmeOzeti| -> (u64, u64) {
        let (mut u, mut a) = match s.grup_uyku_esigi_sn {
            None => (uyku_esigi, atma_esigi),
            Some(grup_esigi) => {
                let u = (grup_esigi / bolen).max(1);
                // Atma eşiği ayarlardaki uyku/atma oranıyla ölçekleniyor.
                // Olduğu gibi bırakmak, "1 dakikada uyusun" diyen grubun
                // sekmelerini 59 dakika uyanık-ama-uyuyor tutardı. Sıfıra
                // bölme yok: `duzelt` uyku eşiğini en az 30'a çekiyor.
                let oran =
                    (ayarlar.atma_esigi_sn as f64 / ayarlar.uyku_esigi_sn.max(1) as f64).max(1.0);
                (u, ((u as f64 * oran) as u64).max(u))
            }
        };
        if s.katli {
            u = (u / 2).max(1);
            a = (a / 2).max(u);
        }
        (u, a)
    };

    let mut eylemler = Vec::new();
    let mut islenen: Vec<SekmeId> = Vec::new();

    // --- 1. Atma eşiğini geçenler ---
    for s in sekmeler {
        if s.durum == Durum::Atilmis || !atilabilir(s, ayarlar) {
            continue;
        }
        if s.bosta_sn >= sekme_esikleri(s).1 {
            eylemler.push(Eylem::At(s.id));
            islenen.push(s.id);
        }
    }

    // --- 2. Uyku eşiğini geçenler ---
    //
    // Askıya alma yeteneği yoksa `Uyuyan` atlanıyor ve doğrudan atmaya
    // düşülüyor (`docs/Setup.md`). Yanlış yönde hata yapmak burada doğru yön:
    // atılan sekme URL'den geri geliyor, sessizce çalışmayan bir `TrySuspend`
    // ise sekmenin belleğini hiç bırakmıyor.
    for s in sekmeler {
        if islenen.contains(&s.id) || !s.durum.uyanik() || !uyutulabilir(s, ayarlar) {
            continue;
        }
        if s.bosta_sn >= sekme_esikleri(s).0 {
            if ayarlar.askiya_alma_var {
                eylemler.push(Eylem::Uyut(s.id));
            } else if atilabilir(s, ayarlar) {
                eylemler.push(Eylem::At(s.id));
            } else {
                continue;
            }
            islenen.push(s.id);
        }
    }

    // --- 3. Uyanık üst sınırı ---
    //
    // Eşiği geçmemiş olsalar bile, aynı anda uyanık sekme sayısı sınırı
    // aşıyorsa en uzun süredir dokunulmamışlar uyutuluyor. `Yuksek` ve
    // `Kritik` baskıda bu sınır **zorlanıyor**.
    let mut uyaniklar: Vec<&SekmeOzeti> = sekmeler
        .iter()
        .filter(|s| s.durum.uyanik() && !islenen.contains(&s.id))
        .collect();
    let sinir = ayarlar.uyanik_ust_sinir as usize;
    if uyaniklar.len() > sinir {
        // En eski önce: büyük `bosta_sn` başa.
        uyaniklar.sort_by_key(|s| std::cmp::Reverse(s.bosta_sn));
        let fazla = uyaniklar.len() - sinir;
        for s in uyaniklar
            .into_iter()
            .filter(|s| uyutulabilir(s, ayarlar))
            .take(fazla)
        {
            if ayarlar.askiya_alma_var {
                eylemler.push(Eylem::Uyut(s.id));
            } else if atilabilir(s, ayarlar) {
                eylemler.push(Eylem::At(s.id));
            } else {
                continue;
            }
            islenen.push(s.id);
        }
    }

    // --- 4. Kritik baskı: en eski uyuyanları at ---
    //
    // Uyuyan sekme hâlâ render belleği tutuyor (yığın kısmen boşalıyor ama
    // süreç duruyor). Kritikte tek çare onu tamamen bırakmak.
    if baski == BellekBaskisi::Kritik {
        let mut uyuyanlar: Vec<&SekmeOzeti> = sekmeler
            .iter()
            .filter(|s| s.durum == Durum::Uyuyan && !islenen.contains(&s.id))
            .filter(|s| atilabilir(s, ayarlar))
            .collect();
        uyuyanlar.sort_by_key(|s| std::cmp::Reverse(s.bosta_sn));
        for s in uyuyanlar {
            eylemler.push(Eylem::At(s.id));
            islenen.push(s.id);
        }
    }

    // --- 5. Hedef düşürme ---
    //
    // Uyku eşiğinin yarısını geçmiş ama henüz uyumamış arka plan sekmeleri.
    // Yalnız baskı varken: baskısız bir makinede önbellek boşaltmak sayfayı
    // yavaşlatıyor ve karşılığında kimse bir şey kazanmıyor.
    //
    // Gözcü aynı sekmeye iki kez `HedefDusur` göndermiyor — tekrarı eleyen
    // durum onda (`gozcu.rs`), burada değil: bu dosya saf kalmalı.
    if ayarlar.bellek_hedefi_var && baski >= BellekBaskisi::Orta {
        for s in sekmeler {
            if islenen.contains(&s.id) || s.durum != Durum::Arkaplan {
                continue;
            }
            if s.bosta_sn >= sekme_esikleri(s).0 / 2 {
                eylemler.push(Eylem::HedefDusur(s.id));
            }
        }
    }

    eylemler
}

#[cfg(test)]
mod testler {
    use super::*;

    fn ornek(id: SekmeId, durum: Durum, bosta_sn: u64) -> SekmeOzeti {
        SekmeOzeti {
            id,
            url: format!("https://ornek{id}.com/sayfa"),
            baslik: String::new(),
            gorunen_ad: String::new(),
            favicon: None,
            durum,
            ebeveyn: None,
            derinlik: 0,
            sabit: false,
            uyutma_istisnasi: false,
            ses_caliyor: false,
            sessiz: false,
            form_dolu: false,
            gizli: false,
            grup: None,
            katli: false,
            grup_uyku_esigi_sn: None,
            tam_ekran: false,
            yukleniyor: false,
            geri_var: false,
            ileri_var: false,
            etkin: false,
            bosta_sn,
        }
    }

    fn ayarlar() -> BellekAyarlari {
        BellekAyarlari {
            uyku_esigi_sn: 480,
            atma_esigi_sn: 3600,
            uyanik_ust_sinir: 14,
            istisna_alanlari: Vec::new(),
            askiya_alma_var: true,
            bellek_hedefi_var: true,
            muiwatch_sekmeleri: Vec::new(),
        }
    }

    // ------------------------------------------------------ koruma kuralları

    #[test]
    fn etkin_sekme_korumali() {
        let mut s = ornek(1, Durum::Etkin, 99_999);
        s.etkin = true;
        assert_eq!(korumali(&s, &ayarlar()), Some(KorumaSebebi::Etkin));
        assert!(!uyutulabilir(&s, &ayarlar()));
        assert!(!atilabilir(&s, &ayarlar()));
    }

    #[test]
    fn ses_calan_sekme_korumali() {
        let mut s = ornek(1, Durum::Arkaplan, 99_999);
        s.ses_caliyor = true;
        assert_eq!(korumali(&s, &ayarlar()), Some(KorumaSebebi::SesCaliyor));
    }

    #[test]
    fn sessize_alinan_sekme_korumasini_kaybediyor() {
        // `docs/Sekmeler.md`: kullanıcı sesi kapatmışsa o sekmeyi dinlemiyor.
        let mut s = ornek(1, Durum::Arkaplan, 99_999);
        s.ses_caliyor = true;
        s.sessiz = true;
        assert_eq!(korumali(&s, &ayarlar()), None);
    }

    #[test]
    fn sabit_ve_istisna_korumali() {
        let mut a = ornek(1, Durum::Arkaplan, 99_999);
        a.sabit = true;
        assert_eq!(korumali(&a, &ayarlar()), Some(KorumaSebebi::Sabit));

        let mut b = ornek(2, Durum::Arkaplan, 99_999);
        b.uyutma_istisnasi = true;
        assert_eq!(
            korumali(&b, &ayarlar()),
            Some(KorumaSebebi::UyutmaIstisnasi)
        );
    }

    #[test]
    fn tam_ekran_korumali() {
        let mut s = ornek(1, Durum::Arkaplan, 99_999);
        s.tam_ekran = true;
        assert_eq!(korumali(&s, &ayarlar()), Some(KorumaSebebi::TamEkran));
    }

    #[test]
    fn form_dolu_atilmiyor_ama_uyutulabiliyor() {
        // Kural #4'ün tamamı: uyku sayfanın durumunu koruyor, atma kaybediyor.
        let mut s = ornek(1, Durum::Arkaplan, 99_999);
        s.form_dolu = true;
        assert!(uyutulabilir(&s, &ayarlar()));
        assert!(!atilabilir(&s, &ayarlar()));

        let eylemler = karar(&[s], BellekBaskisi::Dusuk, &ayarlar());
        assert_eq!(eylemler, vec![Eylem::Uyut(1)]);
    }

    // ------------------------------------------------------ alan istisnası

    #[test]
    fn alan_istisnasi_alt_alanlari_da_kapsiyor() {
        let mut a = ayarlar();
        a.istisna_alanlari = vec!["sirket.com".into()];

        let mut s = ornek(1, Durum::Arkaplan, 99_999);
        s.url = "https://posta.sirket.com/gelen".into();
        assert_eq!(korumali(&s, &a), Some(KorumaSebebi::AlanIstisnasi));

        s.url = "https://sirket.com/".into();
        assert_eq!(korumali(&s, &a), Some(KorumaSebebi::AlanIstisnasi));
    }

    #[test]
    fn alan_istisnasi_benzeyen_adresi_kapsamiyor() {
        // `sirket.com` istisnası `sirket.com.saldirgan.net`i korumamalı —
        // adres çubuğundaki alan adı vurgusuyla aynı tuzak.
        let mut a = ayarlar();
        a.istisna_alanlari = vec!["sirket.com".into()];

        let mut s = ornek(1, Durum::Arkaplan, 99_999);
        s.url = "https://sirket.com.saldirgan.net/".into();
        assert_eq!(korumali(&s, &a), None);

        s.url = "https://xsirket.com/".into();
        assert_eq!(korumali(&s, &a), None);
    }

    #[test]
    fn alan_ayristirma_port_ve_kullanici_atiyor() {
        assert_eq!(alan("https://ali@ornek.com:8443/yol?a=1"), "ornek.com");
        assert_eq!(alan("localhost:3000/x"), "localhost");
        assert_eq!(alan("muiren://yeni"), "yeni");
    }

    // ------------------------------------------------------------ eşikler

    #[test]
    fn esigi_gecmeyen_sekmeye_dokunulmuyor() {
        let s = ornek(1, Durum::Arkaplan, 10);
        assert!(karar(&[s], BellekBaskisi::Dusuk, &ayarlar()).is_empty());
    }

    #[test]
    fn uyku_esigini_gecen_uyuyor_atma_esigini_gecen_atiliyor() {
        let a = ayarlar();
        let uyur = ornek(1, Durum::Arkaplan, 500); // > 480
        let atilir = ornek(2, Durum::Uyuyan, 4000); // > 3600
        let eylemler = karar(&[uyur, atilir], BellekBaskisi::Dusuk, &a);
        assert!(eylemler.contains(&Eylem::Uyut(1)));
        assert!(eylemler.contains(&Eylem::At(2)));
    }

    #[test]
    fn ayni_sekmeye_iki_eylem_uretilmiyor() {
        // Atma eşiğini geçen bir sekme uyku eşiğini de geçmiş oluyor; ikisi
        // birden gönderilirse motor önce atıp sonra olmayan webview'i
        // uyutmaya çalışır.
        let s = ornek(1, Durum::Arkaplan, 99_999);
        let eylemler = karar(&[s], BellekBaskisi::Dusuk, &ayarlar());
        assert_eq!(
            eylemler
                .iter()
                .filter(|e| matches!(e, Eylem::At(1) | Eylem::Uyut(1)))
                .count(),
            1
        );
        assert_eq!(eylemler[0], Eylem::At(1));
    }

    #[test]
    fn baski_esikleri_kisaltiyor() {
        let a = ayarlar();
        // 200 sn boşta. Uyku eşiği 480; bölen sırasıyla 1, 2, 4 →
        // 480, 240, 120. Yani yalnız `Yuksek` baskıda uyuyor.
        let s = ornek(1, Durum::Arkaplan, 200);
        let uyudu = |b| karar(std::slice::from_ref(&s), b, &a).contains(&Eylem::Uyut(1));
        assert!(!uyudu(BellekBaskisi::Dusuk));
        assert!(!uyudu(BellekBaskisi::Orta));
        assert!(uyudu(BellekBaskisi::Yuksek));
    }

    #[test]
    fn atma_esigi_uyku_esiginin_altina_inemiyor() {
        // Bölme sonrası atma eşiği uyku eşiğinin altına düşerse sekmeler
        // uyumadan atılırdı; kullanıcı her sekme değişiminde sayfanın
        // yeniden yüklendiğini görürdü (`docs/Depolama.md` aynı tuzak).
        let mut a = ayarlar();
        a.uyku_esigi_sn = 100;
        a.atma_esigi_sn = 120;
        let s = ornek(1, Durum::Arkaplan, 26); // Yuksek'te uyku 25, atma 30
        let eylemler = karar(&[s], BellekBaskisi::Yuksek, &a);
        assert_eq!(eylemler, vec![Eylem::Uyut(1)]);
    }

    // -------------------------------------------------------- uyanık sınırı

    #[test]
    fn uyanik_ust_siniri_en_eskileri_uyutuyor() {
        let mut a = ayarlar();
        a.uyanik_ust_sinir = 2;
        // Üçü de eşiğin altında ama sınır 2.
        let sekmeler = vec![
            ornek(1, Durum::Arkaplan, 5),
            ornek(2, Durum::Arkaplan, 50),
            ornek(3, Durum::Arkaplan, 30),
        ];
        let eylemler = karar(&sekmeler, BellekBaskisi::Dusuk, &a);
        // En eski (2) uyutulmalı, en yeni (1) kalmalı.
        assert_eq!(eylemler, vec![Eylem::Uyut(2)]);
    }

    #[test]
    fn uyanik_siniri_korumaliyi_uyutmuyor() {
        let mut a = ayarlar();
        a.uyanik_ust_sinir = 1;
        let mut sabit = ornek(1, Durum::Arkaplan, 999);
        sabit.sabit = true;
        let sekmeler = vec![sabit, ornek(2, Durum::Arkaplan, 5)];
        let eylemler = karar(&sekmeler, BellekBaskisi::Dusuk, &a);
        // Sabit olan korumalı; sınırı aşan tek aday 2 ama o da en yeni.
        assert!(!eylemler.contains(&Eylem::Uyut(1)));
    }

    // ------------------------------------------------------------- kritik

    #[test]
    fn kritik_baskida_uyuyanlar_atiliyor() {
        let a = ayarlar();
        let sekmeler = vec![ornek(1, Durum::Uyuyan, 10), ornek(2, Durum::Uyuyan, 20)];
        let eylemler = karar(&sekmeler, BellekBaskisi::Kritik, &a);
        // En eski önce.
        assert_eq!(eylemler, vec![Eylem::At(2), Eylem::At(1)]);
    }

    #[test]
    fn kritik_baskida_bile_korumali_atilmiyor() {
        let a = ayarlar();
        let mut sabit = ornek(1, Durum::Uyuyan, 999);
        sabit.sabit = true;
        let eylemler = karar(&[sabit], BellekBaskisi::Kritik, &a);
        assert!(eylemler.is_empty());
    }

    // ------------------------------------------------------------ yetenek

    #[test]
    fn askiya_alma_yoksa_dogrudan_atiliyor() {
        // `docs/Setup.md` yetenek tablosu: `TrySuspend` yoksa `Uyuyan`
        // atlanıyor.
        let mut a = ayarlar();
        a.askiya_alma_var = false;
        let s = ornek(1, Durum::Arkaplan, 500);
        assert_eq!(karar(&[s], BellekBaskisi::Dusuk, &a), vec![Eylem::At(1)]);
    }

    #[test]
    fn askiya_alma_yoksa_formlu_sekme_yine_de_atilmiyor() {
        // Yetenek yokluğu koruma kuralını GEÇERSİZ kılmıyor: form dolu bir
        // sekme atılamıyorsa uyutulamıyor da olsa yerinde kalıyor.
        let mut a = ayarlar();
        a.askiya_alma_var = false;
        let mut s = ornek(1, Durum::Arkaplan, 500);
        s.form_dolu = true;
        assert!(karar(&[s], BellekBaskisi::Dusuk, &a).is_empty());
    }

    #[test]
    fn bellek_hedefi_yoksa_hedef_dusur_uretilmiyor() {
        let mut a = ayarlar();
        a.bellek_hedefi_var = false;
        let s = ornek(1, Durum::Arkaplan, 300); // uyku/2'yi geçiyor
        let eylemler = karar(&[s], BellekBaskisi::Orta, &a);
        assert!(!eylemler.iter().any(|e| matches!(e, Eylem::HedefDusur(_))));
    }

    #[test]
    fn hedef_dusur_yalniz_baski_varken() {
        let a = ayarlar();
        // `Orta` baskıda uyku eşiği 240; 150 sn onun altında ama yarısının
        // (120) üstünde — yani "henüz uyumaz ama önbelleğini boşaltabilir".
        let s = ornek(1, Durum::Arkaplan, 150);
        // Baskısız makinede önbellek boşaltmanın karşılığı yok: sayfa
        // yavaşlıyor, kimse bir şey kazanmıyor.
        assert!(karar(std::slice::from_ref(&s), BellekBaskisi::Dusuk, &a).is_empty());
        assert_eq!(
            karar(&[s], BellekBaskisi::Orta, &a),
            vec![Eylem::HedefDusur(1)]
        );
    }

    #[test]
    fn uyku_esigini_gecen_sekmeye_hedef_dusur_gonderilmiyor() {
        // Uyuyacak sekmeye ayrıca "önbelleğini boşalt" demenin karşılığı yok;
        // motor iki iş birden alırdı ve ikincisi askıya alınmış bir
        // webview'e giderdi.
        let a = ayarlar();
        let s = ornek(1, Durum::Arkaplan, 300); // Orta'da uyku eşiği 240
        let eylemler = karar(&[s], BellekBaskisi::Orta, &a);
        assert_eq!(eylemler, vec![Eylem::Uyut(1)]);
    }

    // -------------------------------------------------------------- baskı

    #[test]
    fn baski_yuzdeden_hesaplaniyor() {
        assert_eq!(BellekBaskisi::yuzdeden(80.0), BellekBaskisi::Dusuk);
        assert_eq!(BellekBaskisi::yuzdeden(40.0), BellekBaskisi::Dusuk);
        assert_eq!(BellekBaskisi::yuzdeden(39.9), BellekBaskisi::Orta);
        assert_eq!(BellekBaskisi::yuzdeden(20.0), BellekBaskisi::Orta);
        assert_eq!(BellekBaskisi::yuzdeden(19.9), BellekBaskisi::Yuksek);
        assert_eq!(BellekBaskisi::yuzdeden(10.0), BellekBaskisi::Yuksek);
        assert_eq!(BellekBaskisi::yuzdeden(9.9), BellekBaskisi::Kritik);
    }

    #[test]
    fn etkin_baski_kullanici_durumu_yoksa_olculeni_veriyor() {
        for b in [
            BellekBaskisi::Dusuk,
            BellekBaskisi::Orta,
            BellekBaskisi::Yuksek,
            BellekBaskisi::Kritik,
        ] {
            assert_eq!(BellekBaskisi::etkin(b, false, false), b);
        }
    }

    #[test]
    fn gizli_pencere_baskiyi_yukselte_cikariyor() {
        // Kullanıcı bakmıyorsa eşikler ÷ 4 (`docs/Bellek.md`, "Gözcü").
        assert_eq!(
            BellekBaskisi::etkin(BellekBaskisi::Dusuk, false, true),
            BellekBaskisi::Yuksek
        );
        assert_eq!(
            BellekBaskisi::etkin(BellekBaskisi::Orta, false, true),
            BellekBaskisi::Yuksek
        );
    }

    #[test]
    fn oyun_modu_baskiyi_yukselte_cikariyor() {
        assert_eq!(
            BellekBaskisi::etkin(BellekBaskisi::Dusuk, true, false),
            BellekBaskisi::Yuksek
        );
    }

    #[test]
    fn kritik_baski_gizli_pencerede_dusmuyor() {
        // Bu testin varlık sebebi: `max` yerine atama yazmak. O hâlde bellek
        // gerçekten tükenmişken pencereyi küçültmek politikayı GEVŞETİRDİ —
        // sessiz ve tam ters yönde bir hata. Aynısı oyun modu için de
        // geçerli.
        assert_eq!(
            BellekBaskisi::etkin(BellekBaskisi::Kritik, false, true),
            BellekBaskisi::Kritik
        );
        assert_eq!(
            BellekBaskisi::etkin(BellekBaskisi::Kritik, true, false),
            BellekBaskisi::Kritik
        );
        assert_eq!(
            BellekBaskisi::etkin(BellekBaskisi::Kritik, true, true),
            BellekBaskisi::Kritik
        );
    }

    #[test]
    fn gizli_pencere_esikleri_dorde_boluyor() {
        // Uçtan uca: 8 dakikalık uyku eşiğine sahip bir profilde 2 dakikadır
        // boşta olan sekme, pencere açıkken uyumuyor; gizliyken uyuyor.
        let a = ayarlar(); // uyku_esigi_sn = 480
        let s = ornek(1, Durum::Arkaplan, 130);

        let acik = BellekBaskisi::etkin(BellekBaskisi::Dusuk, false, false);
        assert!(karar(std::slice::from_ref(&s), acik, &a).is_empty());

        let gizli = BellekBaskisi::etkin(BellekBaskisi::Dusuk, false, true);
        assert_eq!(karar(&[s], gizli, &a), vec![Eylem::Uyut(1)]);
    }

    #[test]
    fn gizli_pencere_korumali_sekmeyi_uyutmuyor() {
        // Simge durumuna almak koruma kurallarını **kaldırmıyor**: arka planda
        // müzik dinlemek birinci sınıf kullanım ve pencereyi küçültmek onu
        // bitirmek demek değil (koruma kuralı #2).
        //
        // `is_empty()` beklenmiyor: korumalı sekme `HedefDusur` alabiliyor ve
        // bu doğru. Hedef düşürmek sayfayı durdurmuyor, motorun önbelleğini
        // boşaltıyor — müzik çalmaya devam ediyor. Koruma **uyutmayı ve
        // atmayı** engelliyor, o kadar.
        let mut s = ornek(1, Durum::Arkaplan, 99_999);
        s.ses_caliyor = true;
        let gizli = BellekBaskisi::etkin(BellekBaskisi::Dusuk, false, true);
        let eylemler = karar(&[s], gizli, &ayarlar());
        assert!(
            !eylemler
                .iter()
                .any(|e| matches!(e, Eylem::Uyut(_) | Eylem::At(_))),
            "ses çalan sekme gizli pencerede de uyutulmamalı: {eylemler:?}"
        );
    }

    #[test]
    fn atilmis_sekme_tekrar_atilmiyor() {
        let s = ornek(1, Durum::Atilmis, 99_999);
        assert!(karar(&[s], BellekBaskisi::Kritik, &ayarlar()).is_empty());
    }

    // ------------------------------------------------- grup bazlı eşik

    #[test]
    fn grup_esigi_genel_esigin_yerine_geciyor() {
        // Genel eşik 480 sn; grup 60 sn demiş. 100 saniyedir boşta olan
        // sekme genel eşikte uyanık kalırdı, grup eşiğinde uyuyor.
        let mut s = ornek(1, Durum::Arkaplan, 100);
        s.grup_uyku_esigi_sn = Some(60);
        assert_eq!(
            karar(&[s], BellekBaskisi::Dusuk, &ayarlar()),
            vec![Eylem::Uyut(1)]
        );
    }

    #[test]
    fn grup_esigi_uzun_da_olabiliyor() {
        // Grup eşiği yalnız kısaltmıyor: "bu gruba dokunma" diyen kullanıcı
        // genel eşikten uzun bir değer yazabiliyor.
        let mut s = ornek(1, Durum::Arkaplan, 600);
        s.grup_uyku_esigi_sn = Some(3600);
        assert!(karar(&[s], BellekBaskisi::Dusuk, &ayarlar()).is_empty());
    }

    #[test]
    fn grup_esigi_atma_esigini_de_olcekliyor() {
        // Ayarlarda oran 3600/480 = 7.5. Grup 60 sn dediğinde atma eşiği
        // 450 sn oluyor. Ölçeklenmeseydi, "1 dakikada uyusun" diyen grubun
        // sekmeleri saatlerce uyur durumda beklerdi.
        let mut s = ornek(1, Durum::Uyuyan, 500);
        s.grup_uyku_esigi_sn = Some(60);
        assert_eq!(
            karar(&[s], BellekBaskisi::Dusuk, &ayarlar()),
            vec![Eylem::At(1)]
        );
    }

    #[test]
    fn grup_esigi_baskiyla_birlikte_bolunuyor() {
        // Baskı bölenleri grup eşiğine de uygulanıyor: `Orta`da 120/2 = 60.
        let mut s = ornek(1, Durum::Arkaplan, 70);
        s.grup_uyku_esigi_sn = Some(120);
        assert_eq!(
            karar(&[s], BellekBaskisi::Orta, &ayarlar()),
            vec![Eylem::Uyut(1)]
        );
    }

    #[test]
    fn grup_esigi_koruma_kuralini_gecmiyor() {
        // Grup eşiği bir hızlandırıcı, koruma kurallarının üstünde değil:
        // ses çalan sekme grubu ne derse desin uyumuyor.
        let mut s = ornek(1, Durum::Arkaplan, 9_999);
        s.grup_uyku_esigi_sn = Some(30);
        s.ses_caliyor = true;
        assert!(karar(&[s], BellekBaskisi::Dusuk, &ayarlar()).is_empty());
    }

    #[test]
    fn katlanmis_grup_esigi_yariya_indiriyor() {
        // `docs/Sekmeler.md`: katlamak "şu an bunlarla işim yok" demek ve
        // bellek politikasının kullanıcı niyetinden sinyal aldığı tek yer.
        // Genel eşik 480; katlıyken 240. 300 saniyedir boşta olan sekme
        // katlanmamışken uyanık kalıyor, katlıyken uyuyor.
        let a = ayarlar();
        let mut duz = ornek(1, Durum::Arkaplan, 300);
        duz.katli = false;
        assert!(karar(&[duz], BellekBaskisi::Dusuk, &a).is_empty());

        let mut katli = ornek(1, Durum::Arkaplan, 300);
        katli.katli = true;
        assert_eq!(
            karar(&[katli], BellekBaskisi::Dusuk, &a),
            vec![Eylem::Uyut(1)]
        );
    }

    #[test]
    fn katlama_grup_esiginin_ustune_biniyor() {
        // İki ayrı sinyal ve ikisi de kullanıcıdan: grup 120 sn demiş,
        // katlama onu 60'a indiriyor. Katlama grubun eşiğinin YERİNE
        // geçseydi, kullanıcının yazdığı sayı katlanınca anlamını yitirirdi.
        let a = ayarlar();
        let mut s = ornek(1, Durum::Arkaplan, 70);
        s.grup_uyku_esigi_sn = Some(120);
        s.katli = true;
        assert_eq!(karar(&[s], BellekBaskisi::Dusuk, &a), vec![Eylem::Uyut(1)]);
    }

    #[test]
    fn katlama_koruma_kuralini_gecmiyor() {
        let a = ayarlar();
        let mut s = ornek(1, Durum::Arkaplan, 9_999);
        s.katli = true;
        s.sabit = true;
        assert!(karar(&[s], BellekBaskisi::Dusuk, &a).is_empty());
    }

    #[test]
    fn gruba_ait_olmayan_sekme_genel_esikte() {
        let a = ayarlar();
        let mut grupsuz = ornek(1, Durum::Arkaplan, 100);
        grupsuz.grup_uyku_esigi_sn = None;
        let mut gruplu = ornek(2, Durum::Arkaplan, 100);
        gruplu.grup_uyku_esigi_sn = Some(60);

        assert_eq!(
            karar(&[grupsuz, gruplu], BellekBaskisi::Dusuk, &a),
            vec![Eylem::Uyut(2)],
            "grupsuz sekme grup eşiğinden etkilendi"
        );
    }

    // ------------------------------------------------- koruma kuralı #7

    #[test]
    fn muiwatch_sekmesi_korumali() {
        let s = ornek(1, Durum::Arkaplan, 99_999);
        let mut a = ayarlar();
        a.muiwatch_sekmeleri = vec![1];
        assert_eq!(korumali(&s, &a), Some(KorumaSebebi::MuiwatchOturumu));
        assert!(!uyutulabilir(&s, &a));
        assert!(!atilabilir(&s, &a));
    }

    #[test]
    fn muiwatch_sekmesi_kritik_baskida_bile_atilmiyor() {
        // Bu kuralın bütün anlamı burada: uyuyan sekme `nav_event` alamıyor
        // ve senkron SESSİZCE kopuyor. Baskı ne olursa olsun, bağlı sekme
        // ayakta kalıyor.
        let s = ornek(1, Durum::Uyuyan, 99_999);
        let mut a = ayarlar();
        a.muiwatch_sekmeleri = vec![1];
        assert!(karar(&[s], BellekBaskisi::Kritik, &a).is_empty());
    }

    #[test]
    fn muiwatch_sekmesi_uyanik_ust_siniri_zorlamiyor() {
        // Üst sınır 1: normalde ikisinden biri uyurdu. Bağlı olan hariç
        // tutuluyor ve fazlalık diğerinden alınıyor.
        let mut a = ayarlar();
        a.uyanik_ust_sinir = 1;
        a.muiwatch_sekmeleri = vec![1];
        let eylemler = karar(
            &[ornek(1, Durum::Arkaplan, 10), ornek(2, Durum::Arkaplan, 20)],
            BellekBaskisi::Dusuk,
            &a,
        );
        assert_eq!(eylemler, vec![Eylem::Uyut(2)]);
    }

    #[test]
    fn bagli_olmayan_sekme_etkilenmiyor() {
        // Liste dolu ama sekme listede değil: koruma yok.
        let s = ornek(2, Durum::Arkaplan, 99_999);
        let mut a = ayarlar();
        a.muiwatch_sekmeleri = vec![1];
        assert_eq!(korumali(&s, &a), None);
    }
}
