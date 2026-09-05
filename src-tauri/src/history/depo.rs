//! Geçmiş ve yer imi sorguları.
//!
//! Bağlantı bir `Mutex` arkasında: SQLite bağlantısı `Sync` değil ve geçmiş
//! iki yerden yazılıyor (gezinme olayı, budama iş parçacığı). Kilit **kısa**
//! tutuluyor — sorgu dışında hiçbir şey yapılmıyor, motor çağrılmıyor.
//!
//! Arama `baslik_kucuk` sütununda koşuyor, `LOWER()` ile değil: SQLite'ın
//! `LOWER()` fonksiyonu ASCII dışına dokunmuyor ve `"İSTANBUL"` kaydı
//! `"istanbul"` aramasıyla eşleşmiyor (`docs/Depolama.md`, CLAUDE.md #10).

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};

use super::{alan_adi, gecisler, turkce_kucult, Aralik, GecmisKaydi, YerImi, YerImiKlasor};
use crate::hata::{MuirenHata, Sonuc};

fn hata(e: rusqlite::Error) -> MuirenHata {
    MuirenHata::Dosya(e.to_string())
}

/// Şu anki unix zaman damgası (saniye).
fn simdi() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        // Sistem saati 1970'in gerisindeyse damga 0 oluyor; kayıt yine
        // yazılıyor, yalnız sırası bozuk olur. Kaydı hiç yazmamaktan iyi.
        .unwrap_or(0)
}

pub struct Depo {
    db: Mutex<Connection>,
}

impl Depo {
    pub fn ac(yol: &Path) -> Sonuc<Self> {
        Ok(Depo {
            db: Mutex::new(gecisler::ac(yol)?),
        })
    }

    #[cfg(test)]
    fn bellekte() -> Self {
        let db = Connection::open_in_memory().unwrap();
        db.pragma_update(None, "foreign_keys", "ON").unwrap();
        gecisler::uygula(&db).unwrap();
        Depo { db: Mutex::new(db) }
    }

    // ------------------------------------------------------------- geçmiş

    /// Bir ziyareti kaydeder.
    ///
    /// Aynı adrese ikinci kez gidildiğinde **yeni satır açılmıyor**: `sayac`
    /// artıyor ve `ziyaret` tazeleniyor. Yoksa günde otuz kez açılan bir site
    /// geçmişi tek başına dolduruyor ve adres çubuğu önerileri kullanılamaz
    /// hâle geliyor.
    ///
    /// Başlık boş gelirse **eskisi korunuyor**: başarısız bir gezinmeden
    /// sonra gelen boş başlık, daha önce doğru kaydedilmiş başlığı silmemeli
    /// (CLAUDE.md #7 ile aynı gerekçe).
    pub fn ziyaret_ekle(&self, url: &str, baslik: &str) -> Sonuc<()> {
        let db = self.db.lock().unwrap();
        db.execute(
            "INSERT INTO gecmis (url, baslik, ziyaret, sayac, alan, baslik_kucuk)
             VALUES (?1, ?2, ?3, 1, ?4, ?5)
             ON CONFLICT(url) DO UPDATE SET
                 ziyaret      = excluded.ziyaret,
                 sayac        = gecmis.sayac + 1,
                 baslik       = CASE WHEN excluded.baslik = '' THEN gecmis.baslik
                                     ELSE excluded.baslik END,
                 baslik_kucuk = CASE WHEN excluded.baslik = '' THEN gecmis.baslik_kucuk
                                     ELSE excluded.baslik_kucuk END",
            params![url, baslik, simdi(), alan_adi(url), turkce_kucult(baslik)],
        )
        .map_err(hata)?;
        Ok(())
    }

    /// Var olan bir kaydın başlığını günceller. **Sayacı artırmıyor.**
    ///
    /// Gerekli çünkü sıralama tersine: `NavigationCompleted` başlıktan ÖNCE
    /// geliyor, yani ziyaret boş başlıkla kaydediliyor ve başlık saniyeler
    /// sonra ayrı bir olayla ulaşıyor. O anda `ziyaret_ekle` çağrılsaydı her
    /// sayfa açılışı sayacı iki artırırdı ve adres çubuğu önerileri
    /// bozulurdu.
    ///
    /// Kayıt yoksa hiçbir şey yapmıyor: başlık, geçmişe girmeyen bir sayfaya
    /// (kabuk sayfası, `about:`) ait olabilir.
    pub fn baslik_guncelle(&self, url: &str, baslik: &str) -> Sonuc<()> {
        if baslik.is_empty() {
            return Ok(());
        }
        let db = self.db.lock().unwrap();
        db.execute(
            "UPDATE gecmis SET baslik = ?2, baslik_kucuk = ?3 WHERE url = ?1",
            params![url, baslik, turkce_kucult(baslik)],
        )
        .map_err(hata)?;
        Ok(())
    }

    /// Geçmişte arama. Boş sorgu en son ziyaretleri veriyor.
    ///
    /// Sıralama: önce sayaç, sonra tazelik. Adres çubuğunda kullanıcının
    /// aradığı şey neredeyse her zaman "sık gittiğim yer"; ilk sıraya bir kez
    /// açılmış bir sayfa çıkarsa öneri işe yaramıyor.
    pub fn ara(&self, sorgu: &str, limit: u32) -> Sonuc<Vec<GecmisKaydi>> {
        let db = self.db.lock().unwrap();
        let k = turkce_kucult(sorgu.trim());
        // `limit` sınırlanıyor: arayüzden gelen bir sayı doğrudan sorguya
        // girmemeli.
        let limit = limit.clamp(1, 500) as i64;

        let mut ifade = db
            .prepare(
                "SELECT id, url, baslik, ziyaret, sayac, alan
                 FROM gecmis
                 WHERE ?1 = '' OR baslik_kucuk LIKE '%' || ?1 || '%'
                                OR url          LIKE '%' || ?1 || '%'
                 ORDER BY sayac DESC, ziyaret DESC
                 LIMIT ?2",
            )
            .map_err(hata)?;

        let satirlar = ifade
            .query_map(params![k, limit], |r| {
                Ok(GecmisKaydi {
                    id: r.get(0)?,
                    url: r.get(1)?,
                    baslik: r.get(2)?,
                    ziyaret: r.get(3)?,
                    sayac: r.get(4)?,
                    alan: r.get(5)?,
                })
            })
            .map_err(hata)?;

        satirlar.collect::<Result<Vec<_>, _>>().map_err(hata)
    }

    pub fn gecmis_sil(&self, id: i64) -> Sonuc<()> {
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM gecmis WHERE id = ?1", [id])
            .map_err(hata)?;
        Ok(())
    }

    /// Aralıktaki kayıtları siler. Dönüş: silinen satır sayısı.
    pub fn gecmis_temizle(&self, aralik: Aralik) -> Sonuc<u64> {
        let db = self.db.lock().unwrap();
        let n = db
            .execute(
                "DELETE FROM gecmis WHERE ziyaret >= ?1",
                [aralik.baslangic(simdi())],
            )
            .map_err(hata)?;
        Ok(n as u64)
    }

    /// Saklama süresini geçmiş kayıtları budar. Dönüş: silinen satır sayısı.
    ///
    /// `gun` anlamları (`docs/Depolama.md`): `-1` geçmiş tutulmuyor (hepsi
    /// silinir), `0` sınırsız (hiçbiri silinmez), pozitif değer gün sayısı.
    ///
    /// `VACUUM` **her budamada koşmuyor** — pahalı. Ayrı bir karar
    /// (`docs/Depolama.md`).
    pub fn buda(&self, gun: i32) -> Sonuc<u64> {
        if gun == 0 {
            return Ok(0);
        }
        let db = self.db.lock().unwrap();
        let sinir = if gun < 0 {
            // "Geçmiş tutma": var olanı da temizle.
            i64::MAX
        } else {
            simdi() - (gun as i64) * 86_400
        };
        let n = db
            .execute("DELETE FROM gecmis WHERE ziyaret < ?1", [sinir])
            .map_err(hata)?;
        Ok(n as u64)
    }

    // ----------------------------------------------------------- yer imleri

    pub fn yer_imi_ekle(&self, url: &str, baslik: &str, klasor: Option<i64>) -> Sonuc<i64> {
        let db = self.db.lock().unwrap();
        // Sıra: aynı klasörün sonuna.
        let sira: i64 = db
            .query_row(
                "SELECT COALESCE(MAX(sira), -1) + 1 FROM yer_imi
                 WHERE klasor IS ?1",
                params![klasor],
                |r| r.get(0),
            )
            .map_err(hata)?;
        db.execute(
            "INSERT INTO yer_imi (url, baslik, klasor, sira, eklendi)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![url, baslik, klasor, sira, simdi()],
        )
        .map_err(hata)?;
        Ok(db.last_insert_rowid())
    }

    pub fn yer_imi_sil(&self, id: i64) -> Sonuc<()> {
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM yer_imi WHERE id = ?1", [id])
            .map_err(hata)?;
        Ok(())
    }

    pub fn yer_imi_listesi(&self, klasor: Option<i64>) -> Sonuc<Vec<YerImi>> {
        let db = self.db.lock().unwrap();
        let mut ifade = db
            .prepare(
                "SELECT id, url, baslik, klasor, sira, eklendi
                 FROM yer_imi WHERE klasor IS ?1
                 ORDER BY sira ASC, id ASC",
            )
            .map_err(hata)?;
        let satirlar = ifade
            .query_map(params![klasor], |r| {
                Ok(YerImi {
                    id: r.get(0)?,
                    url: r.get(1)?,
                    baslik: r.get(2)?,
                    klasor: r.get(3)?,
                    sira: r.get(4)?,
                    eklendi: r.get(5)?,
                })
            })
            .map_err(hata)?;
        satirlar.collect::<Result<Vec<_>, _>>().map_err(hata)
    }

    pub fn klasor_ekle(&self, ad: &str, ebeveyn: Option<i64>) -> Sonuc<i64> {
        let db = self.db.lock().unwrap();
        db.execute(
            "INSERT INTO yer_imi_klasor (ad, ebeveyn, sira) VALUES (?1, ?2, 0)",
            params![ad, ebeveyn],
        )
        .map_err(hata)?;
        Ok(db.last_insert_rowid())
    }

    pub fn klasor_listesi(&self) -> Sonuc<Vec<YerImiKlasor>> {
        let db = self.db.lock().unwrap();
        let mut ifade = db
            .prepare("SELECT id, ad, ebeveyn, sira FROM yer_imi_klasor ORDER BY sira, id")
            .map_err(hata)?;
        let satirlar = ifade
            .query_map([], |r| {
                Ok(YerImiKlasor {
                    id: r.get(0)?,
                    ad: r.get(1)?,
                    ebeveyn: r.get(2)?,
                    sira: r.get(3)?,
                })
            })
            .map_err(hata)?;
        satirlar.collect::<Result<Vec<_>, _>>().map_err(hata)
    }

    /// Bu adres yer imlerinde mi. Adres çubuğundaki yıldız bunu okuyor.
    pub fn yer_imi_mi(&self, url: &str) -> Sonuc<Option<i64>> {
        let db = self.db.lock().unwrap();
        db.query_row(
            "SELECT id FROM yer_imi WHERE url = ?1 LIMIT 1",
            [url],
            |r| r.get(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            baska => Err(hata(baska)),
        })
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn ziyaret_ekleniyor_ve_araniyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://ornek.com/", "Örnek Sayfa").unwrap();
        let sonuc = d.ara("örnek", 10).unwrap();
        assert_eq!(sonuc.len(), 1);
        assert_eq!(sonuc[0].url, "https://ornek.com/");
        assert_eq!(sonuc[0].alan, "ornek.com");
        assert_eq!(sonuc[0].sayac, 1);
    }

    #[test]
    fn ayni_adres_sayaci_artiriyor_yeni_satir_acmiyor() {
        // Günde otuz kez açılan bir site geçmişi tek başına doldurmamalı.
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://ornek.com/", "Bir").unwrap();
        d.ziyaret_ekle("https://ornek.com/", "Bir").unwrap();
        d.ziyaret_ekle("https://ornek.com/", "Bir").unwrap();
        let sonuc = d.ara("", 10).unwrap();
        assert_eq!(sonuc.len(), 1);
        assert_eq!(sonuc[0].sayac, 3);
    }

    #[test]
    fn bos_baslik_eskisini_silmiyor() {
        // Başarısız bir gezinmeden sonra gelen boş başlık, daha önce doğru
        // kaydedilmiş başlığı silmemeli (CLAUDE.md #7).
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://ornek.com/", "Doğru Başlık")
            .unwrap();
        d.ziyaret_ekle("https://ornek.com/", "").unwrap();
        assert_eq!(d.ara("", 10).unwrap()[0].baslik, "Doğru Başlık");
    }

    #[test]
    fn turkce_arama_calisiyor() {
        // **Bu testin varlık sebebi CLAUDE.md #10.** SQLite'ın `LOWER()`
        // fonksiyonuyla yazılsaydı bu test kırmızı olurdu.
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://tr.wikipedia.org/wiki/Istanbul", "İSTANBUL")
            .unwrap();
        assert_eq!(d.ara("istanbul", 10).unwrap().len(), 1);
        assert_eq!(d.ara("İstanbul", 10).unwrap().len(), 1);
        assert_eq!(d.ara("İSTANBUL", 10).unwrap().len(), 1);
    }

    #[test]
    fn baslik_guncelleme_sayaci_artirmiyor() {
        // `NavigationCompleted` başlıktan ÖNCE geliyor: ziyaret boş başlıkla
        // kaydediliyor, başlık ayrı bir olayla ulaşıyor. O anda
        // `ziyaret_ekle` çağrılsaydı her sayfa açılışı sayacı iki artırırdı.
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://ornek.com/", "").unwrap();
        d.baslik_guncelle("https://ornek.com/", "Sonradan Gelen Başlık")
            .unwrap();
        let k = &d.ara("", 10).unwrap()[0];
        assert_eq!(k.baslik, "Sonradan Gelen Başlık");
        assert_eq!(k.sayac, 1);
    }

    #[test]
    fn baslik_guncellemesi_aramaya_giriyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://ornek.com/", "").unwrap();
        d.baslik_guncelle("https://ornek.com/", "İZMİR").unwrap();
        assert_eq!(d.ara("izmir", 10).unwrap().len(), 1);
    }

    #[test]
    fn bos_baslikla_guncelleme_yok_sayiliyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://ornek.com/", "Doğru").unwrap();
        d.baslik_guncelle("https://ornek.com/", "").unwrap();
        assert_eq!(d.ara("", 10).unwrap()[0].baslik, "Doğru");
    }

    #[test]
    fn arama_adreste_de_esliyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://crunchyroll.com/seri", "").unwrap();
        assert_eq!(d.ara("crunchy", 10).unwrap().len(), 1);
    }

    #[test]
    fn siralama_once_sayaca_sonra_tazelige_bakiyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://az.com/", "Az").unwrap();
        d.ziyaret_ekle("https://cok.com/", "Çok").unwrap();
        d.ziyaret_ekle("https://cok.com/", "Çok").unwrap();
        let sonuc = d.ara("", 10).unwrap();
        assert_eq!(sonuc[0].url, "https://cok.com/");
    }

    #[test]
    fn bos_sorgu_son_ziyaretleri_veriyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://bir.com/", "Bir").unwrap();
        d.ziyaret_ekle("https://iki.com/", "İki").unwrap();
        assert_eq!(d.ara("   ", 10).unwrap().len(), 2);
    }

    #[test]
    fn limit_sinirlaniyor() {
        let d = Depo::bellekte();
        for i in 0..5 {
            d.ziyaret_ekle(&format!("https://s{i}.com/"), "S").unwrap();
        }
        assert_eq!(d.ara("", 2).unwrap().len(), 2);
        // 0 ve devasa değerler sorguya olduğu gibi girmiyor.
        assert_eq!(d.ara("", 0).unwrap().len(), 1);
        assert_eq!(d.ara("", u32::MAX).unwrap().len(), 5);
    }

    #[test]
    fn gecmis_siliniyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://ornek.com/", "Ö").unwrap();
        let id = d.ara("", 1).unwrap()[0].id;
        d.gecmis_sil(id).unwrap();
        assert!(d.ara("", 10).unwrap().is_empty());
    }

    #[test]
    fn temizle_hepsi_hepsini_siliyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://a.com/", "A").unwrap();
        d.ziyaret_ekle("https://b.com/", "B").unwrap();
        assert_eq!(d.gecmis_temizle(Aralik::Hepsi).unwrap(), 2);
        assert!(d.ara("", 10).unwrap().is_empty());
    }

    #[test]
    fn budama_sinirsizda_hicbir_sey_silmiyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://a.com/", "A").unwrap();
        assert_eq!(d.buda(0).unwrap(), 0);
        assert_eq!(d.ara("", 10).unwrap().len(), 1);
    }

    #[test]
    fn budama_gecmis_tutma_ayarinda_hepsini_siliyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://a.com/", "A").unwrap();
        assert_eq!(d.buda(-1).unwrap(), 1);
        assert!(d.ara("", 10).unwrap().is_empty());
    }

    #[test]
    fn budama_taze_kaydi_birakiyor() {
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://a.com/", "A").unwrap();
        assert_eq!(d.buda(90).unwrap(), 0);
        assert_eq!(d.ara("", 10).unwrap().len(), 1);
    }

    // ----------------------------------------------------------- yer imleri

    #[test]
    fn yer_imi_ekleniyor_listeleniyor_siliniyor() {
        let d = Depo::bellekte();
        let id = d.yer_imi_ekle("https://ornek.com/", "Örnek", None).unwrap();
        let liste = d.yer_imi_listesi(None).unwrap();
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].baslik, "Örnek");
        d.yer_imi_sil(id).unwrap();
        assert!(d.yer_imi_listesi(None).unwrap().is_empty());
    }

    #[test]
    fn yer_imi_sirasi_sona_ekleniyor() {
        let d = Depo::bellekte();
        d.yer_imi_ekle("https://bir.com/", "Bir", None).unwrap();
        d.yer_imi_ekle("https://iki.com/", "İki", None).unwrap();
        let liste = d.yer_imi_listesi(None).unwrap();
        assert_eq!(liste[0].sira, 0);
        assert_eq!(liste[1].sira, 1);
    }

    #[test]
    fn klasordeki_yer_imleri_ayri_listeleniyor() {
        let d = Depo::bellekte();
        let k = d.klasor_ekle("Anime", None).unwrap();
        d.yer_imi_ekle("https://kok.com/", "Kök", None).unwrap();
        d.yer_imi_ekle("https://ic.com/", "İç", Some(k)).unwrap();
        assert_eq!(d.yer_imi_listesi(None).unwrap().len(), 1);
        assert_eq!(d.yer_imi_listesi(Some(k)).unwrap().len(), 1);
    }

    #[test]
    fn yer_imi_mi_dogru_cevap_veriyor() {
        let d = Depo::bellekte();
        assert_eq!(d.yer_imi_mi("https://ornek.com/").unwrap(), None);
        let id = d.yer_imi_ekle("https://ornek.com/", "Ö", None).unwrap();
        assert_eq!(d.yer_imi_mi("https://ornek.com/").unwrap(), Some(id));
    }

    #[test]
    fn gecmis_silmek_yer_imini_etkilemiyor() {
        // Yer imleri kullanıcının kendi ürettiği içerik: "temizle" düğmesiyle
        // kaybolmamalı (`docs/Depolama.md`).
        let d = Depo::bellekte();
        d.ziyaret_ekle("https://ornek.com/", "Ö").unwrap();
        d.yer_imi_ekle("https://ornek.com/", "Ö", None).unwrap();
        d.gecmis_temizle(Aralik::Hepsi).unwrap();
        assert_eq!(d.yer_imi_listesi(None).unwrap().len(), 1);
    }
}
