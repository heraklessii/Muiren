//! Süreç → sekme eşlemesi — **saf** (`docs/Bellek.md`, "Ölçüm").
//!
//! `docs/Bellek.md` sekme başına belleği "zor kısım" diye yazıp iki yol
//! bırakmıştı: (1) motordan süreç → çerçeve eşlemesini almak, (2) olmuyorsa
//! toplamı uyanık sekme sayısına bölüp "yaklaşık" demek. Uzun süre 2. yol
//! yürüdü ve `olcum_yaklasik` **her zaman** `true` idi.
//!
//! Bu dosya 1. yolun hesap tarafı. Motor `GetProcessExtendedInfos` ile hangi
//! sürecin hangi sekmelerin çerçevelerini taşıdığını söylüyor
//! (`motor::SurecKaydi`), Win32 hangi sürecin ne kadar yer tuttuğunu
//! (`olcum::surecler`); ikisini birleştirip sekme başına rakama çeviren yer
//! burası.
//!
//! Ayrım `esik.rs` ve `gecikme.rs` ile aynı: **hesap burada, sistem çağrısı
//! dışarıda.** Bu dosya bir PID'e bakmıyor, bir COM arayüzü tanımıyor, saat
//! okumuyor — dolayısıyla `--no-default-features` derlemesinde de test
//! ediliyor ve aşağıdaki testler onlarca kombinasyonu sabitliyor.
//!
//! **Neden bir defter, tek seferlik bir fonksiyon değil.** Atılmış bir
//! sekmenin süreci yok, dolayısıyla ölçülecek bir şeyi de yok — ama panelin
//! göstermesi gereken sayı tam da o: "bu sekme uyanık olsaydı ne kadar yer
//! tutardı". Cevap yalnız geçmiş turlarda duruyor. Defter onu saklıyor
//! ve **uyuyan sekmeyi ölçmek için uyandırmıyor** (CLAUDE.md #5).

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::motor::SurecKaydi;
use crate::tabs::SekmeId;

/// Bir sekmenin ölçülen belleği (`docs/IPC.md`, `BellekOzeti`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SekmeBellegi {
    pub id: SekmeId,
    pub mb: u64,
    /// Render süreci başka sekmelerle paylaşılıyor → rakam bölüştürülmüş.
    ///
    /// `--process-per-site` açıkken aynı siteden on sekme tek sürece
    /// düşüyor ve o sürecin belleği onlara **eşit** bölünüyor. Eşit bölmek
    /// gerçeği tam yansıtmıyor (bir sekme diğerinden ağır olabilir) ve
    /// arayüz bunu söylemek zorunda: paylaşılan bir rakamı tek sekmenin
    /// maliyeti gibi göstermek, kullanıcıyı yanlış sekmeyi kapatmaya iter.
    pub paylasimli: bool,
    /// Değer **bu turda ölçülmedi**; son bilinen değer gösteriliyor.
    ///
    /// Atılmış sekmenin süreci yok; uyuyan sekmenin süreci bazen ölçülüyor
    /// bazen ölçülmüyor. İkisinde de doğru cevap son bilinen değer —
    /// sıfır göstermek "bu sekme bedava" demek olurdu.
    pub bayat: bool,
}

/// Bir turun sonucu.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tur {
    /// Sekme başına bellek; yalnız bir değeri **olan** sekmeler.
    ///
    /// Hiç ölçülmemiş sekme (oturumdan atılmış doğan, bir kez bile webview
    /// almamış) listede yok. Sıfırla listelemek, ölçümün yokluğunu bir ölçüm
    /// sonucu gibi gösterirdi.
    pub sekmeler: Vec<SekmeBellegi>,
    /// Sekmelere düşmeyen WebView2 belleği: tarayıcı, GPU, ağ ve yardımcı
    /// süreçler + sahibi bulunamayan render süreçleri.
    ///
    /// Panelde ayrı bir satır olarak duruyor ve sebebi somut: sekme başına
    /// rakamların toplamı `toplam_mb`ye eşit çıkmıyor ve aradaki farkı
    /// söylemeyen bir panel, kullanıcıya ölçümün bozuk olduğunu düşündürür.
    pub ortak_mb: u64,
    /// Her render sürecinin sahibi bulundu mu.
    ///
    /// `false` ise eşleme eksik: ya motor süreç bilgisini vermiyor ya da bir
    /// render sürecinde tanımadığımız bir çerçeve var. `olcum_yaklasik`
    /// bunun aynası.
    pub tam: bool,
    /// Ölçülmüş kazanç: pasif sekmelerin **uyanıkken** tuttuğu yerden bugün
    /// tuttukları çıkarılmış hâli.
    ///
    /// Tahminden farkı, "uyusaydı" ile "uyuyor" arasındaki farkı gerçekten
    /// hesaplaması: uyuyan bir sekme sıfır yer tutmuyor (askıya alınmış
    /// render süreci hâlâ ayakta) ve tuttuğu yeri kazanç saymak, panelin
    /// gösterdiği tek büyük rakamı olduğundan iyi gösterirdi.
    pub kazanc_mb: u64,
    /// Kazancı **bilinmeyen** pasif sekme sayısı: uyanıkken hiç ölçülmemiş
    /// olanlar. Bunlar için tahmine düşülüyor ve `olcum_yaklasik` true
    /// kalıyor.
    pub kazanc_bilinmeyen: u32,
}

/// Bir sekme hakkında defterde duran şey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Kayit {
    /// En son ölçülen değer (sekme o an uyanık da olabilir uyuyan da).
    son_mb: u64,
    /// Sekme **uyanıkken** ölçülen son değer.
    ///
    /// Kazanç hesabının dayanağı. `son_mb` ile aynı şey değil: uyuduktan
    /// sonra ölçülen düşük değer `son_mb`yi ezerse "uyutmakla ne kazandık"
    /// sorusunun cevabı kaybolurdu.
    uyanikken_mb: Option<u64>,
    paylasimli: bool,
    /// Bu turda ölçüldü mü.
    taze: bool,
}

/// Süreç ölçümlerini sekme başına rakama çeviren defter.
///
/// Durum tutuyor ama **karar vermiyor**: çıktısı yalnız panele gidiyor, eşik
/// kararına değil. `docs/Bellek.md` bu ayrımı baştan koyuyor — uyutma kararı
/// boşta kalma süresine ve sistem baskısına dayanıyor, sekme başına rakama
/// değil. Bu yüzden buradaki bir hata paneli yanıltır, politikayı bozmaz.
#[derive(Debug, Default)]
pub struct Defter {
    kayitlar: HashMap<SekmeId, Kayit>,
}

impl Defter {
    /// Bir turu işler.
    ///
    /// - `kayitlar` motorun son bilinen süreç → sekme anlık görüntüsü.
    /// - `pid_mb` Win32'nin ölçtüğü süreç başına özel bellek (MB).
    /// - `sekmeler` o anki sekmeler ve uyanık olup olmadıkları. Kapanmış
    ///   sekmeler defterden **düşüyor**: aksi hâlde uzun bir oturumda defter
    ///   kapanan her sekmeyi sonsuza kadar taşırdı ve bellek iddiası olan bir
    ///   programda ölçüm aracının kendisi bir bellek kalemi olamaz.
    pub fn tur(
        &mut self,
        kayitlar: &[SurecKaydi],
        pid_mb: &HashMap<u32, u64>,
        sekmeler: &[(SekmeId, bool)],
    ) -> Tur {
        let canli: HashSet<SekmeId> = sekmeler.iter().map(|(id, _)| *id).collect();
        self.kayitlar.retain(|id, _| canli.contains(id));
        for k in self.kayitlar.values_mut() {
            k.taze = false;
        }

        let mut ortak_mb = 0u64;
        // Anlık görüntü hiç yoksa eşleme "tam" olamaz: doğrulanmadan `true`
        // denmiyor (`motor::Yetenekler::surec_bilgisi` ile aynı kural).
        let mut tam = !kayitlar.is_empty();

        for kayit in kayitlar {
            let mb = pid_mb.get(&kayit.pid).copied().unwrap_or(0);
            if !kayit.render {
                ortak_mb += mb;
                continue;
            }
            // Kabuk da bir sahip: arayüzümüz de bir render sürecinde koşuyor.
            // Sahip sayılmasaydı o süreç her turda "sahipsiz" olur, eşleme
            // hiçbir zaman tam çıkmaz ve panel kalıcı olarak "yaklaşık"
            // derdi (`motor::SurecKaydi::kabuk`).
            let sahip_sayisi = kayit.sekmeler.len() + usize::from(kayit.kabuk);
            // Sahibi bulunamayan render süreci: belleği ortak gidere yazılıyor
            // **ve** eşleme eksik sayılıyor. Sessizce yutmak, panelin eksik
            // bir tabloyu tam gibi göstermesi olurdu.
            if sahip_sayisi == 0 {
                ortak_mb += mb;
                tam = false;
                continue;
            }

            let pay = mb / sahip_sayisi as u64;
            let paylasimli = sahip_sayisi > 1;
            if kayit.kabuk {
                // Kabuğun payı ortak gidere: kullanıcı kabuğu kapatarak yer
                // açamaz, dolayısıyla o rakam bir sekmenin maliyeti değil.
                // Bölme kalanı da burada kalıyor; sekmelere dağıtılmayan
                // MB'nin kaybolması, panelin toplamını tutmaz hâle getirirdi.
                ortak_mb += mb - pay * kayit.sekmeler.len() as u64;
            }
            for id in &kayit.sekmeler {
                // Bir sekme birden çok render sürecinde olabilir (site
                // izolasyonu: farklı kökenli çerçeveler ayrı süreçlerde).
                // Payları toplamak doğru; ilkini alıp geri kalanını atmak,
                // gömülü oynatıcısı olan bir sayfayı olduğundan hafif
                // gösterirdi.
                let k = self.kayitlar.entry(*id).or_insert(Kayit {
                    son_mb: 0,
                    uyanikken_mb: None,
                    paylasimli: false,
                    taze: false,
                });
                if !k.taze {
                    // Turun ilk payı: eski değeri ezip sıfırdan topluyoruz.
                    k.son_mb = 0;
                    k.paylasimli = false;
                    k.taze = true;
                }
                k.son_mb += pay;
                k.paylasimli |= paylasimli;
            }
        }

        // Uyanıkken ölçülen değer ayrıca saklanıyor. Sıra önemli: sekmenin o
        // andaki durumu yukarıdaki döngüde bilinmiyor, çünkü orada süreçler
        // dolaşılıyor, sekmeler değil.
        for (id, uyanik) in sekmeler {
            if let Some(k) = self.kayitlar.get_mut(id) {
                if *uyanik && k.taze && k.son_mb > 0 {
                    k.uyanikken_mb = Some(k.son_mb);
                }
            }
        }

        let mut kazanc_mb = 0u64;
        let mut kazanc_bilinmeyen = 0u32;
        let mut liste = Vec::new();
        for (id, uyanik) in sekmeler {
            let Some(k) = self.kayitlar.get(id) else {
                // Hiç ölçülmemiş sekme. Pasifse kazancı da bilinmiyor.
                if !uyanik {
                    kazanc_bilinmeyen += 1;
                }
                continue;
            };
            liste.push(SekmeBellegi {
                id: *id,
                mb: k.son_mb,
                paylasimli: k.paylasimli,
                bayat: !k.taze,
            });
            if *uyanik {
                continue;
            }
            // Pasif sekme. Kazanç = uyanıkken tuttuğu yer − bugün tuttuğu yer.
            // Atılmış sekmede ikinci terim sıfır; uyuyanda askıya alınmış
            // sürecin kalan payı.
            match k.uyanikken_mb {
                Some(uyanikken) => {
                    let guncel = if k.taze { k.son_mb } else { 0 };
                    kazanc_mb += uyanikken.saturating_sub(guncel);
                }
                None => kazanc_bilinmeyen += 1,
            }
        }
        liste.sort_by_key(|s| s.id);

        Tur {
            sekmeler: liste,
            ortak_mb,
            tam,
            kazanc_mb,
            kazanc_bilinmeyen,
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn render(pid: u32, sekmeler: &[SekmeId]) -> SurecKaydi {
        SurecKaydi {
            pid,
            render: true,
            sekmeler: sekmeler.to_vec(),
            kabuk: false,
        }
    }

    fn ortak(pid: u32) -> SurecKaydi {
        SurecKaydi {
            pid,
            render: false,
            sekmeler: Vec::new(),
            kabuk: false,
        }
    }

    /// Kabuğun arayüzünü taşıyan render süreci.
    fn kabuk(pid: u32, sekmeler: &[SekmeId]) -> SurecKaydi {
        SurecKaydi {
            pid,
            render: true,
            sekmeler: sekmeler.to_vec(),
            kabuk: true,
        }
    }

    fn mb(pairs: &[(u32, u64)]) -> HashMap<u32, u64> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn tek_sekmeli_surec_tam_yaziliyor() {
        let mut d = Defter::default();
        let t = d.tur(&[render(100, &[1])], &mb(&[(100, 180)]), &[(1, true)]);
        assert_eq!(
            t.sekmeler,
            vec![SekmeBellegi {
                id: 1,
                mb: 180,
                paylasimli: false,
                bayat: false
            }]
        );
        assert!(t.tam);
        assert_eq!(t.ortak_mb, 0);
    }

    #[test]
    fn paylasilan_surec_esit_bolunuyor_ve_isaretleniyor() {
        // `--process-per-site` açıkken beklenen hâl: aynı siteden üç sekme
        // tek süreçte.
        let mut d = Defter::default();
        let t = d.tur(
            &[render(100, &[1, 2, 3])],
            &mb(&[(100, 300)]),
            &[(1, true), (2, true), (3, true)],
        );
        assert_eq!(t.sekmeler.len(), 3);
        for s in &t.sekmeler {
            assert_eq!(s.mb, 100);
            assert!(s.paylasimli, "paylaşılan rakam öyle etiketlenmeli");
        }
    }

    #[test]
    fn sekme_birden_cok_surecte_toplaniyor() {
        // Site izolasyonu: sayfanın kendi süreci + gömülü oynatıcının süreci.
        // İlkini alıp diğerini atmak sayfayı olduğundan hafif gösterirdi.
        let mut d = Defter::default();
        let t = d.tur(
            &[render(100, &[1]), render(101, &[1])],
            &mb(&[(100, 120), (101, 40)]),
            &[(1, true)],
        );
        assert_eq!(t.sekmeler[0].mb, 160);
    }

    #[test]
    fn render_disi_surecler_ortak_gidere_yaziliyor() {
        let mut d = Defter::default();
        let t = d.tur(
            &[render(100, &[1]), ortak(7), ortak(8)],
            &mb(&[(100, 180), (7, 90), (8, 30)]),
            &[(1, true)],
        );
        assert_eq!(t.sekmeler[0].mb, 180);
        assert_eq!(t.ortak_mb, 120);
        assert!(t.tam, "tarayıcı ve GPU süreçleri eşlemeyi eksik yapmıyor");
    }

    #[test]
    fn sahipsiz_render_sureci_eslemeyi_eksik_yapiyor() {
        let mut d = Defter::default();
        let t = d.tur(
            &[render(100, &[1]), render(101, &[])],
            &mb(&[(100, 180), (101, 60)]),
            &[(1, true)],
        );
        assert!(!t.tam, "tanımadığımız bir render süreci varken tam denmez");
        assert_eq!(t.ortak_mb, 60, "sahipsiz bellek kaybolmuyor");
    }

    #[test]
    fn kabuk_sureci_ortak_gidere_yaziliyor_ve_eslemeyi_bozmuyor() {
        // Kabuğun arayüzü de bir render sürecinde koşuyor. Tanınmasaydı
        // eşleme hiçbir turda tam çıkmaz, panel kalıcı olarak "yaklaşık"
        // derdi — sekme başına ölçümün bütün amacı da orada biterdi.
        let mut d = Defter::default();
        let t = d.tur(
            &[render(100, &[1]), kabuk(101, &[])],
            &mb(&[(100, 180), (101, 90)]),
            &[(1, true)],
        );
        assert!(t.tam, "kabuğun süreci sahipsiz sayılmıyor");
        assert_eq!(t.ortak_mb, 90, "kabuğun belleği ortak gider");
        assert_eq!(t.sekmeler.len(), 1);
        assert_eq!(t.sekmeler[0].mb, 180, "kabuk bir sekmeye yazılmıyor");
    }

    #[test]
    fn kabuk_bir_sekmeyle_ayni_surecteyse_pay_bolunuyor() {
        // Kabuk ayrı bir kökende ve normalde ayrı süreçte, ama süreç sınırına
        // dayanılmışsa aynı sürece düşebiliyor. Bütün süreci sekmeye yazmak
        // o sekmeyi olduğundan ağır gösterirdi.
        let mut d = Defter::default();
        let t = d.tur(&[kabuk(100, &[1])], &mb(&[(100, 200)]), &[(1, true)]);
        assert_eq!(t.sekmeler[0].mb, 100);
        assert!(
            t.sekmeler[0].paylasimli,
            "paylaşılan rakam öyle etiketleniyor"
        );
        assert_eq!(t.ortak_mb, 100, "kabuğun payı ortak giderde");
        assert!(t.tam);
    }

    #[test]
    fn bolme_kalani_kabuk_payinda_kaybolmuyor() {
        // 100 MB, kabuk + iki sekme → sekme başına 33, kalan 34 kabuğa.
        // Kalanı düşürmek panelin toplamını tutmaz hâle getirirdi.
        let mut d = Defter::default();
        let t = d.tur(
            &[kabuk(100, &[1, 2])],
            &mb(&[(100, 100)]),
            &[(1, true), (2, true)],
        );
        assert_eq!(t.sekmeler[0].mb, 33);
        assert_eq!(t.sekmeler[1].mb, 33);
        assert_eq!(t.ortak_mb, 34);
        assert_eq!(t.sekmeler[0].mb + t.sekmeler[1].mb + t.ortak_mb, 100);
    }

    #[test]
    fn anlik_goruntu_yokken_tam_denmiyor() {
        // Motor süreç bilgisini vermiyor (eski Runtime, motorsuz derleme).
        let mut d = Defter::default();
        let t = d.tur(&[], &mb(&[]), &[(1, true)]);
        assert!(!t.tam);
        assert!(t.sekmeler.is_empty());
    }

    #[test]
    fn uyuyan_sekme_son_bilinen_degeri_koruyor() {
        let mut d = Defter::default();
        d.tur(&[render(100, &[1])], &mb(&[(100, 200)]), &[(1, true)]);
        // Sekme atıldı: süreci yok, ölçülecek bir şey de yok.
        let t = d.tur(&[ortak(7)], &mb(&[(7, 50)]), &[(1, false)]);
        assert_eq!(t.sekmeler[0].mb, 200);
        assert!(t.sekmeler[0].bayat, "ölçülmeyen değer öyle etiketlenmeli");
    }

    #[test]
    fn kazanc_uyanikken_olculen_degerden_hesaplaniyor() {
        let mut d = Defter::default();
        d.tur(&[render(100, &[1])], &mb(&[(100, 200)]), &[(1, true)]);
        let t = d.tur(&[ortak(7)], &mb(&[(7, 50)]), &[(1, false)]);
        assert_eq!(t.kazanc_mb, 200, "atılmış sekmenin tamamı kazanç");
        assert_eq!(t.kazanc_bilinmeyen, 0);
    }

    #[test]
    fn uyuyan_sekmenin_kalan_payi_kazanctan_dusuluyor() {
        // Askıya alınmış render süreci sıfır yer tutmuyor. Tuttuğu yeri
        // kazanç saymak, panelin tek büyük rakamını şişirirdi.
        let mut d = Defter::default();
        d.tur(&[render(100, &[1])], &mb(&[(100, 200)]), &[(1, true)]);
        let t = d.tur(&[render(100, &[1])], &mb(&[(100, 60)]), &[(1, false)]);
        assert_eq!(t.kazanc_mb, 140);
        assert_eq!(t.sekmeler[0].mb, 60, "panel bugünkü değeri gösteriyor");
        assert!(!t.sekmeler[0].bayat);
    }

    #[test]
    fn uyanikken_olculen_deger_uyku_olcumuyle_ezilmiyor() {
        let mut d = Defter::default();
        d.tur(&[render(100, &[1])], &mb(&[(100, 200)]), &[(1, true)]);
        // İki tur uyku: ikincisinde de kazanç 200 − 60 olmalı, 0 değil.
        d.tur(&[render(100, &[1])], &mb(&[(100, 60)]), &[(1, false)]);
        let t = d.tur(&[render(100, &[1])], &mb(&[(100, 60)]), &[(1, false)]);
        assert_eq!(t.kazanc_mb, 140);
    }

    #[test]
    fn hic_olculmemis_pasif_sekme_bilinmeyen_sayiliyor() {
        // Oturumdan `Atilmis` doğan sekme: bir kez bile webview almadı.
        let mut d = Defter::default();
        let t = d.tur(
            &[render(100, &[1])],
            &mb(&[(100, 180)]),
            &[(1, true), (2, false)],
        );
        assert_eq!(t.kazanc_bilinmeyen, 1);
        assert_eq!(t.kazanc_mb, 0);
        assert_eq!(
            t.sekmeler.len(),
            1,
            "ölçülmemiş sekme sıfırla listelenmiyor"
        );
    }

    #[test]
    fn kapanan_sekme_defterden_dusuyor() {
        let mut d = Defter::default();
        d.tur(
            &[render(100, &[1]), render(101, &[2])],
            &mb(&[(100, 100), (101, 100)]),
            &[(1, true), (2, true)],
        );
        assert_eq!(d.kayitlar.len(), 2);
        d.tur(&[render(100, &[1])], &mb(&[(100, 100)]), &[(1, true)]);
        assert_eq!(d.kayitlar.len(), 1, "kapanan sekme defterde kalmıyor");
    }

    #[test]
    fn olculemeyen_pid_sekmeyi_dusurmuyor() {
        // `OpenProcess` bir sürece izin vermeyebilir. O süreç sıfır sayılıyor
        // ama sekme listeden düşmüyor: eşleme bilgisi hâlâ doğru.
        let mut d = Defter::default();
        let t = d.tur(&[render(100, &[1])], &mb(&[]), &[(1, true)]);
        assert_eq!(t.sekmeler[0].mb, 0);
        assert!(t.tam);
    }

    #[test]
    fn liste_kimlige_gore_sirali() {
        let mut d = Defter::default();
        let t = d.tur(
            &[render(100, &[3]), render(101, &[1]), render(102, &[2])],
            &mb(&[(100, 30), (101, 30), (102, 30)]),
            &[(3, true), (1, true), (2, true)],
        );
        assert_eq!(
            t.sekmeler.iter().map(|s| s.id).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }
}
