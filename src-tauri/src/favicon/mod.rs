//! Favicon deposu.
//!
//! Faz 1'in üçüncü açık maddesi ve Faz 3'ün ilk açık maddesi aynı iş
//! (`docs/Roadmap.md`). Sekme çubuğunda uzun süre yalnız durum noktası vardı.
//!
//! ## Neden bir depo, neden doğrudan adres değil
//!
//! `ICoreWebView2_15::FaviconUri` sayfanın favicon adresini veriyor ve onu
//! sekme çubuğunda bir `<img src>` olarak kullanmak tek satır olurdu. **Bu
//! yapılmıyor.** Sebebi `tabs::Sekme::favicon` alanının notunda yazılı: uzak
//! bir adresi kabuğa basmak, açılan her sitenin **kabuk penceresine** ağ
//! isteği yaptırabilmesi demek. Kabuk, sayfaların göremediği her şeyi görüyor;
//! oraya sayfanın seçtiği bir adres girmiyor.
//!
//! Bunun yerine motor baytları veriyor (`GetFavicon`, kendi önbelleğinden —
//! ikinci bir ağ isteği yok), depo onları diske yazıyor ve kabuğa yalnız bir
//! **kimlik** gidiyor.
//!
//! ## Kimlik içeriğin kendisinden
//!
//! Dosya adı içeriğin karması. Üç kazanç:
//!
//! - Aynı ikon yüzlerce sekmede tek dosya. Bir sitenin 30 sekmesi 30 kopya
//!   üretmiyor.
//! - Kimlik **sayfadan gelen hiçbir dizeyi taşımıyor**; dolayısıyla dosya adı
//!   üzerinden dizin taşması mümkün değil ([`kimlik_gecerli`] yine de
//!   doğruluyor — okuma yolu ayrı bir kapı).
//! - Değişmezlik: aynı kimlik hep aynı baytlar, yani arayüz kimliğe göre
//!   sınırsız önbellekleyebiliyor.
//!
//! Karma **kriptografik değil** ve olması gerekmiyor: çakışmanın bedeli yanlış
//! bir ikon göstermek. 128 bitlik FNV-1a bunun için fazlasıyla yeterli ve bir
//! bağımlılık eklemiyor.

use std::path::{Path, PathBuf};

use crate::hata::Sonuc;

/// Depoya girebilecek en büyük ikon. Motor tarafında da bir sınır var
/// (`motor/gercek.rs`, `FAVICON_SINIRI`); buradaki ikinci kapı, baytların
/// başka bir yoldan gelmesi ihtimaline karşı.
const AZAMI_BOYUT: usize = 512 * 1024;

/// Depo dizini.
pub fn dizin(veri_dizini: &Path) -> PathBuf {
    veri_dizini.join("favicon")
}

/// İçeriğin 128 bitlik FNV-1a karması, onaltılık.
///
/// **Saf ve testli.** Aynı baytlar hep aynı kimliği vermek zorunda: vermezse
/// arayüzün önbelleği her turda ıskalar ve sekme çubuğu ikonları titrer.
pub fn kimlik(veri: &[u8]) -> String {
    const TEMEL: u128 = 0x6c62272e07bb014262b821756295c58d;
    const CARPAN: u128 = 0x0000000001000000000000000000013b;

    let mut karma = TEMEL;
    for bayt in veri {
        karma ^= *bayt as u128;
        karma = karma.wrapping_mul(CARPAN);
    }
    format!("{karma:032x}")
}

/// Kimlik bu depoya ait olabilir mi.
///
/// **Bir güvenlik sınırı.** Kimlik arayüzden geri geliyor (`favicon_oku`
/// komutu) ve bir dosya yoluna dönüşüyor; `..\..\ayarlar.json` gibi bir dize
/// buradan geçemez. Yalnız 32 onaltılık karakter kabul ediliyor — yani
/// [`kimlik`] fonksiyonunun üretebileceği tek biçim.
pub fn kimlik_gecerli(kimlik: &str) -> bool {
    kimlik.len() == 32 && kimlik.chars().all(|c| c.is_ascii_hexdigit())
}

/// İkonu diske yazar ve kimliğini döndürür.
///
/// Aynı içerik ikinci kez geldiğinde dosya **yeniden yazılmıyor**: bir sitenin
/// 30 sekmesi 30 disk yazımı üretmiyor.
pub fn yaz(dizin: &Path, veri: &[u8]) -> Sonuc<String> {
    if veri.is_empty() || veri.len() > AZAMI_BOYUT {
        return Err(crate::hata::MuirenHata::Bicim("favicon boyutu".into()));
    }
    if !png_mi(veri) {
        // Motordan PNG istiyoruz (`COREWEBVIEW2_FAVICON_IMAGE_FORMAT_PNG`).
        // Başka bir şey geldiyse depoya girmiyor: arayüz `data:image/png`
        // diyor ve yanlış biçim sessizce kırık bir ikon üretirdi.
        return Err(crate::hata::MuirenHata::Bicim("favicon PNG değil".into()));
    }

    let kimlik = kimlik(veri);
    let yol = dizin.join(format!("{kimlik}.png"));
    if yol.exists() {
        return Ok(kimlik);
    }
    std::fs::create_dir_all(dizin)?;
    // Atomik yazım: yarım yazılmış bir PNG, kimliği "var" gösterip her
    // açılışta kırık ikon verirdi.
    let gecici = dizin.join(format!("{kimlik}.png.yeni"));
    std::fs::write(&gecici, veri)?;
    std::fs::rename(&gecici, &yol)?;
    Ok(kimlik)
}

/// İkonu `data:` adresi olarak okur.
///
/// Arayüze **bu** gidiyor, dosya yolu değil: kabuğa dosya sistemi kanalı
/// açmamak için (`capabilities/varsayilan.json` yalnız kabuğu tanıyor ve ona
/// da dosya okuma izni verilmiyor).
pub fn oku(dizin: &Path, kimlik: &str) -> Option<String> {
    if !kimlik_gecerli(kimlik) {
        return None;
    }
    let veri = std::fs::read(dizin.join(format!("{kimlik}.png"))).ok()?;
    if veri.len() > AZAMI_BOYUT || !png_mi(&veri) {
        return None;
    }
    Some(format!("data:image/png;base64,{}", base64_kodla(&veri)))
}

/// Depoda kullanılmayan ikonları siler. Dönüş: silinen dosya sayısı.
///
/// `kullanilan` şu an bir sekmeye ya da yer imine bağlı kimlikler. Budama
/// **veri temizleme ekranından** ve açılıştan çağrılıyor: depo kendiliğinden
/// küçülmüyor, çünkü kimlik içerikten geliyor ve hiçbir kayıt "artık bu ikon
/// lazım değil" demiyor.
pub fn buda(dizin: &Path, kullanilan: &[String]) -> Sonuc<u64> {
    let Ok(girisler) = std::fs::read_dir(dizin) else {
        // Depo hiç yaratılmamış: budanacak bir şey yok.
        return Ok(0);
    };
    let mut silinen = 0;
    for giris in girisler.flatten() {
        let yol = giris.path();
        let Some(ad) = yol.file_stem().and_then(|a| a.to_str()) else {
            continue;
        };
        // Depoya ait olmayan bir dosya (kullanıcı elle koymuş, yarım kalmış
        // `.yeni`) silinmiyor: bu fonksiyonun işi kendi çöpünü toplamak.
        if !kimlik_gecerli(ad) || yol.extension().is_none_or(|u| u != "png") {
            continue;
        }
        if !kullanilan.iter().any(|k| k == ad) && std::fs::remove_file(&yol).is_ok() {
            silinen += 1;
        }
    }
    Ok(silinen)
}

/// Deponun tamamını siler (veri temizleme ekranı).
pub fn temizle(dizin: &Path) -> Sonuc<u64> {
    buda(dizin, &[])
}

/// PNG imzası.
fn png_mi(veri: &[u8]) -> bool {
    veri.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])
}

/// Asgari base64 kodlayıcı.
///
/// Bir bağımlılık eklemek yerine yirmi satır. Saf ve testli. `pub(crate)`
/// çünkü ikinci kullanıcısı `theme::arkaplan_veri`: arka plan görseli de
/// kabuğa `data:` adresi olarak gidiyor, aynı gerekçeyle (dosya sistemi
/// kanalı açmamak için).
pub(crate) fn base64_kodla(veri: &[u8]) -> String {
    const ALFABE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut cikti = String::with_capacity(veri.len().div_ceil(3) * 4);
    for parca in veri.chunks(3) {
        let b0 = parca[0] as u32;
        let b1 = *parca.get(1).unwrap_or(&0) as u32;
        let b2 = *parca.get(2).unwrap_or(&0) as u32;
        let uclu = (b0 << 16) | (b1 << 8) | b2;

        cikti.push(ALFABE[(uclu >> 18) as usize & 63] as char);
        cikti.push(ALFABE[(uclu >> 12) as usize & 63] as char);
        cikti.push(if parca.len() > 1 {
            ALFABE[(uclu >> 6) as usize & 63] as char
        } else {
            '='
        });
        cikti.push(if parca.len() > 2 {
            ALFABE[uclu as usize & 63] as char
        } else {
            '='
        });
    }
    cikti
}

#[cfg(test)]
mod testler {
    use super::*;

    /// Küçük ama geçerli bir PNG imzası + gövde.
    fn png(govde: &[u8]) -> Vec<u8> {
        let mut v = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        v.extend_from_slice(govde);
        v
    }

    #[test]
    fn ayni_icerik_ayni_kimlik() {
        // Vermezse arayüzün önbelleği her turda ıskalıyor ve ikonlar titriyor.
        assert_eq!(kimlik(b"abc"), kimlik(b"abc"));
        assert_ne!(kimlik(b"abc"), kimlik(b"abd"));
    }

    #[test]
    fn kimlik_bicimi_sabit() {
        let k = kimlik(b"muiren");
        assert_eq!(k.len(), 32);
        assert!(kimlik_gecerli(&k));
    }

    #[test]
    fn dizin_tirmanmasi_kimlik_sayilmiyor() {
        // Kimlik arayüzden geri geliyor ve bir dosya yoluna dönüşüyor.
        assert!(!kimlik_gecerli("../../ayarlar.json"));
        assert!(!kimlik_gecerli(r"..\ayarlar"));
        assert!(!kimlik_gecerli(""));
        assert!(!kimlik_gecerli("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"));
        assert!(!kimlik_gecerli("abc"), "kısa kimlik geçerli sayıldı");
    }

    #[test]
    fn yaz_oku_gidis_donus() {
        let d = tempfile::tempdir().unwrap();
        let dizin = d.path();
        let veri = png(b"govde");

        let k = yaz(dizin, &veri).unwrap();
        let adres = oku(dizin, &k).unwrap();
        assert!(adres.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn ayni_ikon_ikinci_kez_yazilmiyor() {
        let d = tempfile::tempdir().unwrap();
        let veri = png(b"x");
        let k1 = yaz(d.path(), &veri).unwrap();
        let degisim = std::fs::metadata(d.path().join(format!("{k1}.png")))
            .unwrap()
            .modified()
            .unwrap();
        let k2 = yaz(d.path(), &veri).unwrap();
        assert_eq!(k1, k2);
        assert_eq!(
            std::fs::metadata(d.path().join(format!("{k2}.png")))
                .unwrap()
                .modified()
                .unwrap(),
            degisim,
            "aynı ikon yeniden yazıldı"
        );
    }

    #[test]
    fn png_olmayan_veri_reddediliyor() {
        let d = tempfile::tempdir().unwrap();
        assert!(yaz(d.path(), b"<svg>...</svg>").is_err());
        assert!(yaz(d.path(), b"").is_err());
    }

    #[test]
    fn cok_buyuk_ikon_reddediliyor() {
        let d = tempfile::tempdir().unwrap();
        let dev = png(&vec![0u8; AZAMI_BOYUT]);
        assert!(yaz(d.path(), &dev).is_err());
    }

    #[test]
    fn gecersiz_kimlikle_okuma_none() {
        let d = tempfile::tempdir().unwrap();
        assert!(oku(d.path(), "../ayarlar").is_none());
    }

    #[test]
    fn budama_kullanilmayani_siliyor() {
        let d = tempfile::tempdir().unwrap();
        let kalan = yaz(d.path(), &png(b"kalan")).unwrap();
        let giden = yaz(d.path(), &png(b"giden")).unwrap();

        assert_eq!(buda(d.path(), std::slice::from_ref(&kalan)).unwrap(), 1);
        assert!(d.path().join(format!("{kalan}.png")).exists());
        assert!(!d.path().join(format!("{giden}.png")).exists());
    }

    #[test]
    fn budama_yabanci_dosyaya_dokunmuyor() {
        let d = tempfile::tempdir().unwrap();
        let yabanci = d.path().join("notlarim.txt");
        std::fs::write(&yabanci, b"onemli").unwrap();
        yaz(d.path(), &png(b"a")).unwrap();

        buda(d.path(), &[]).unwrap();
        assert!(yabanci.exists(), "depoya ait olmayan dosya silindi");
    }

    #[test]
    fn budama_olmayan_dizinde_patlamiyor() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(buda(&d.path().join("yok"), &[]).unwrap(), 0);
    }

    #[test]
    fn base64_bilinen_degerler() {
        assert_eq!(base64_kodla(b""), "");
        assert_eq!(base64_kodla(b"f"), "Zg==");
        assert_eq!(base64_kodla(b"fo"), "Zm8=");
        assert_eq!(base64_kodla(b"foo"), "Zm9v");
        assert_eq!(base64_kodla(b"foob"), "Zm9vYg==");
        assert_eq!(base64_kodla(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_kodla(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_ikili_veri() {
        // Yalnız ASCII ile test etmek, üst bitleri kaybeden bir hatayı
        // gizlerdi.
        assert_eq!(base64_kodla(&[0xFF, 0xFE, 0xFD]), "//79");
        assert_eq!(base64_kodla(&[0x00, 0x00, 0x00]), "AAAA");
    }
}
