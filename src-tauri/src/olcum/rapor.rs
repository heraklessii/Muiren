//! Ölçüm raporu — **saf** (`docs/olcumler/README.md` iskeleti).
//!
//! Girdi ölçülmüş sayılar, çıktı Markdown. Hiçbir sistem çağrısı yok, saat
//! okunmuyor (tarih parametre olarak geliyor) — dolayısıyla iskeletin
//! doğruluğu `--no-default-features` derlemesinde de test ediliyor.
//!
//! **Neden bu dosya bu kadar titiz:** rapor, ölçümün kendisinden daha uzun
//! yaşıyor. Yanlış etiketlenmiş bir satır ("yaklaşık" olduğu yazılmamış bir
//! rakam, hedefi geçmediği hâlde geçmiş gibi görünen bir kriter) aylar sonra
//! kimsenin doğrulayamayacağı bir yanlış olur.

use crate::memory::BellekOzeti;

use super::plan::{Asama, Plan};

/// Bir aşamada alınan anlık görüntü.
#[derive(Debug, Clone)]
pub struct Ornek {
    pub asama: Asama,
    /// Gözcü henüz ilk turunu koşmadıysa `None` — **sıfır yazılmıyor.**
    pub ozet: Option<BellekOzeti>,
    /// Ölçüm, aşamanın iddia ettiği şeyi ölçüyor mu.
    ///
    /// `false` olduğu gerçek bir hâl var ve ilk denemede çıktı: oturumdan
    /// gelen sekmeler varken alınan "boş tarayıcı" ölçümü taban maliyet
    /// **değil**. Rakam yine yazılıyor (silmek bilgi kaybı olurdu) ama
    /// hedefle karşılaştırılmıyor — geçersiz bir ölçümü "hedefi geçmedi"
    /// diye raporlamak, olmayan bir gerilemeyi belgelemek olurdu.
    pub gecerli: bool,
}

/// Raporun başlık tablosuna giren, ölçüm dışı bilgiler.
#[derive(Debug, Clone)]
pub struct Ortam {
    pub tarih: String,
    pub muiren: String,
    pub isletim: String,
    pub profil: String,
    pub surec_politikasi: String,
    /// Tarayıcının bilebildiği hâliyle WebView2 Runtime sürümü; bilinmiyorsa
    /// `None` ve rapor bunu `?` ile işaretliyor (`docs/olcumler/README.md`:
    /// "sürüm alanları boş bırakılmıyor").
    pub webview2: Option<String>,
}

/// Elle doldurulacak alanların işareti.
const ELLE: &str = "? — elle doldurun";

/// Raporu üretir.
///
/// `notlar` koşucunun tuttuğu olaylar (açılamayan adres, kırpılan liste).
/// **Boş bırakılmıyor**: protokol "sorunsuz geçen bir ölçümde bile 'sorunsuz'
/// yazan bir satır" istiyor, çünkü boşluk not almayı unutmuş olmakla aynı
/// görünür.
pub fn olustur(plan: &Plan, ortam: &Ortam, ornekler: &[Ornek], notlar: &[String]) -> String {
    let mut m = String::new();

    m.push_str(&format!("# {} · {}\n\n", ortam.tarih, plan.ad));
    m.push_str(
        "> Bu rapor **ölçüm modu** ile üretildi (`MUIREN_OLCUM`). Muiren\n\
         > tarafındaki sayılar otomatik; Chrome sütunu ve ortam satırlarının\n\
         > bir kısmı elle dolduruluyor (`docs/olcumler/README.md`).\n\n",
    );

    m.push_str("| Alan | Değer |\n|---|---|\n");
    m.push_str(&format!("| Tarih | {} |\n", ortam.tarih));
    m.push_str(&format!("| Makine | {} · işlemci modeli: ? |\n", ortam.isletim));
    m.push_str(&format!(
        "| WebView2 Runtime | {} |\n",
        ortam.webview2.as_deref().unwrap_or(ELLE)
    ));
    m.push_str(&format!("| Muiren | {} · derleme: ? |\n", ortam.muiren));
    m.push_str(&format!("| Bellek profili | {} |\n", ortam.profil));
    m.push_str(&format!("| Süreç politikası | {} |\n", ortam.surec_politikasi));
    m.push_str(&format!("| Ağ | {ELLE} |\n"));
    m.push_str(&format!("| Karşılaştırılan | {ELLE} |\n\n"));

    m.push_str("## Sonuç\n\n");
    m.push_str("| Ölçüt | Hedef | Muiren | Chrome | Geçti mi |\n|---|---|---|---|---|\n");

    let bos = bul(ornekler, Asama::Bos);
    let acildi = bul(ornekler, Asama::Acildi);
    let bosta = bul(ornekler, Asama::Bosta);

    // Boş tarayıcı: **mutlak** bir hedef, dolayısıyla otomatik yargılanabiliyor.
    // Aşağıdaki üç satır Chrome'a göreli ve o sütun elle doluyor; "geçti"
    // yazmak, karşılaştırma yapılmadan verilmiş bir karar olurdu.
    let bos_mb = bos.and_then(|o| o.ozet.as_ref()).map(|o| o.toplam_mb + o.kabuk_mb);
    let bos_gecerli = bos.map(|o| o.gecerli).unwrap_or(false);
    m.push_str(&format!(
        "| Boş tarayıcı | < 150 MB | {}{} | — | {} |\n",
        mb(bos_mb),
        if bos.is_some() && !bos_gecerli {
            " (geçersiz)"
        } else {
            ""
        },
        // Geçersiz ölçüm yargılanmıyor: "hayır" yazmak, olmayan bir gerilemeyi
        // belgelemek olurdu. Sebebi "Notlar" bölümünde duruyor.
        gecti(bos_mb.filter(|_| bos_gecerli).map(|v| v < 150))
    ));
    m.push_str(&format!(
        "| {} sekme, hemen sonra | Chrome'un altında | {} | {ELLE} | {ELLE} |\n",
        plan.adresler.len(),
        mb(acildi.and_then(|o| o.ozet.as_ref()).map(|o| o.toplam_mb + o.kabuk_mb)),
    ));
    m.push_str(&format!(
        "| Render süreç sayısı | Chrome'un yarısı | {} | {ELLE} | {ELLE} |\n",
        sayi(acildi.and_then(|o| o.ozet.as_ref()).map(|o| o.surec_sayisi as u64)),
    ));
    m.push_str(&format!(
        "| {} sekme, {} dk boşta | Chrome'un yarısı | {} | {ELLE} | {ELLE} |\n",
        plan.adresler.len(),
        plan.bosta_sn / 60,
        mb(bosta.and_then(|o| o.ozet.as_ref()).map(|o| o.toplam_mb + o.kabuk_mb)),
    ));

    // Kabul tablosunun son iki satırı (uyanma gecikmeleri) buraya **hiç
    // yazılmıyor**: ölçüm modu bir sekmeye tıklamıyor, dolayısıyla bir uyanma
    // da üretmiyor. Sayısı olmayan bir satırı tabloya koymak, o satır
    // ölçülmüş sanılırdı; "Ham veri" bölümü neden boş olduğunu söylüyor.

    m.push_str("\n## Ham veri\n\n");
    for o in ornekler {
        m.push_str(&format!("### {}\n\n", o.asama.ad()));
        match &o.ozet {
            None => m.push_str("Ölçüm alınamadı: gözcü bu aşamada henüz tur koşmamıştı.\n\n"),
            Some(z) => {
                m.push_str(&format!(
                    "- Sayfalar: {} MB{}\n- Kabuk: {} MB\n- Ortak (tarayıcı, GPU, ağ, kabuk arayüzü): {} MB\n",
                    z.toplam_mb,
                    if z.olcum_yaklasik { " (yaklaşık)" } else { "" },
                    z.kabuk_mb,
                    z.ortak_mb,
                ));
                m.push_str(&format!("- WebView2 süreci: {}\n", z.surec_sayisi));
                m.push_str(&format!(
                    "- Sekmeler: {} etkin · {} arkaplan · {} uyuyan · {} atılmış\n",
                    z.etkin, z.arkaplan, z.uyuyan, z.atilmis
                ));
                m.push_str(&format!(
                    "- Kazanç: {} MB{}\n",
                    z.tahmini_kazanc_mb,
                    if z.olcum_yaklasik {
                        " (yaklaşık — eşleme eksik ya da bir pasif sekme hiç ölçülmemiş)"
                    } else {
                        ""
                    }
                ));
                m.push_str(&format!(
                    "- Sistem: {} MB toplam · {} MB boş\n\n",
                    z.sistem_toplam_mb, z.sistem_bos_mb
                ));
            }
        }
    }

    m.push_str(
        "### Uyanma gecikmeleri\n\n\
         Ölçülmedi. Ölçüm modu hiçbir sekmeye tıklamıyor, dolayısıyla bir\n\
         uyanma da üretmiyor. Kabul tablosunun son iki satırı elle koşuluyor:\n\
         bellek panelinde \"Ölçümü sıfırla\", sonra en az 20 sekmeye tıklama\n\
         (`docs/olcumler/README.md`, 5–6).\n\n",
    );

    m.push_str("## Notlar\n\n");
    if notlar.is_empty() {
        m.push_str("- Sorunsuz: koşu sırasında beklenmeyen bir şey olmadı.\n");
    } else {
        for n in notlar {
            m.push_str(&format!("- {n}\n"));
        }
    }
    m.push_str(
        "- Chrome karşılaştırması bu koşuda **yapılmadı**; yukarıdaki Chrome\n  sütunu elle doldurulacak.\n",
    );

    m.push_str("\n## Sekme listesi\n\n");
    for a in &plan.adresler {
        m.push_str(&format!("- {a}\n"));
    }

    m
}

/// Unix saniyesini `YYYY-AA-GG` biçimine çevirir (UTC).
///
/// Kendi hesabımız, çünkü tek bir tarih dizesi için bir takvim bağımlılığı
/// eklemenin karşılığı yok. Hinnant'ın "civil_from_days" algoritması; saf ve
/// testli — dosya adı bundan üretiliyor ve yanlış tarihli bir rapor,
/// **karşılaştırılamaz** bir rapor demek (`docs/olcumler/README.md`: tarih
/// başta, çünkü aynı ölçüm tekrarlanacak).
///
/// UTC bilinçli: yerel saati okumak işletim sistemine özel bir çağrı
/// gerektiriyor ve o çağrı `--no-default-features` derlemesinde yok. Gece
/// yarısına yakın koşan bir ölçümün tarihi bir gün ileride görünebilir; rapor
/// zaten elden geçiyor.
pub fn tarih(unix_sn: u64) -> String {
    let gun = (unix_sn / 86_400) as i64;
    let z = gun + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let g = doy - (153 * mp + 2) / 5 + 1;
    let a = if mp < 10 { mp + 3 } else { mp - 9 };
    let yil = if a <= 2 { y + 1 } else { y };
    format!("{yil:04}-{a:02}-{g:02}")
}

fn bul(ornekler: &[Ornek], asama: Asama) -> Option<&Ornek> {
    ornekler.iter().find(|o| o.asama == asama)
}

fn mb(deger: Option<u64>) -> String {
    match deger {
        Some(v) => format!("{v} MB"),
        None => "ölçülemedi".into(),
    }
}

fn sayi(deger: Option<u64>) -> String {
    match deger {
        Some(v) => v.to_string(),
        None => "ölçülemedi".into(),
    }
}

/// Otomatik yargılanabilen satırların cevabı.
///
/// `None` = "yargılanmadı". Ölçülmemiş bir kriteri "geçti" saymak, raporu
/// okuyanı yanıltmanın en kolay yolu.
fn gecti(sonuc: Option<bool>) -> &'static str {
    match sonuc {
        Some(true) => "evet",
        Some(false) => "**hayır**",
        None => "ölçülemedi",
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::memory::esik::BellekBaskisi;

    fn ozet(toplam: u64, kabuk: u64, surec: u32, yaklasik: bool) -> BellekOzeti {
        BellekOzeti {
            baski: BellekBaskisi::Dusuk,
            toplam_mb: toplam,
            kabuk_mb: kabuk,
            sistem_toplam_mb: 32768,
            sistem_bos_mb: 20000,
            etkin: 1,
            arkaplan: 0,
            uyuyan: 0,
            atilmis: 0,
            tahmini_kazanc_mb: 0,
            olcum_yaklasik: yaklasik,
            sekme_mb: Vec::new(),
            ortak_mb: 40,
            surec_sayisi: surec,
        }
    }

    fn ortam() -> Ortam {
        Ortam {
            tarih: "2026-09-05".into(),
            muiren: "0.1.0".into(),
            isletim: "Windows · 16 çekirdek · 32 GB".into(),
            profil: "Dengeli".into(),
            surec_politikasi: "process-per-site açık".into(),
            webview2: Some("141.0.3537.71".into()),
        }
    }

    fn plan() -> Plan {
        Plan {
            ad: "faz2".into(),
            adresler: vec!["https://a.example/".into(), "https://b.example/".into()],
            ..Default::default()
        }
    }

    #[test]
    fn iskeletin_butun_bolumleri_var() {
        let m = olustur(&plan(), &ortam(), &[], &[]);
        for bolum in ["## Sonuç", "## Ham veri", "## Notlar", "## Sekme listesi"] {
            assert!(m.contains(bolum), "eksik bölüm: {bolum}");
        }
    }

    #[test]
    fn bos_tarayici_hedefi_otomatik_yargilaniyor() {
        let ornekler = vec![Ornek {
            asama: Asama::Bos,
            ozet: Some(ozet(90, 40, 3, false)),
            gecerli: true,
        }];
        let m = olustur(&plan(), &ortam(), &ornekler, &[]);
        assert!(m.contains("| Boş tarayıcı | < 150 MB | 130 MB | — | evet |"));
    }

    #[test]
    fn hedefi_gecmeyen_satir_kalin_hayir() {
        let ornekler = vec![Ornek {
            asama: Asama::Bos,
            ozet: Some(ozet(180, 40, 3, false)),
            gecerli: true,
        }];
        let m = olustur(&plan(), &ortam(), &ornekler, &[]);
        assert!(m.contains("**hayır**"), "eşiği geçmeyen satır öyle yazılmalı");
    }

    #[test]
    fn chrome_satirlari_otomatik_gecti_demiyor() {
        // Chrome'a göreli üç satır karşılaştırma yapılmadan yargılanamaz.
        let ornekler = vec![Ornek {
            asama: Asama::Acildi,
            ozet: Some(ozet(1200, 40, 8, false)),
            gecerli: true,
        }];
        let m = olustur(&plan(), &ortam(), &ornekler, &[]);
        let satir = m
            .lines()
            .find(|l| l.contains("hemen sonra"))
            .expect("satır yok");
        assert!(satir.contains(ELLE), "Chrome sütunu elle doldurulacak");
        assert!(!satir.contains("| evet |"));
    }

    #[test]
    fn gecersiz_taban_olcumu_yargilanmiyor() {
        // İlk gerçek koşuda çıktı: oturumdan gelen sekmeler varken alınan
        // "boş tarayıcı" ölçümü taban maliyet değil. O rakamı hedefle
        // karşılaştırıp "hayır" yazmak, olmayan bir gerilemeyi belgelemek
        // olurdu.
        let ornekler = vec![Ornek {
            asama: Asama::Bos,
            ozet: Some(ozet(412, 6, 8, true)),
            gecerli: false,
        }];
        let m = olustur(&plan(), &ortam(), &ornekler, &[]);
        assert!(m.contains("418 MB (geçersiz)"), "rakam siliniyor değil, işaretleniyor");
        assert!(!m.contains("**hayır**"));
        assert!(m.contains("| Boş tarayıcı | < 150 MB | 418 MB (geçersiz) | — | ölçülemedi |"));
    }

    #[test]
    fn olculmemis_asama_sifir_yazmiyor() {
        // "Ölçüm alınamadı" ile "0 MB" arasındaki fark, raporun tamamının
        // güvenilirliği.
        let ornekler = vec![Ornek {
            asama: Asama::Bosta,
            ozet: None,
            gecerli: true,
        }];
        let m = olustur(&plan(), &ortam(), &ornekler, &[]);
        assert!(m.contains("Ölçüm alınamadı"));
        assert!(m.contains("| Boş tarayıcı | < 150 MB | ölçülemedi | — | ölçülemedi |"));
    }

    #[test]
    fn yaklasik_olcum_isaretleniyor() {
        let ornekler = vec![Ornek {
            asama: Asama::Acildi,
            ozet: Some(ozet(1200, 40, 8, true)),
            gecerli: true,
        }];
        let m = olustur(&plan(), &ortam(), &ornekler, &[]);
        assert!(m.contains("1200 MB (yaklaşık)"));
    }

    #[test]
    fn notlar_bos_birakilmiyor() {
        // Protokol: "sorunsuz" yazan bir satır, o günün gerçekten sorunsuz
        // olduğunu söylüyor; boşluk not almayı unutmuş olmakla aynı görünür.
        let m = olustur(&plan(), &ortam(), &[], &[]);
        assert!(m.contains("- Sorunsuz:"));
    }

    #[test]
    fn kosucunun_notu_rapora_giriyor() {
        let m = olustur(&plan(), &ortam(), &[], &["3 adres açılamadı".into()]);
        assert!(m.contains("- 3 adres açılamadı"));
        assert!(!m.contains("- Sorunsuz:"));
    }

    #[test]
    fn sekme_listesi_rapora_ekleniyor() {
        // Farklı sitelerle alınan iki sayı karşılaştırılamaz; liste raporun
        // parçası (`docs/olcumler/README.md`).
        let m = olustur(&plan(), &ortam(), &[], &[]);
        assert!(m.contains("- https://a.example/"));
        assert!(m.contains("- https://b.example/"));
    }

    #[test]
    fn bilinmeyen_webview2_surumu_isaretli() {
        let mut o = ortam();
        o.webview2 = None;
        let m = olustur(&plan(), &o, &[], &[]);
        assert!(m.contains(&format!("| WebView2 Runtime | {ELLE} |")));
    }

    #[test]
    fn tarih_bilinen_gunleri_dogru_veriyor() {
        assert_eq!(tarih(0), "1970-01-01");
        // 2026-09-05 00:00:00 UTC.
        assert_eq!(tarih(1_788_566_400), "2026-09-05");
        // Artık gün: 2024-02-29.
        assert_eq!(tarih(1_709_164_800), "2024-02-29");
        // Yıl sonu, gün içindeki saat tarihi kaydırmıyor.
        assert_eq!(tarih(1_735_689_599), "2024-12-31");
    }

    #[test]
    fn gecikme_bolumu_olculmedigini_soyluyor() {
        // Ölçüm modu hiçbir sekmeye tıklamıyor; "0 ms" yazmak uydurma olurdu.
        let m = olustur(&plan(), &ortam(), &[], &[]);
        assert!(m.contains("Uyanma gecikmeleri"));
        assert!(m.contains("Ölçülmedi."));
    }
}
