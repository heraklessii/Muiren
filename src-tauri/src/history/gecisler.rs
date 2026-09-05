//! Şema ve sürüm yükseltme.
//!
//! > **Kural (CLAUDE.md #12):** `CREATE TABLE IF NOT EXISTS` var olan tabloyu
//! > DEĞİŞTİRMİYOR. Kullanıcının diskindeki veritabanı eski şemada kalıyor ve
//! > ilk sorguda patlıyor. Yeni bir sütun eklerken şemayı düzenlemek
//! > **yetmez**, buraya bir geçiş adımı da yazılır.
//!
//! Her adım **tekrar çalıştırılabilir** olmak zorunda (yarıda kesilen geçiş)
//! ama geri alınabilir olmak zorunda değil (`docs/Depolama.md`).

use rusqlite::Connection;

use crate::hata::{MuirenHata, Sonuc};

/// Şemanın bugünkü sürümü. Yeni adım eklendiğinde bu da artıyor.
pub const HEDEF_SURUM: i64 = 1;

fn hata(e: rusqlite::Error) -> MuirenHata {
    MuirenHata::Dosya(e.to_string())
}

/// Veritabanını açar, ayarlarını yapar ve geçişleri uygular.
pub fn ac(yol: &std::path::Path) -> Sonuc<Connection> {
    let db = Connection::open(yol).map_err(hata)?;

    // WAL: okuma ile yazma birbirini kilitlemiyor. Adres çubuğu her tuşta
    // geçmişte arama yaparken arka planda budama koşabiliyor.
    db.pragma_update(None, "journal_mode", "WAL")
        .map_err(hata)?;
    // `NORMAL`: `FULL`in her işlemde fsync'i, saniyede birkaç kez geçmiş
    // yazan bir tarayıcıda diski gereksiz yoruyor. WAL ile birlikte
    // dayanıklılık kaybı yalnız işletim sistemi çökmesinde ve kaybedilen şey
    // son birkaç geçmiş kaydı.
    db.pragma_update(None, "synchronous", "NORMAL")
        .map_err(hata)?;
    db.pragma_update(None, "foreign_keys", "ON").map_err(hata)?;

    uygula(&db)?;
    Ok(db)
}

pub fn uygula(db: &Connection) -> Sonuc<()> {
    // `surum` tablosu her adımdan önce var olmak zorunda: sürümü okuyacağımız
    // yer orası.
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS surum (
             anahtar TEXT PRIMARY KEY,
             deger   INTEGER NOT NULL
         );",
    )
    .map_err(hata)?;

    let mevcut = surum_oku(db)?;
    if mevcut < 1 {
        adim_1(db)?;
    }
    // if mevcut < 2 { adim_2(db)?; }   ← örn: gecmis.favicon eklendi

    surum_yaz(db, HEDEF_SURUM)
}

/// İlk şema (`docs/Depolama.md`).
fn adim_1(db: &Connection) -> Sonuc<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS gecmis (
             id           INTEGER PRIMARY KEY,
             url          TEXT NOT NULL,
             baslik       TEXT NOT NULL DEFAULT '',
             ziyaret      INTEGER NOT NULL,
             sayac        INTEGER NOT NULL DEFAULT 1,
             alan         TEXT NOT NULL,
             baslik_kucuk TEXT NOT NULL DEFAULT ''
         );
         CREATE INDEX IF NOT EXISTS gecmis_ziyaret ON gecmis(ziyaret DESC);
         CREATE INDEX IF NOT EXISTS gecmis_alan    ON gecmis(alan);
         -- Aynı adrese ikinci kez gidildiğinde yeni satır değil `sayac`
         -- artıyor; bu indeks o `UPDATE`in aradığı satırı bulan yer ve
         -- aynı zamanda tekrarları engelliyor.
         CREATE UNIQUE INDEX IF NOT EXISTS gecmis_url ON gecmis(url);

         CREATE TABLE IF NOT EXISTS yer_imi_klasor (
             id      INTEGER PRIMARY KEY,
             ad      TEXT NOT NULL,
             ebeveyn INTEGER REFERENCES yer_imi_klasor(id) ON DELETE CASCADE,
             sira    INTEGER NOT NULL DEFAULT 0
         );

         CREATE TABLE IF NOT EXISTS yer_imi (
             id      INTEGER PRIMARY KEY,
             url     TEXT NOT NULL,
             baslik  TEXT NOT NULL,
             klasor  INTEGER REFERENCES yer_imi_klasor(id) ON DELETE CASCADE,
             sira    INTEGER NOT NULL DEFAULT 0,
             eklendi INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS yer_imi_klasor_ix ON yer_imi(klasor);",
    )
    .map_err(hata)
}

pub fn surum_oku(db: &Connection) -> Sonuc<i64> {
    db.query_row("SELECT deger FROM surum WHERE anahtar = 'sema'", [], |r| {
        r.get(0)
    })
    .or_else(|e| match e {
        // Kayıt yoksa sürüm 0: veritabanı yeni.
        rusqlite::Error::QueryReturnedNoRows => Ok(0),
        baska => Err(hata(baska)),
    })
}

fn surum_yaz(db: &Connection, surum: i64) -> Sonuc<()> {
    db.execute(
        "INSERT INTO surum (anahtar, deger) VALUES ('sema', ?1)
         ON CONFLICT(anahtar) DO UPDATE SET deger = ?1",
        [surum],
    )
    .map_err(hata)?;
    Ok(())
}

#[cfg(test)]
mod testler {
    use super::*;

    fn bellekte() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        uygula(&db).unwrap();
        db
    }

    #[test]
    fn yeni_veritabani_hedef_surumde() {
        let db = bellekte();
        assert_eq!(surum_oku(&db).unwrap(), HEDEF_SURUM);
    }

    #[test]
    fn tablolar_olusuyor() {
        let db = bellekte();
        for tablo in ["gecmis", "yer_imi", "yer_imi_klasor", "surum"] {
            let var: i64 = db
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [tablo],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(var, 1, "{tablo} yok");
        }
    }

    #[test]
    fn gecis_tekrar_calistirilabiliyor() {
        // Yarıda kesilen bir geçiş ikinci açılışta baştan koşuyor; adımların
        // tekrara dayanıklı olması şart (`docs/Depolama.md`).
        let db = bellekte();
        uygula(&db).unwrap();
        uygula(&db).unwrap();
        assert_eq!(surum_oku(&db).unwrap(), HEDEF_SURUM);
    }

    #[test]
    fn ayni_url_iki_kez_eklenemiyor() {
        // `gecmis_url` benzersiz indeksi, "aynı adrese ikinci kez gidilince
        // sayac artsın" davranışının temeli.
        let db = bellekte();
        db.execute(
            "INSERT INTO gecmis (url, baslik, ziyaret, alan) VALUES ('u', '', 1, 'a')",
            [],
        )
        .unwrap();
        let ikinci = db.execute(
            "INSERT INTO gecmis (url, baslik, ziyaret, alan) VALUES ('u', '', 2, 'a')",
            [],
        );
        assert!(ikinci.is_err());
    }

    #[test]
    fn klasor_silinince_yer_imleri_de_gidiyor() {
        let db = Connection::open_in_memory().unwrap();
        db.pragma_update(None, "foreign_keys", "ON").unwrap();
        uygula(&db).unwrap();
        db.execute(
            "INSERT INTO yer_imi_klasor (id, ad) VALUES (1, 'Anime')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO yer_imi (url, baslik, klasor, eklendi) VALUES ('u', 'b', 1, 0)",
            [],
        )
        .unwrap();
        db.execute("DELETE FROM yer_imi_klasor WHERE id = 1", [])
            .unwrap();
        let kalan: i64 = db
            .query_row("SELECT count(*) FROM yer_imi", [], |r| r.get(0))
            .unwrap();
        assert_eq!(kalan, 0);
    }
}
