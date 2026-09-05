//! Oturum — sekmelerin hayatta kalması.
//!
//! `oturum.json`, sekme listesinin diske yazılmış hâli. İçinde webview yok,
//! sadece [`Sekme`] kayıtları — birkaç yüz KB, 200 sekmede bile.
//!
//! **Neden SQLite değil** (`docs/Depolama.md`): oturum saniyede birkaç kez
//! değişebiliyor ve tamamı her seferinde yeniden yazılıyor. Küçük ve bütün
//! hâlinde okunan bir veri için işlem yükü gereksiz; JSON + atomik rename'in
//! çökme sonrası davranışı da öngörülebilir.
//!
//! **Nasıl yazılıyor:** geçici dosyaya yaz → `sync_all` → var olanı `.bak`a
//! kopyala → atomik rename. Yarıda kesilen yazım eski oturumu bozmuyor; çökme
//! sonrası bozuk JSON gelirse yedeğe düşülüyor.
//!
//! **Açılışta:** sabitlenmemiş bütün sekmeler `Atilmis` doğuyor. 200 sekmelik
//! bir oturum saniyeler içinde açılıyor ve sıfır render belleği tüketiyor.
//! "Açılışta hepsini yükle" seçeneği bilinçli olarak **yok** — projenin tezine
//! aykırı (`docs/Sekmeler.md`).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::durum::Durum;
use super::{Depo, Sekme, SekmeId};
use crate::hata::Sonuc;

/// Biçim sürümü. Alan eklendiğinde artmıyor (serde eksik alanı varsayılana
/// çeviriyor); **anlamı** değişen bir alan geldiğinde artacak.
const SURUM: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OturumSekmesi {
    pub id: SekmeId,
    pub url: String,
    #[serde(default)]
    pub baslik: String,
    #[serde(default)]
    pub gecmis: Vec<String>,
    #[serde(default)]
    pub gecmis_konum: usize,
    #[serde(default)]
    pub kaydirma: f32,
    #[serde(default)]
    pub ebeveyn: Option<SekmeId>,
    #[serde(default)]
    pub sabit: bool,
    #[serde(default)]
    pub uyutma_istisnasi: bool,
    /// Sekmenin grubu. `#[serde(default)]` sayesinde eski oturum dosyaları
    /// (grup alanı olmayan) hâlâ okunuyor.
    #[serde(default)]
    pub grup: Option<crate::tabs::GrupId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Oturum {
    #[serde(default)]
    pub surum: u32,
    #[serde(default)]
    pub etkin: Option<SekmeId>,
    #[serde(default)]
    pub sekmeler: Vec<OturumSekmesi>,
    /// Sekme grupları (Faz 5).
    ///
    /// Sekmelerle **aynı dosyada** duruyorlar: grup bir sekme özelliği ve
    /// ayrı bir dosyaya yazılsaydı ikisi arasında yarım kalmış bir yazımda
    /// ayrışabilirlerdi (grubu olan ama grubu olmayan sekmeler).
    #[serde(default)]
    pub gruplar: Vec<crate::tabs::Grup>,
}

impl Default for Oturum {
    fn default() -> Self {
        Oturum {
            surum: SURUM,
            etkin: None,
            sekmeler: Vec::new(),
            gruplar: Vec::new(),
        }
    }
}

fn yedek_yolu(yol: &Path) -> PathBuf {
    yol.with_extension("bak")
}

/// Depoyu serileştirilebilir hâle çevirir.
pub fn topla(depo: &Depo) -> Oturum {
    Oturum {
        surum: SURUM,
        etkin: depo.etkin(),
        sekmeler: depo
            .hepsi()
            .iter()
            // Gizli sekmeler oturum dosyasına **girmiyor**. Girselerdi
            // "gizli" iddiası ilk yeniden başlatmada çökerdi: adres diske
            // yazılmış olurdu ve kullanıcı onu bir daha hiç görmezdi.
            .filter(|s| !s.gizli)
            .map(|s| OturumSekmesi {
                id: s.id,
                url: s.url.clone(),
                // Başlık zaten temizlenmiş hâlde tutuluyor; ham metin oturum
                // dosyasına da girmiyor (CLAUDE.md #8).
                baslik: s.baslik.clone(),
                gecmis: s.gecmis.clone(),
                gecmis_konum: s.gecmis_konum,
                kaydirma: s.kaydirma,
                ebeveyn: s.ebeveyn,
                sabit: s.sabit,
                uyutma_istisnasi: s.uyutma_istisnasi,
                grup: s.grup,
            })
            .collect(),
        gruplar: depo.gruplar().to_vec(),
    }
}

/// Atomik yazım.
pub fn yaz(yol: &Path, oturum: &Oturum) -> Sonuc<()> {
    if let Some(dizin) = yol.parent() {
        fs::create_dir_all(dizin)?;
    }
    let gecici = yol.with_extension("json.gecici");
    let veri = serde_json::to_vec_pretty(oturum)?;

    {
        let mut dosya = fs::File::create(&gecici)?;
        dosya.write_all(&veri)?;
        // `sync_all` olmadan rename, içeriği diske inmemiş bir dosyayı
        // yerine koyabiliyor: elektrik kesilirse boş bir oturum dosyası
        // kalıyor ve bütün sekmeler gidiyor.
        dosya.sync_all()?;
    }

    if yol.exists() {
        // Yedek kopyalanamazsa yazımı iptal etmiyoruz — yedek bir konfor,
        // güncel oturumu yazmak asıl iş.
        let _ = fs::copy(yol, yedek_yolu(yol));
    }
    fs::rename(&gecici, yol)?;
    Ok(())
}

/// **Asla hata döndürmüyor.**
///
/// Bozuk bir oturum dosyası yüzünden tarayıcının açılmaması kabul edilemez.
/// Sıra: dosya → yedek → boş oturum.
pub fn oku(yol: &Path) -> Oturum {
    if let Some(o) = coz(yol) {
        return o;
    }
    if let Some(o) = coz(&yedek_yolu(yol)) {
        return o;
    }
    Oturum::default()
}

fn coz(yol: &Path) -> Option<Oturum> {
    let veri = fs::read(yol).ok()?;
    serde_json::from_slice::<Oturum>(crate::bom_kirp(&veri)).ok()
}

/// Oturumu depoya yükler.
///
/// Sabitlenmemiş her sekme `Atilmis` doğuyor. Sabitlenmişler `Arkaplan`
/// doğuyor — kullanıcı onları kalıcı olsun diye sabitledi, açılışta
/// yükleniyorlar (`docs/Sekmeler.md`).
pub fn yukle(depo: &mut Depo, oturum: Oturum) {
    // Gruplar sekmelerden ÖNCE: sekmenin `grup` alanı var olmayan bir gruba
    // işaret ediyorsa aşağıda temizleniyor ve o kontrol grupların yüklenmiş
    // olmasını gerektiriyor.
    depo.gruplari_yukle(oturum.gruplar);

    for os in oturum.sekmeler {
        // Ağaç bağı kopuk bir kayıt (elle düzenlenmiş dosya) sekmeyi
        // kaybettirmesin: bilinmeyen ebeveyn köke çekiliyor.
        let mut sekme = Sekme {
            id: os.id,
            baslik: super::baslik_temizle(&os.baslik),
            favicon: None,
            gecmis_konum: os.gecmis_konum.min(os.gecmis.len().saturating_sub(1)),
            gecmis: if os.gecmis.is_empty() {
                vec![os.url.clone()]
            } else {
                os.gecmis.clone()
            },
            url: os.url,
            bekleyen: None,
            kaydirma: os.kaydirma.clamp(0.0, 1.0),
            // Oturumdan gelen sekme `Atilmis` doğuyor; kaydırma tıklandığında
            // `sekme_etkinlestir` tarafından bekletiliyor, burada değil.
            kaydirma_bekliyor: None,
            // Oturumdan gelen sekme **hiçbir zaman gizli değil**: gizli
            // sekmeler dosyaya zaten yazılmıyor (`topla`), dolayısıyla
            // buraya bir gizli kayıt düşmesi tek bir şey demek — dosya elle
            // düzenlenmiş. O durumda da normal sekme olarak açılıyor:
            // "gizli" iddiasını diskten okunan bir bayrağa dayandırmak
            // yanlış olurdu.
            gizli: false,
            ebeveyn: os.ebeveyn,
            // Var olmayan bir gruba işaret eden sekme gruptan çıkıyor: dosya
            // elle düzenlenmişse arayüz ile depo, kimsenin göremediği bir
            // grup kimliği üzerinde ayrışırdı.
            grup: os.grup.filter(|g| depo.grup_bul(*g).is_some()),
            sabit: os.sabit,
            uyutma_istisnasi: os.uyutma_istisnasi,
            durum: if os.sabit {
                Durum::Arkaplan
            } else {
                Durum::Atilmis
            },
            son_etkinlik: std::time::Instant::now(),
            ses_caliyor: false,
            sessiz: false,
            form_dolu: false,
            tam_ekran: false,
            yukleniyor: false,
            geri_var: false,
            ileri_var: false,
        };
        if sekme.ebeveyn == Some(sekme.id) {
            sekme.ebeveyn = None;
        }
        depo.ekle_ham(sekme);
    }

    let bilinen: Vec<SekmeId> = depo.hepsi().iter().map(|s| s.id).collect();
    let temizlenecek: Vec<SekmeId> = depo
        .hepsi()
        .iter()
        .filter(|s| s.ebeveyn.is_some_and(|e| !bilinen.contains(&e)))
        .map(|s| s.id)
        .collect();
    for id in temizlenecek {
        if let Some(s) = depo.bul_mut(id) {
            s.ebeveyn = None;
        }
    }

    if let Some(e) = oturum.etkin.filter(|e| bilinen.contains(e)) {
        depo.etkinlestir(e);
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn dolu_depo() -> Depo {
        let mut d = Depo::yeni();
        let a = d.ac("https://a.example/".into(), None, false);
        let b = d.ac("https://b.example/".into(), Some(a), false);
        d.bul_mut(a).unwrap().baslik = "A".into();
        d.sabitle(a, true);
        d.etkinlestir(b);
        d
    }

    #[test]
    fn gruplar_gidis_donus() {
        let mut d = dolu_depo();
        let g = d.grup_ac("Ders");
        d.grup_guncelle(g, None, None, Some(true), Some(Some(90)));
        let sekme = d.hepsi()[1].id;
        d.grup_ata(sekme, Some(g));

        let mut geri = Depo::yeni();
        yukle(&mut geri, topla(&d));

        assert_eq!(geri.gruplar().len(), 1);
        let gg = geri.grup_bul(g).unwrap();
        assert_eq!(gg.ad, "Ders");
        assert!(gg.katli);
        assert_eq!(gg.uyku_esigi_sn, Some(90));
        assert_eq!(geri.bul(sekme).unwrap().grup, Some(g));
    }

    #[test]
    fn olmayan_gruba_isaret_eden_sekme_gruptan_cikiyor() {
        // Dosya elle düzenlenmişse arayüz ile depo, kimsenin göremediği bir
        // grup kimliği üzerinde ayrışırdı.
        let mut d = Depo::yeni();
        yukle(
            &mut d,
            Oturum {
                surum: SURUM,
                etkin: None,
                gruplar: Vec::new(),
                sekmeler: vec![OturumSekmesi {
                    id: 1,
                    url: "https://a.example/".into(),
                    baslik: String::new(),
                    gecmis: Vec::new(),
                    gecmis_konum: 0,
                    kaydirma: 0.0,
                    ebeveyn: None,
                    sabit: false,
                    uyutma_istisnasi: false,
                    grup: Some(42),
                }],
            },
        );
        assert_eq!(d.bul(1).unwrap().grup, None);
    }

    #[test]
    fn grup_kimligi_yeniden_dagitilmiyor() {
        // `ekle_ham` ile aynı gerekçe: geri yüklenen bir kimlik yeniden
        // dağıtılırsa gözcü ile arayüz farklı grupları kastediyor olur.
        let mut d = Depo::yeni();
        d.gruplari_yukle(vec![crate::tabs::Grup::yeni(7, "Eski")]);
        assert!(d.grup_ac("Yeni") > 7);
    }

    #[test]
    fn gizli_sekme_oturuma_yazilmiyor() {
        // "Gizli" iddiasının en kolay çöktüğü yer: adres oturum dosyasına
        // düşerse kullanıcı onu bir daha hiç görmez ama disk görür.
        let mut d = dolu_depo();
        let g = d.ac("https://gizli.example/".into(), None, true);

        let anlik = topla(&d);
        assert_eq!(anlik.sekmeler.len(), 2, "gizli sekme oturuma girdi");
        assert!(!anlik.sekmeler.iter().any(|s| s.id == g));
        assert!(!anlik
            .sekmeler
            .iter()
            .any(|s| s.url.contains("gizli.example")));
    }

    #[test]
    fn elle_eklenen_gizli_kayit_normal_aciliyor() {
        // Dosya elle düzenlenmişse "gizli" bayrağı diskten okunmuyor:
        // gizlilik iddiasını dosyadan gelen bir bayrağa dayandırmak yanlış
        // olurdu.
        let mut d = Depo::yeni();
        yukle(
            &mut d,
            Oturum {
                surum: SURUM,
                etkin: None,
                gruplar: Vec::new(),
                sekmeler: vec![OturumSekmesi {
                    id: 1,
                    url: "https://a.example/".into(),
                    baslik: String::new(),
                    gecmis: Vec::new(),
                    gecmis_konum: 0,
                    kaydirma: 0.0,
                    ebeveyn: None,
                    sabit: false,
                    uyutma_istisnasi: false,
                    grup: None,
                }],
            },
        );
        assert!(!d.bul(1).unwrap().gizli);
    }

    #[test]
    fn yaz_oku_gidis_donus() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("oturum.json");
        let depo = dolu_depo();

        yaz(&yol, &topla(&depo)).unwrap();
        let geri = oku(&yol);

        assert_eq!(geri.sekmeler.len(), 2);
        assert_eq!(geri.etkin, depo.etkin());
    }

    #[test]
    fn acilista_sabitlenmemis_sekmeler_atilmis_doguyor() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("oturum.json");
        yaz(&yol, &topla(&dolu_depo())).unwrap();

        let mut yeni = Depo::yeni();
        yukle(&mut yeni, oku(&yol));

        let sabit = yeni.hepsi().iter().find(|s| s.sabit).unwrap();
        let normal = yeni.hepsi().iter().find(|s| !s.sabit).unwrap();
        assert_eq!(sabit.durum, Durum::Arkaplan, "sabitlenmiş sekme yüklenmeli");
        // Etkin sekme geri yüklenirken uyanıyor; onun dışındakiler atılmış.
        assert!(matches!(normal.durum, Durum::Atilmis | Durum::Etkin));
    }

    #[test]
    fn bozuk_dosya_yedege_dusuyor() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("oturum.json");

        yaz(&yol, &topla(&dolu_depo())).unwrap();
        // İkinci yazım birinciyi .bak'a kopyalıyor.
        yaz(&yol, &topla(&dolu_depo())).unwrap();
        fs::write(&yol, b"{ yarim kalmis").unwrap();

        assert_eq!(oku(&yol).sekmeler.len(), 2, "yedek okunmadı");
    }

    #[test]
    fn bom_ile_yazilmis_dosya_okunuyor() {
        // Not Defteri ve PowerShell'in `Set-Content -Encoding utf8`i dosyanın
        // başına EF BB BF koyuyor. Kırpılmazsa kullanıcı dosyaya tek satır
        // ekleyip bütün oturumunu kaybediyor.
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("oturum.json");
        let mut veri = vec![0xEF, 0xBB, 0xBF];
        veri.extend_from_slice(&serde_json::to_vec(&topla(&dolu_depo())).unwrap());
        fs::write(&yol, veri).unwrap();

        assert_eq!(oku(&yol).sekmeler.len(), 2);
    }

    #[test]
    fn dosya_yoksa_bos_oturum() {
        let dizin = tempfile::tempdir().unwrap();
        let o = oku(&dizin.path().join("yok.json"));
        assert!(o.sekmeler.is_empty());
    }

    #[test]
    fn ikisi_de_bozuksa_bos_oturum() {
        // Tarayıcı yine de açılıyor. Bu testin varlık sebebi bu tek cümle.
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("oturum.json");
        fs::write(&yol, b"bozuk").unwrap();
        fs::write(yedek_yolu(&yol), b"bu da bozuk").unwrap();
        assert!(oku(&yol).sekmeler.is_empty());
    }

    #[test]
    fn bilinmeyen_ebeveyn_koke_cekiliyor() {
        let mut depo = Depo::yeni();
        yukle(
            &mut depo,
            Oturum {
                surum: SURUM,
                etkin: None,
                gruplar: Vec::new(),
                sekmeler: vec![OturumSekmesi {
                    id: 5,
                    url: "https://a.example/".into(),
                    baslik: String::new(),
                    gecmis: vec![],
                    gecmis_konum: 0,
                    kaydirma: 0.0,
                    ebeveyn: Some(999),
                    sabit: false,
                    uyutma_istisnasi: false,
                    grup: None,
                }],
            },
        );
        assert_eq!(depo.hepsi()[0].ebeveyn, None);
    }

    #[test]
    fn geri_yuklenen_kimlik_yeniden_dagitilmiyor() {
        let mut depo = Depo::yeni();
        yukle(
            &mut depo,
            Oturum {
                surum: SURUM,
                etkin: None,
                gruplar: Vec::new(),
                sekmeler: vec![OturumSekmesi {
                    id: 40,
                    url: "https://a.example/".into(),
                    baslik: String::new(),
                    gecmis: vec![],
                    gecmis_konum: 0,
                    kaydirma: 0.0,
                    ebeveyn: None,
                    sabit: false,
                    uyutma_istisnasi: false,
                    grup: None,
                }],
            },
        );
        assert!(depo.ac("https://b.example/".into(), None, false) > 40);
    }
}
