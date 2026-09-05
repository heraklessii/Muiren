//! Ölçüm planı — **saf** (`docs/olcumler/README.md`).
//!
//! Plan bir dosya: hangi adresler, hangi aralıklarla açılacak ve hangi
//! aşamalarda ölçüm alınacak. Ayrıştırma ve düzeltme burada, koşturma
//! [`super::kosucu`] içinde; ayrım `esik.rs`/`olcum.rs` ayrımının aynısı ve
//! sebebi somut: bir ölçüm aracının kendi hesabı yanlışsa ürettiği rapor
//! yanlış olur ve **bunu kimse fark etmez**, çünkü karşılaştıracak ikinci bir
//! sayı yok.
//!
//! Plan neden bir dosya, komut satırı bayrağı değil: `docs/olcumler/README.md`
//! "aynı liste her ölçümde kullanılıyor ve raporun sonuna ekleniyor" diyor.
//! Elli adresi her koşuda yeniden yazmak, iki ölçümün karşılaştırılabilirliğini
//! ilk günden kaybettirirdi.

use serde::{Deserialize, Serialize};

/// Ölçümün bir aşaması.
///
/// Sıra `docs/olcumler/README.md` içindeki sırayla aynı ve **kasıtlı**:
/// aynı oturumda yukarıdan aşağı koşuluyor, yoksa sayılar birbirini
/// tutmuyor (15 dakika boşta kalmış bir tarayıcının "hemen sonra" ölçümü
/// alınamaz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Asama {
    /// Sıfır sekme, yeni sekme sayfası açık — taban maliyet.
    Bos,
    /// Bütün sekmeler açıldıktan ve yerleştikten sonra.
    Acildi,
    /// Boşta bekleme bittikten sonra: uyutmanın işini yaptığı yer.
    Bosta,
}

impl Asama {
    /// Rapordaki satır adı.
    pub fn ad(self) -> &'static str {
        match self {
            Asama::Bos => "Boş tarayıcı",
            Asama::Acildi => "Sekmeler açıldıktan hemen sonra",
            Asama::Bosta => "Boşta bekleme sonrası",
        }
    }
}

/// Ölçüm planı — dosyadan okunan hâli.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Plan {
    /// Rapor başlığında ve dosya adında görünen kısa ad.
    pub ad: String,
    /// Açılacak adresler. **Boşsa plan geçersiz**: sekme açmayan bir ölçüm
    /// yalnız boş tarayıcı satırını doldurabilirdi ve o zaten elle bakılan
    /// bir sayı.
    pub adresler: Vec<String>,
    /// İki sekme açılışı arasındaki bekleme.
    ///
    /// Sıfır olamaz: elli webview'i aynı anda yaratmak ölçtüğümüz şeyi değil,
    /// açılış darboğazını ölçerdi.
    pub acilis_araligi_ms: u64,
    /// Son sekme açıldıktan sonra beklenen süre (protokol: 60 saniye).
    pub yerlesme_sn: u64,
    /// Boşta bekleme (protokol: 15 dakika).
    pub bosta_sn: u64,
    /// Raporun yazılacağı dosya. Boşsa koşucu `docs/olcumler/` altına
    /// tarih + ad ile kendisi karar veriyor.
    pub cikti: String,
    /// Rapor yazıldıktan sonra Muiren kapansın mı.
    ///
    /// **Varsayılan kapalı.** Açıkken ölçüm betikten koşturulabiliyor
    /// (aç → bekle → raporu oku); kapalıyken tarayıcı ölçüm sonrası ayakta
    /// kalıyor ve panele bakılabiliyor. Varsayılanın kapalı olması bilinçli:
    /// kendini kapatan bir tarayıcı, ayarı yanlış anlayan biri için veri
    /// kaybı riski.
    pub kapat_bitince: bool,
}

impl Default for Plan {
    fn default() -> Self {
        Plan {
            ad: "olcum".into(),
            adresler: Vec::new(),
            acilis_araligi_ms: 1500,
            yerlesme_sn: 60,
            bosta_sn: 15 * 60,
            cikti: String::new(),
            kapat_bitince: false,
        }
    }
}

/// Plandaki değerleri kullanılabilir sınırlara çeker.
///
/// `settings::duzelt` ile aynı kalıp ve aynı gerekçe: geçersiz bir değerin
/// hatası çalışma zamanına kalmasın. Buradaki sınırlar protokolün istediği
/// sürelerin etrafında geniş tutuldu — hızlı bir deneme koşusu (`bosta_sn: 60`)
/// da, protokolün tam hâli de (`900`) geçerli.
pub fn duzelt(ham: Plan) -> Plan {
    Plan {
        ad: {
            let a = dosya_adi_gibi(&ham.ad);
            if a.is_empty() {
                "olcum".into()
            } else {
                a
            }
        },
        // Adres **ayıklanmıyor**, yalnız boşluklar kırpılıyor: hangi adresin
        // açılamadığı raporun "Notlar" bölümüne düşmeli, sessizce listeden
        // düşmemeli (`engel/liste.rs` ile aynı kural).
        adresler: ham
            .adresler
            .into_iter()
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .collect(),
        acilis_araligi_ms: ham.acilis_araligi_ms.clamp(100, 60_000),
        yerlesme_sn: ham.yerlesme_sn.clamp(5, 3_600),
        bosta_sn: ham.bosta_sn.min(24 * 3_600),
        cikti: ham.cikti.trim().to_string(),
        kapat_bitince: ham.kapat_bitince,
    }
}

/// Plan koşulabilir mi.
///
/// Dönüş `Err` ise sebep **kullanıcıya yazılıyor** ve ölçüm hiç başlamıyor:
/// yarım koşan bir ölçüm, koşmayan bir ölçümden kötü — raporu var ama sayısı
/// yok.
pub fn dogrula(plan: &Plan) -> Result<(), String> {
    if plan.adresler.is_empty() {
        return Err("planda hiç adres yok".into());
    }
    if let Some(kotu) = plan
        .adresler
        .iter()
        .find(|a| !a.starts_with("http://") && !a.starts_with("https://"))
    {
        // Arama kutusuna düşen bir dize ölçümü sessizce arama motoruna
        // çevirirdi; ölçüm listesi tam olarak yazıldığı gibi açılmalı.
        return Err(format!("adres http(s) ile başlamıyor: {kotu}"));
    }
    Ok(())
}

/// Ölçümün toplam süresi (saniye) — koşmadan önce yazdırmak için.
///
/// Kullanıcının 20 dakika sürecek bir işi başlattığını **önceden** bilmesi
/// gerekiyor; ölçüm sırasında makineye dokunulmaması gereken bir protokol bu.
pub fn toplam_sn(plan: &Plan) -> u64 {
    let acilis = (plan.adresler.len() as u64) * plan.acilis_araligi_ms / 1000;
    acilis + plan.yerlesme_sn + plan.bosta_sn
}

/// Dosya adına girebilecek hâle indirger: harf, rakam, `-`.
///
/// Türkçe harfler **karşılıklarına** çevriliyor, atılmıyor: "çok-sekme" adı
/// "ok-sekme" olsaydı raporun adı sessizce bozulurdu.
fn dosya_adi_gibi(ham: &str) -> String {
    let mut cikti = String::with_capacity(ham.len());
    for c in ham.trim().chars() {
        let d = match c {
            'ç' | 'Ç' => 'c',
            'ğ' | 'Ğ' => 'g',
            'ı' | 'I' => 'i',
            'İ' | 'i' => 'i',
            'ö' | 'Ö' => 'o',
            'ş' | 'Ş' => 's',
            'ü' | 'Ü' => 'u',
            ' ' | '_' | '.' => '-',
            c if c.is_ascii_alphanumeric() => c.to_ascii_lowercase(),
            '-' => '-',
            _ => continue,
        };
        // İki tire yan yana gelmesin: "faz 2 · 50 sekme" → "faz-2-50-sekme".
        if d == '-' && cikti.ends_with('-') {
            continue;
        }
        cikti.push(d);
    }
    cikti.trim_matches('-').to_string()
}

#[cfg(test)]
mod testler {
    use super::*;

    fn plan() -> Plan {
        Plan {
            adresler: vec!["https://example.com/".into()],
            ..Default::default()
        }
    }

    #[test]
    fn varsayilanlar_protokolle_ayni() {
        // `docs/olcumler/README.md`: 60 saniye yerleşme, 15 dakika boşta.
        let p = Plan::default();
        assert_eq!(p.yerlesme_sn, 60);
        assert_eq!(p.bosta_sn, 900);
    }

    #[test]
    fn adressiz_plan_reddediliyor() {
        assert!(dogrula(&Plan::default()).is_err());
    }

    #[test]
    fn arama_dizesi_adres_sayilmiyor() {
        // Ölçüm listesi tam olarak yazıldığı gibi açılmalı; "example.com"
        // adres çubuğunda arama motoruna da gidebilirdi.
        let p = Plan {
            adresler: vec!["example.com".into()],
            ..Default::default()
        };
        assert!(dogrula(&p).is_err());
    }

    #[test]
    fn gecerli_plan_geciyor() {
        assert!(dogrula(&plan()).is_ok());
    }

    #[test]
    fn acilis_araligi_sifira_inmiyor() {
        // Elli webview'i aynı anda yaratmak açılış darboğazını ölçerdi.
        let p = duzelt(Plan {
            acilis_araligi_ms: 0,
            ..plan()
        });
        assert_eq!(p.acilis_araligi_ms, 100);
    }

    #[test]
    fn bos_adres_satirlari_atiliyor() {
        let p = duzelt(Plan {
            adresler: vec!["  ".into(), " https://a.example/ ".into()],
            ..plan()
        });
        assert_eq!(p.adresler, vec!["https://a.example/"]);
    }

    #[test]
    fn ad_dosya_adina_uygun_hale_geliyor() {
        let p = duzelt(Plan {
            ad: "Faz 2 · 50 SEKME".into(),
            ..plan()
        });
        assert_eq!(p.ad, "faz-2-50-sekme");
    }

    #[test]
    fn turkce_harfler_atilmiyor_cevriliyor() {
        // "çok-sekme" adı "ok-sekme" olsaydı raporun adı sessizce bozulurdu.
        let p = duzelt(Plan {
            ad: "çok şeker İĞNE".into(),
            ..plan()
        });
        assert_eq!(p.ad, "cok-seker-igne");
    }

    #[test]
    fn bos_ad_varsayilana_donuyor() {
        let p = duzelt(Plan {
            ad: "···".into(),
            ..plan()
        });
        assert_eq!(p.ad, "olcum");
    }

    #[test]
    fn toplam_sure_hesaplaniyor() {
        // 10 sekme × 1.5 sn + 60 + 900.
        let p = duzelt(Plan {
            adresler: (0..10).map(|i| format!("https://s{i}.example/")).collect(),
            ..Default::default()
        });
        assert_eq!(toplam_sn(&p), 15 + 60 + 900);
    }

    #[test]
    fn duzelt_kararli() {
        let bir = duzelt(Plan {
            ad: "Faz 2".into(),
            acilis_araligi_ms: 0,
            yerlesme_sn: 1,
            ..plan()
        });
        assert_eq!(duzelt(bir.clone()), bir);
    }

    #[test]
    fn eksik_anahtarli_plan_okunuyor() {
        // `#[serde(default)]`: yalnız adres listesi yazan bir plan geçerli
        // olmalı, gerisi protokolün varsayılanı.
        let p: Plan = serde_json::from_str(r#"{"adresler":["https://a.example/"]}"#).unwrap();
        assert_eq!(p.yerlesme_sn, 60);
        assert_eq!(p.adresler.len(), 1);
    }
}
