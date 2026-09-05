//! Filtre listesi — **saf**, `theme/jeton.rs` ile aynı ruhta.
//!
//! Kural metni kullanıcıdan ya da indirdiği bir dosyadan geliyor; burası onu
//! ayrıştırıp bir eşleştiriciye çeviriyor. Tek bir Win32 ya da motor çağrısı
//! yok, dolayısıyla tamamı test edilebilir ve testleri
//! `--no-default-features` derlemesinde de koşuyor.
//!
//! ## Bilinçli olarak uBlock değiliz
//!
//! Adblock Plus söz dizimi düzinelerce seçenek taşıyor (`$script`,
//! `$third-party`, `$domain=`, öğe gizleme `##`, `#?#` …). Onu tam
//! uygulamak ayrı bir proje ve `docs/Roadmap.md` Faz 6 uzantı tartışmasının
//! konusu. Burada **altyapı** var (`docs/Roadmap.md` Faz 4 aynı kelimeyi
//! kullanıyor: "istek filtreleme altyapısı") ve dört biçim tanınıyor:
//!
//! ```text
//! ! yorum satırı
//! # yorum satırı
//!
//! reklam.example.com          alan adı ve ALT alan adları
//! ||reklam.example.com^       aynı şey — ABP'den kopyala-yapıştır çalışsın
//! /reklamlar/                 yol/sorgu içinde geçen parça
//! @@reklam.example.com        istisna: bu kural her şeyi geçiriyor
//! ```
//!
//! Tanınmayan bir söz dizimi **sessizce atılmıyor**: [`Liste::coz`] atılan
//! satırların sayısını döndürüyor ve ayarlar ekranı "12 kural anlaşılamadı"
//! diyor. Sessiz atmak, kullanıcının koyduğunu sandığı ama çalışmayan bir
//! listeyle dolaşması demek.
//!
//! ## Eşleşme neden alan adı sonekiyle
//!
//! `reklam.example.com` yazan kullanıcı `a.reklam.example.com` adresinin de
//! engellenmesini bekliyor; ama `kotureklam.example.com` engellenmemeli. Düz
//! `contains` ikincisini de yakalardı ve bu, ayarlar ekranına iki satır yazıp
//! yarım internetini kapatan kullanıcı demek. `esik.rs` içindeki
//! `istisna_mi` ile aynı kural, aynı gerekçe.

use std::collections::HashSet;

/// Bir kuralın anlamı.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Kural {
    /// Alan adı ve alt alan adları.
    Alan(String),
    /// Adresin herhangi bir yerinde geçen parça.
    Parca(String),
}

/// Ayrıştırılmış, eşleştirmeye hazır liste.
///
/// Alan kuralları `HashSet` içinde: 50 bin satırlık bir liste için
/// doğrusal arama, her alt kaynak isteğinde 50 bin karşılaştırma demek
/// olurdu. Eşleşme alan adının **eklerinde** dolaşıyor, listede değil —
/// `a.b.c.com` için dört sorgu.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Liste {
    alanlar: HashSet<String>,
    parcalar: Vec<String>,
    /// İstisnalar (`@@`). Ayrı tutuluyorlar çünkü **önce** bakılıyor.
    istisna_alanlari: HashSet<String>,
    istisna_parcalari: Vec<String>,
}

/// Ayrıştırma sonucu.
#[derive(Debug, Clone, PartialEq)]
pub struct Cozum {
    pub liste: Liste,
    /// Anlaşılmayan satırlar. Arayüz bunları gösteriyor — sessizce atmak,
    /// kullanıcının çalışmayan bir listeyle dolaşması demek.
    pub anlasilmayanlar: Vec<String>,
}

impl Liste {
    /// Kural metnini ayrıştırır.
    pub fn coz(kurallar: &[String]) -> Cozum {
        let mut liste = Liste::default();
        let mut anlasilmayanlar = Vec::new();

        for ham in kurallar {
            for satir in ham.lines() {
                let satir = satir.trim();
                if satir.is_empty() || satir.starts_with('!') || satir.starts_with('#') {
                    continue;
                }
                let (istisna, govde) = match satir.strip_prefix("@@") {
                    Some(k) => (true, k.trim()),
                    None => (false, satir),
                };
                match kurali_coz(govde) {
                    Some(Kural::Alan(a)) if istisna => {
                        liste.istisna_alanlari.insert(a);
                    }
                    Some(Kural::Alan(a)) => {
                        liste.alanlar.insert(a);
                    }
                    Some(Kural::Parca(p)) if istisna => liste.istisna_parcalari.push(p),
                    Some(Kural::Parca(p)) => liste.parcalar.push(p),
                    None => anlasilmayanlar.push(satir.to_string()),
                }
            }
        }

        Cozum {
            liste,
            anlasilmayanlar,
        }
    }

    pub fn bos_mu(&self) -> bool {
        self.alanlar.is_empty() && self.parcalar.is_empty()
    }

    pub fn kural_sayisi(&self) -> usize {
        self.alanlar.len() + self.parcalar.len()
    }

    /// Bu adres engelleniyor mu.
    ///
    /// **İstisnalar önce**: `@@` bir kuralı değil bütün listeyi geçiyor.
    /// Sıra tersine olsaydı istisna yazmanın anlamı kalmazdı.
    pub fn engelli_mi(&self, url: &str) -> bool {
        let alan = alan(url);
        if alan_eslesiyor(&self.istisna_alanlari, alan)
            || parca_eslesiyor(&self.istisna_parcalari, url)
        {
            return false;
        }
        alan_eslesiyor(&self.alanlar, alan) || parca_eslesiyor(&self.parcalar, url)
    }
}

/// Tek bir kuralı çözer. Tanınmayan söz dizimi `None`.
fn kurali_coz(govde: &str) -> Option<Kural> {
    // Seçenekli ABP kuralları (`$script`, `$third-party`, `$domain=`) ve öğe
    // gizleme (`##`, `#?#`) uygulanmıyor. **Kabul edilip yok sayılmıyorlar** —
    // anlaşılmayan sayılıyorlar ki kullanıcı çalışmadıklarını görsün.
    //
    // Bu kontrol `||` kalıbından **önce** geliyor ve sırası önemli: sonra
    // gelseydi `||izleyici.com^$third-party` seçenek kısmı atılarak
    // "izleyici.com" kuralına dönerdi. Yani kullanıcının yalnız üçüncü taraf
    // isteklerde geçerli olmasını istediği kural, sitenin tamamını
    // engellerdi — sessizce ve kullanıcı yazdığından çok daha geniş.
    if govde.contains('$') || govde.contains("##") || govde.contains("#?#") {
        return None;
    }

    // ABP alan adı kalıbı: `||alan^` ya da `||alan/`. Kopyala-yapıştır
    // çalışsın diye tanınıyor; bir kullanıcının hazır listeden aldığı satırı
    // elle sadeleştirmesini beklemek gereksiz.
    if let Some(kalan) = govde.strip_prefix("||") {
        let alan = kalan
            .trim_end_matches('^')
            .split(['^', '/', '*'])
            .next()
            .unwrap_or("");
        return gecerli_alan(alan).map(Kural::Alan);
    }

    // Yol/sorgu parçası: en az bir `/` içeren ya da `/` ile başlayan.
    if govde.starts_with('/') {
        let p = govde.trim_matches('/');
        return (!p.is_empty()).then(|| Kural::Parca(p.to_ascii_lowercase()));
    }

    // Düz alan adı.
    gecerli_alan(govde).map(Kural::Alan)
}

/// Alan adı olarak kabul edilebilir mi.
///
/// Dar tutuluyor: bir yazım hatası (`example .com`, `htp://x`) kural
/// olmaktan çok sessiz bir hata kaynağı.
fn gecerli_alan(ham: &str) -> Option<String> {
    let a = ham.trim().trim_start_matches('.').to_ascii_lowercase();
    if a.is_empty() || a.len() > 253 || !a.contains('.') {
        return None;
    }
    let gecerli = a.split('.').all(|etiket| {
        !etiket.is_empty()
            && etiket.len() <= 63
            && etiket
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    });
    gecerli.then_some(a)
}

/// Adresin alan adı. `esik::alan` ile aynı yaklaşım ve aynı gerekçe: tam bir
/// ayrıştırıcı değil, hata durumunda boş dize.
fn alan(url: &str) -> &str {
    let kalan = url.split_once("://").map(|(_, k)| k).unwrap_or(url);
    let host = kalan.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or(host);
    host.split(':').next().unwrap_or(host)
}

/// Alan adı ya da bir üst alan adı listede mi.
///
/// `a.b.example.com` için sırayla `a.b.example.com`, `b.example.com`,
/// `example.com`, `com` sorgulanıyor. `com` da sorgulanıyor çünkü
/// [`gecerli_alan`] nokta içermeyen kuralı zaten reddediyor; listeye `com`
/// giremiyor.
fn alan_eslesiyor(liste: &HashSet<String>, alan: &str) -> bool {
    if liste.is_empty() || alan.is_empty() {
        return false;
    }
    let alan = alan.to_ascii_lowercase();
    let mut kalan = alan.as_str();
    loop {
        if liste.contains(kalan) {
            return true;
        }
        match kalan.split_once('.') {
            Some((_, geri_kalan)) => kalan = geri_kalan,
            None => return false,
        }
    }
}

fn parca_eslesiyor(parcalar: &[String], url: &str) -> bool {
    if parcalar.is_empty() {
        return false;
    }
    let kucuk = url.to_ascii_lowercase();
    parcalar.iter().any(|p| kucuk.contains(p.as_str()))
}

#[cfg(test)]
mod testler {
    use super::*;

    fn liste(kurallar: &[&str]) -> Liste {
        Liste::coz(&[kurallar.join("\n")]).liste
    }

    #[test]
    fn duz_alan_adi_engelleniyor() {
        let l = liste(&["reklam.example.com"]);
        assert!(l.engelli_mi("https://reklam.example.com/afis.js"));
    }

    #[test]
    fn alt_alan_adi_da_engelleniyor() {
        // Kullanıcı "reklam.example.com" yazdığında alt alan adlarının da
        // kapanmasını bekliyor.
        let l = liste(&["reklam.example.com"]);
        assert!(l.engelli_mi("https://a.b.reklam.example.com/x"));
    }

    #[test]
    fn benzer_ama_farkli_alan_engellenmiyor() {
        // Düz `contains` bunu da yakalardı; kullanıcı iki satırla yarım
        // internetini kapatırdı.
        let l = liste(&["reklam.example.com"]);
        assert!(!l.engelli_mi("https://kotureklam.example.com/x"));
        assert!(!l.engelli_mi("https://example.com/reklam"));
    }

    #[test]
    fn abp_alan_kalibi_taniniyor() {
        // Hazır listeden kopyalanan satır elle sadeleştirilmeden çalışmalı.
        let l = liste(&["||izleyici.example.net^"]);
        assert!(l.engelli_mi("https://izleyici.example.net/piksel.gif"));
        assert!(l.engelli_mi("https://cdn.izleyici.example.net/a"));
    }

    #[test]
    fn yol_parcasi_engelleniyor() {
        let l = liste(&["/reklamlar/"]);
        assert!(l.engelli_mi("https://site.example/icerik/reklamlar/1.png"));
        assert!(!l.engelli_mi("https://site.example/icerik/haber/1.png"));
    }

    #[test]
    fn istisna_kurali_engellemeyi_geciyor() {
        let l = liste(&["example.com", "@@izin.example.com"]);
        assert!(l.engelli_mi("https://baska.example.com/x"));
        assert!(
            !l.engelli_mi("https://izin.example.com/x"),
            "istisna kuralı çalışmadı"
        );
    }

    #[test]
    fn yorum_satirlari_atlaniyor() {
        let c = Liste::coz(&["! liste başlığı\n# not\n\nreklam.example.com".into()]);
        assert!(c.anlasilmayanlar.is_empty());
        assert_eq!(c.liste.kural_sayisi(), 1);
    }

    #[test]
    fn secenekli_kurallar_anlasilmayan_sayiliyor() {
        // Kabul edip yok saysaydık kullanıcı kuralın çalıştığını sanırdı.
        let c = Liste::coz(&["||izleyici.com^$third-party\nexample.com##.reklam".into()]);
        assert_eq!(c.anlasilmayanlar.len(), 2, "{:?}", c.anlasilmayanlar);
        assert!(c.liste.bos_mu());
    }

    #[test]
    fn secenek_ayiklanip_kural_genisletilmiyor() {
        // `$third-party` atılıp geriye "izleyici.com" kalsaydı, kullanıcının
        // yalnız üçüncü taraf istekler için yazdığı kural sitenin tamamını
        // engellerdi — sessizce ve yazdığından çok daha geniş.
        let c = Liste::coz(&["||izleyici.com^$third-party".into()]);
        assert!(
            !c.liste.engelli_mi("https://izleyici.com/kendi-sayfasi"),
            "seçenekli kural tam alan kuralına genişledi"
        );
    }

    #[test]
    fn bozuk_alan_adi_anlasilmayan_sayiliyor() {
        let c = Liste::coz(&["example .com\nyalnizca-etiket\nhttp://x y".into()]);
        assert_eq!(c.anlasilmayanlar.len(), 3, "{:?}", c.anlasilmayanlar);
    }

    #[test]
    fn buyuk_harf_ve_bosluk_normalize_ediliyor() {
        let l = liste(&["  Reklam.Example.COM  "]);
        assert!(l.engelli_mi("https://REKLAM.EXAMPLE.COM/x"));
    }

    #[test]
    fn bos_liste_hicbir_seyi_engellemiyor() {
        let l = Liste::default();
        assert!(l.bos_mu());
        assert!(!l.engelli_mi("https://ornek.com/"));
    }

    #[test]
    fn port_ve_kullanici_adi_alan_adini_bozmuyor() {
        let l = liste(&["reklam.example.com"]);
        assert!(l.engelli_mi("https://kullanici@reklam.example.com:8443/x"));
    }

    #[test]
    fn tek_etiketli_kural_listeye_giremiyor() {
        // `com` listeye girseydi alan adı eşleşmesindeki üst-alan taraması
        // interneti kapatırdı.
        let c = Liste::coz(&["com".into()]);
        assert!(c.liste.bos_mu());
        assert_eq!(c.anlasilmayanlar, vec!["com"]);
    }
}
