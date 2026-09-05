//! Kısayol tablosu — **tek kaynak**, saf.
//!
//! `docs/Frontend.md` şunu söylüyor: webview odaktayken tuşlar kabuğa
//! ulaşmıyor, dolayısıyla her kısayolun iki karşılığı olmak zorunda. Bunu iki
//! ayrı tablo yazarak yapmıyoruz — iki tablo sessizce ayrışır ve sonuç "bazen
//! çalışmıyor" diye rapor edilir (Muiply'dan devralınan ders).
//!
//! Tablo burada, ona **iki kapıdan** giriliyor:
//!
//! 1. Kabuk odaktayken — React `keydown` dinleyicisi `kisayol_bas` komutunu
//!    çağırıyor.
//! 2. Sayfa odaktayken — `motor/gercek.rs` içindeki `AcceleratorKeyPressed`
//!    kaydı aynı [`coz`] fonksiyonunu çağırıyor.
//!
//! Burada sistem çağrısı, motor çağrısı ve durum yok: girdi tuş + değiştirici,
//! çıktı bir eylem. Uygulaması `tabs/surucu.rs` içinde.

use serde::Serialize;

/// Bir kısayolun karşılığı olan eylem.
///
/// [`Kisayol::arayuz_isi`] ayrımı önemli: bir kısmı backend eylemi (sekme aç,
/// uyut), bir kısmı yalnız arayüzü ilgilendiriyor (adres çubuğuna odaklan).
/// Sayfa odaktayken ikincisi `muiren://kisayol` olayıyla kabuğa gönderiliyor
/// (`docs/IPC.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kisayol {
    // --- backend eylemleri ---
    YeniSekme,
    /// `Ctrl+Shift+N` — gizli sekme.
    ///
    /// Chrome bu tuşla gizli **pencere** açıyor. Muiren tek pencerede
    /// çalıştığı için (`lib.rs`, pencere düzeni) karşılığı bir sekme: kas
    /// hafızası aynı tuşta kalıyor, sonuç mimariye uyuyor. Sekme çubuğunda
    /// ayrı bir işaretle çiziliyor — kullanıcının hangi sekmenin gizli
    /// olduğunu görmesi şart (`tabs::SekmeOzeti::gizli`).
    GizliSekme,
    SekmeKapat,
    GeriAl,
    SonrakiSekme,
    OncekiSekme,
    /// `Ctrl+1..8` — 1 tabanlı sıra.
    SekmeNo(u8),
    /// `Ctrl+9` son sekme. Chrome davranışı; kas hafızası orada.
    SonSekme,
    BuSekmeyiUyut,
    HepsiniUyut,
    Yenile,
    Geri,
    Ileri,
    Durdur,

    // --- arayüz işleri ---
    AdresOdak,
    SekmeArama,
    BellekPaneli,
    YanPanel,
    TamEkran,
}

impl Kisayol {
    /// Bu eylemi arayüz mü yapıyor?
    ///
    /// Sayfa odaktayken backend eylemleri doğrudan uygulanıyor, arayüz işleri
    /// olayla kabuğa gönderiliyor. Ayrım tek yerde durunca yeni bir kısayol
    /// eklerken "bunu kim yapacak" sorusu bir kere cevaplanıyor.
    pub fn arayuz_isi(self) -> bool {
        matches!(
            self,
            Kisayol::AdresOdak
                | Kisayol::SekmeArama
                | Kisayol::BellekPaneli
                | Kisayol::YanPanel
                | Kisayol::TamEkran
        )
    }
}

/// Tuş + değiştiricileri eyleme çevirir.
///
/// `tus` normalleştirilmiş bir ad: tek harf (`"t"`), rakam (`"1"`) ya da
/// `"tab"`, `"f5"`, `"f11"`, `"arrowleft"`, `"arrowright"`, `"escape"`.
///
/// Küçültme **ASCII**: bunlar kullanıcı metni değil, tuş adları. Türkçe
/// küçültme (CLAUDE.md #10) buraya uygulanmaz — uygulanırsa `"I"` tuşu `"ı"`
/// olur ve tablo kaçar.
pub fn coz(tus: &str, ctrl: bool, shift: bool, alt: bool) -> Option<Kisayol> {
    let t = tus.trim().to_ascii_lowercase();

    // Değiştiricisiz ve Alt'lı olanlar önce: Ctrl kolu bunları görmemeli.
    match t.as_str() {
        "f5" if !ctrl && !alt => return Some(Kisayol::Yenile),
        "f11" if !ctrl && !alt && !shift => return Some(Kisayol::TamEkran),
        "escape" if !ctrl && !alt && !shift => return Some(Kisayol::Durdur),
        "arrowleft" if alt && !ctrl => return Some(Kisayol::Geri),
        "arrowright" if alt && !ctrl => return Some(Kisayol::Ileri),
        _ => {}
    }

    if ctrl && alt {
        // Bu kol kasıtlı olarak dar: `AltGr` Windows'ta Ctrl+Alt olarak
        // görünüyor ve Türkçe klavyede karakter üretiyor. Genişletilirse
        // kullanıcı yazarken kısayol tetikler.
        return match t.as_str() {
            "z" => Some(Kisayol::HepsiniUyut),
            _ => None,
        };
    }

    if !ctrl {
        return None;
    }

    if shift {
        return match t.as_str() {
            "t" => Some(Kisayol::GeriAl),
            "n" => Some(Kisayol::GizliSekme),
            "a" => Some(Kisayol::SekmeArama),
            "m" => Some(Kisayol::BellekPaneli),
            "e" => Some(Kisayol::YanPanel),
            "z" => Some(Kisayol::BuSekmeyiUyut),
            "tab" => Some(Kisayol::OncekiSekme),
            _ => None,
        };
    }

    match t.as_str() {
        "t" => Some(Kisayol::YeniSekme),
        "w" => Some(Kisayol::SekmeKapat),
        "l" => Some(Kisayol::AdresOdak),
        "r" => Some(Kisayol::Yenile),
        "tab" => Some(Kisayol::SonrakiSekme),
        "9" => Some(Kisayol::SonSekme),
        // `Ctrl+1..8` — `"9"` yukarıda ele alındı, `"0"` tabloda yok.
        _ if t.len() == 1 && matches!(t.as_bytes()[0], b'1'..=b'8') => {
            Some(Kisayol::SekmeNo(t.as_bytes()[0] - b'0'))
        }
        _ => None,
    }
}

/// Windows sanal tuş kodunu [`coz`]un beklediği ada çevirir.
///
/// Motor tarafı (`AcceleratorKeyPressed`) tuşu `VirtualKey` olarak veriyor,
/// React tarafı `KeyboardEvent.key` olarak. İkisinin **aynı** tabloya girmesi
/// için çevirinin tek yeri burası.
///
/// Tanınmayan kod `None`: tabloda olmayan tuşun eylemi de yok.
pub fn vk_adi(vk: u32) -> Option<String> {
    match vk {
        0x09 => Some("tab".into()),        // VK_TAB
        0x1B => Some("escape".into()),     // VK_ESCAPE
        0x25 => Some("arrowleft".into()),  // VK_LEFT
        0x27 => Some("arrowright".into()), // VK_RIGHT
        // Rakamların sanal kodu ASCII rakamla aynı.
        0x30..=0x39 => Some(((vk as u8) as char).to_string()),
        // Harflerin sanal kodu ASCII BÜYÜK harfle aynı.
        0x41..=0x5A => Some(((vk as u8) as char).to_ascii_lowercase().to_string()),
        // VK_F1..VK_F12
        0x70..=0x7B => Some(format!("f{}", vk - 0x6F)),
        _ => None,
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn gizli_sekme_kisayolu() {
        assert_eq!(coz("n", true, true, false), Some(Kisayol::GizliSekme));
        // Shift'siz `Ctrl+N` bir şey YAPMIYOR: Chrome'da yeni pencere açıyor
        // ve Muiren tek pencerede çalışıyor. Sessizce yeni sekme açsaydık
        // kullanıcı pencere beklerken sekme bulurdu.
        assert_eq!(coz("n", true, false, false), None);
        assert!(!Kisayol::GizliSekme.arayuz_isi(), "gizli sekme backend işi");
    }

    #[test]
    fn temel_sekme_kisayollari() {
        assert_eq!(coz("t", true, false, false), Some(Kisayol::YeniSekme));
        assert_eq!(coz("w", true, false, false), Some(Kisayol::SekmeKapat));
        assert_eq!(coz("t", true, true, false), Some(Kisayol::GeriAl));
        assert_eq!(coz("tab", true, false, false), Some(Kisayol::SonrakiSekme));
        assert_eq!(coz("tab", true, true, false), Some(Kisayol::OncekiSekme));
    }

    #[test]
    fn ctrl_dokuz_son_sekme_digerleri_sira() {
        assert_eq!(coz("9", true, false, false), Some(Kisayol::SonSekme));
        assert_eq!(coz("1", true, false, false), Some(Kisayol::SekmeNo(1)));
        assert_eq!(coz("8", true, false, false), Some(Kisayol::SekmeNo(8)));
        // 0 tabloda yok — Chrome'da da bir işi yok.
        assert_eq!(coz("0", true, false, false), None);
    }

    #[test]
    fn degistiricisiz_tuslar() {
        assert_eq!(coz("f5", false, false, false), Some(Kisayol::Yenile));
        assert_eq!(coz("f11", false, false, false), Some(Kisayol::TamEkran));
        assert_eq!(coz("escape", false, false, false), Some(Kisayol::Durdur));
    }

    #[test]
    fn alt_ok_tuslari_gezinme() {
        assert_eq!(coz("arrowleft", false, false, true), Some(Kisayol::Geri));
        assert_eq!(coz("arrowright", false, false, true), Some(Kisayol::Ileri));
        // Alt'sız ok tuşu sayfanın; kısayol değil.
        assert_eq!(coz("arrowleft", false, false, false), None);
    }

    #[test]
    fn ctrl_alt_kolu_dar() {
        assert_eq!(coz("z", true, false, true), Some(Kisayol::HepsiniUyut));
        assert_eq!(coz("q", true, false, true), None);
        assert_eq!(coz("t", true, false, true), None);
    }

    #[test]
    fn degistiricisiz_harf_kisayol_degil() {
        // Sayfaya yazarken her harf kısayol tetiklerse tarayıcı kullanılamaz.
        for h in ["t", "w", "l", "z", "a", "m"] {
            assert_eq!(coz(h, false, false, false), None, "{h}");
        }
    }

    #[test]
    fn buyuk_harf_ve_bosluk_onemsiz() {
        assert_eq!(coz("T", true, false, false), Some(Kisayol::YeniSekme));
        assert_eq!(
            coz(" Tab ", true, false, false),
            Some(Kisayol::SonrakiSekme)
        );
    }

    #[test]
    fn arayuz_isleri_ayrilmis() {
        assert!(Kisayol::AdresOdak.arayuz_isi());
        assert!(Kisayol::SekmeArama.arayuz_isi());
        assert!(Kisayol::BellekPaneli.arayuz_isi());
        assert!(Kisayol::TamEkran.arayuz_isi());
        assert!(!Kisayol::YeniSekme.arayuz_isi());
        assert!(!Kisayol::BuSekmeyiUyut.arayuz_isi());
        assert!(!Kisayol::HepsiniUyut.arayuz_isi());
    }

    #[test]
    fn vk_kodlari_dogru_ada_cevriliyor() {
        assert_eq!(vk_adi(0x54).as_deref(), Some("t")); // VK_T
        assert_eq!(vk_adi(0x5A).as_deref(), Some("z")); // VK_Z
        assert_eq!(vk_adi(0x31).as_deref(), Some("1"));
        assert_eq!(vk_adi(0x09).as_deref(), Some("tab"));
        assert_eq!(vk_adi(0x74).as_deref(), Some("f5"));
        assert_eq!(vk_adi(0x7A).as_deref(), Some("f11"));
        assert_eq!(vk_adi(0x25).as_deref(), Some("arrowleft"));
        assert_eq!(vk_adi(0x10), None); // VK_SHIFT — tek başına eylem değil
    }

    #[test]
    fn iki_kapi_ayni_sonucu_veriyor() {
        // Sayfa odaktayken gelen VK_T ile kabuk odaktayken gelen "t" aynı
        // eylemi üretmeli — tablonun tek kaynak olmasının kanıtı.
        let vk = vk_adi(0x54).unwrap();
        assert_eq!(coz(&vk, true, false, false), coz("t", true, false, false));
    }
}
