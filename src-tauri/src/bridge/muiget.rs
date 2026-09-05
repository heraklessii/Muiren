//! Muiget köprüsü — indirme devri.
//!
//! Muiren'in **kendi indirme motoru yok** (CLAUDE.md, kapsam dışı; karar #4).
//! Motorun `DownloadStarting` olayı iptal ediliyor ve iş Muiget'e geçiyor.
//!
//! ## Protokol Muiget'ten devralındı, icat edilmedi
//!
//! Karşı taraf `Muiget/src-tauri/src/extension_bridge/native_host.rs` içinde
//! yazılı ve test edilmiş. Biçim: **4 bayt uzunluk öneki (yerel bayt sırası) +
//! UTF-8 JSON gövde**, `stdin`/`stdout` üzerinden.
//!
//! ```text
//! Muiren ──(stdio)──> muiget --native-host ──(argv)──> çalışan Muiget penceresi
//! ```
//!
//! ### Faz 3'te doğrulanan üç şey
//!
//! `docs/Kopruler.md` bu köprüde üç soru bırakmıştı; üçünün de cevabı Muiget'in
//! kaynağında duruyordu:
//!
//! 1. **Ayrı bir host ikilisi yok.** Host, Muiget'in kendisi: `muiget.exe
//!    --native-host`. `is_host_invocation` bu bayrağı tek başına köprü işareti
//!    sayıyor, dolayısıyla Muiren'in manifest yazmasına ya da bir uzantı
//!    kimliği taklit etmesine gerek yok.
//! 2. **Uzantı kimliği doğrulaması host sürecinde değil, tarayıcıda.** İzin
//!    listesi native messaging *manifestinde* (`allowed_origins`) duruyor ve
//!    onu tarayıcı uyguluyor. İkiliyi doğrudan çalıştıran bir çağıran o
//!    kapıdan hiç geçmiyor — yani Muiget tarafında **hiçbir değişiklik
//!    gerekmiyor**, karar kaydı da gerekmiyor.
//! 3. **Host durumsuz ve kısa ömürlü.** İsteği `muiget --add <base64>` olarak
//!    ana pencereye devredip çıkıyor. Bu yüzden burada kalıcı bir host süreci
//!    **tutulmuyor**: tutmanın kazancı yok, bedeli boşta duran bir süreç.
//!
//! ## Ne gönderiliyor, ne gönderilmiyor
//!
//! `DownloadRequest` beş alan taşıyabiliyor: `url`, `fileName`, `referrer`,
//! `cookies`, `userAgent`. Muiren **ilk üçünü** dolduruyor.
//!
//! - `cookies` ve `userAgent` **hiç gönderilmiyor** (`docs/Kopruler.md`, "Genel
//!   kural": hiçbir köprü çerez ya da oturum başlığı taşımıyor). Bedeli açık:
//!   giriş gerektiren bir indirme Muiget'te başarısız olabiliyor. Alternatifi,
//!   kullanıcının oturum çerezini süreçler arasında taşımaktı; bu takas
//!   yapılmıyor.
//! - `referrer` gönderiliyor çünkü **bir URL** ve genel kural URL'ye izin
//!   veriyor. Karşılığı somut: hotlink korumalı siteler (hedef kitlenin
//!   yarısı) `Referer` başlığı olmadan indirme vermiyor ve köprü işe yaramaz
//!   hâle geliyordu.
//! - `fileName` **sayfadan geliyor** ve [`super::dosya_adi_temizle`] geçmeden
//!   yola çıkmıyor (CLAUDE.md #8).

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{Bulunan, Kopru, KopruDurumu, Yuk};
use crate::hata::{MuirenHata, Sonuc};

/// Kurulum anahtarındaki ürün adı ve ikilinin adı.
const URUN: &str = "Muiget";
const IKILI: &str = "Muiget.exe";

/// Host kipini açan bayrak (`native_host.rs`, `HOST_FLAG`).
const HOST_BAYRAGI: &str = "--native-host";

/// Muiget'in kendi üst sınırı (`MAX_MESSAGE_SIZE`). Yanıtı okurken aynı sınır
/// **bizim tarafta da** uygulanıyor: bozuk bir uzunluk önekine güvenip
/// `Vec::with_capacity` çağırmak gigabaytlarca bellek ayırtabilirdi.
const AZAMI_MESAJ: usize = 1024 * 1024;

/// Yanıt beklemenin üst sınırı.
///
/// Host durumsuz ve tek iş yapıyor; bu kadar sürüyorsa bir şey ters gitmiş
/// demektir. Süre dolduğunda **hata dönüyoruz**: indirmenin gidip gitmediğini
/// bilmeden "gönderildi" demek, kullanıcının dosyayı beklemesine yol açıyor.
const YANIT_SURESI: Duration = Duration::from_secs(5);

/// Muiget'e giden mesaj. Alan adları `native_host.rs` içindeki
/// `ExtensionMessage` / `DownloadRequest` ile **birebir aynı**.
///
/// `tag = "type"` + `rename_all = "camelCase"`: karşı taraf içten etiketli bir
/// enum bekliyor, yani gövde `{"type":"download","url":…}` biçiminde düz
/// geliyor.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Istek {
    Download(IndirmeIstegi),
    /// Köprünün ayakta olduğunu ölçmek için. Devretmeden önce
    /// gönderilmiyor — fazladan bir tur, kazancı yok.
    #[allow(dead_code)]
    Ping,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct IndirmeIstegi {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    referrer: Option<String>,
    // `cookies` ve `userAgent` alanları BİLEREK yok: gönderilmedikleri için
    // yapıda da durmuyorlar. Alan burada dursaydı "bir gün doldururuz"
    // sessiz bir davete dönüşürdü (`docs/Kopruler.md`).
}

/// Muiget'ten gelen yanıt (`native_host.rs`, `HostResponse`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Yanit {
    Accepted { url: String },
    Rejected { reason: String },
    Pong { version: String },
}

pub struct Muiget {
    bulunan: Bulunan,
}

impl Muiget {
    pub fn yeni() -> Self {
        Muiget {
            bulunan: Bulunan::yeni(),
        }
    }

    pub fn yol(&self) -> Option<PathBuf> {
        self.bulunan.yol(|| super::uygulama_ara(URUN, IKILI))
    }

    pub fn unut(&self) {
        self.bulunan.unut();
    }

    pub fn durum(&self) -> KopruDurumu {
        let yol = self.yol();
        KopruDurumu {
            ad: URUN,
            kurulu: yol.is_some(),
            yol: yol.map(|y| y.to_string_lossy().into_owned()),
        }
    }

    /// İndirmeyi devreder.
    ///
    /// `dosya_adi` **temizlenmiş** gelmeli; çağıran ([`Kopru::devret`]) bunu
    /// zorluyor.
    fn gonder(&self, istek: Istek) -> Sonuc<Yanit> {
        let ikili = self
            .yol()
            .ok_or_else(|| MuirenHata::KopruYok(URUN.into()))?;

        let mut cocuk = Command::new(&ikili)
            .arg(HOST_BAYRAGI)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // `stderr` yutuluyor: host'un günlük satırları Muiren'in
            // konsolunu kirletmemeli.
            .stderr(Stdio::null())
            .spawn()?;

        let govde = serde_json::to_vec(&istek)?;
        {
            let stdin = cocuk
                .stdin
                .as_mut()
                .ok_or_else(|| MuirenHata::Kopru("stdin açılamadı".into()))?;
            // Uzunluk **yerel** bayt sırasında: Chrome'un belgelenmiş
            // davranışı ve Muiget `from_ne_bytes` ile okuyor.
            stdin.write_all(&(govde.len() as u32).to_ne_bytes())?;
            stdin.write_all(&govde)?;
            stdin.flush()?;
        }
        // stdin KAPANIYOR: host bir sonraki mesajı beklemeden çıksın
        // (`read_message` temiz dosya sonunda `Ok(None)` dönüyor).
        drop(cocuk.stdin.take());

        let stdout = cocuk
            .stdout
            .take()
            .ok_or_else(|| MuirenHata::Kopru("stdout açılamadı".into()))?;

        // Okuma ayrı bir iş parçacığında: `Child::stdout` üzerinde zaman aşımı
        // yok ve asılı kalan bir host, devretme çağrısını süresiz bloke
        // ederdi. Kanal, iş parçacığını beklemeden vazgeçmeyi sağlıyor.
        let (gonderen, alan) = mpsc::channel();
        std::thread::Builder::new()
            .name("muiren-muiget".into())
            .spawn(move || {
                let _ = gonderen.send(yaniti_oku(stdout));
            })
            .map_err(|e| MuirenHata::Kopru(e.to_string()))?;

        let sonuc = match alan.recv_timeout(YANIT_SURESI) {
            Ok(y) => y,
            Err(_) => {
                // Süre doldu: host'u öldürüp hata dönüyoruz. Öldürmezsek
                // yanıt vermeyen bir süreç arkada kalıyor.
                let _ = cocuk.kill();
                return Err(MuirenHata::Kopru("yanıt gelmedi".into()));
            }
        };
        // Host durumsuz ve mesajdan sonra çıkıyor; yine de beklemeden
        // bırakmıyoruz ki zombi süreç kalmasın.
        let _ = cocuk.wait();
        sonuc
    }
}

/// Uzunluk önekli tek bir yanıt okur.
///
/// Önek `read_exact` ile değil elle okunuyor — Muiget'in kendi gerekçesiyle
/// aynı: "hiç bayt gelmedi" (host konuşmadan çıktı) ile "önek yarım kaldı"
/// (kanal koptu) farklı şeyler ve `read_exact` ikisine de aynı hatayı veriyor.
fn yaniti_oku<R: Read>(mut kaynak: R) -> Sonuc<Yanit> {
    let mut onek = [0u8; 4];
    let mut okunan = 0;
    while okunan < onek.len() {
        match kaynak.read(&mut onek[okunan..]) {
            Ok(0) if okunan == 0 => {
                return Err(MuirenHata::Kopru("yanıtsız kapandı".into()));
            }
            Ok(0) => return Err(MuirenHata::Kopru("uzunluk öneki yarım".into())),
            Ok(n) => okunan += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(MuirenHata::from(e)),
        }
    }

    let uzunluk = u32::from_ne_bytes(onek) as usize;
    if uzunluk == 0 || uzunluk > AZAMI_MESAJ {
        return Err(MuirenHata::Kopru(format!("geçersiz uzunluk: {uzunluk}")));
    }

    let mut govde = vec![0u8; uzunluk];
    kaynak.read_exact(&mut govde)?;
    serde_json::from_slice(&govde).map_err(MuirenHata::from)
}

impl Kopru for Muiget {
    fn ad(&self) -> &'static str {
        URUN
    }

    fn kurulu(&self) -> bool {
        self.yol().is_some()
    }

    fn devret(&self, yuk: Yuk) -> Sonuc<()> {
        let Yuk::Indirme {
            url,
            dosya_adi,
            kaynak_sayfa,
        } = yuk
        else {
            return Err(MuirenHata::Kopru("Muiget yalnız indirme alıyor".into()));
        };

        // Muiget'in kendi `is_supported` kontrolüyle aynı sınır. İki kat
        // kontrol bilinçli: `file://` ve `data:` bir indirme yöneticisine
        // gitmiyor ve bunu göndermeden önce burada da biliyoruz.
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(MuirenHata::Kopru("yalnız http(s) devrediliyor".into()));
        }

        let istek = Istek::Download(IndirmeIstegi {
            url: url.clone(),
            // Ad zaten temizlenmiş geliyor; burada ikinci kez geçirmek
            // ucuz bir güvence (çağıran yolu değişirse sınır düşmesin).
            file_name: dosya_adi.as_deref().and_then(super::dosya_adi_temizle),
            referrer: kaynak_sayfa,
        });

        match self.gonder(istek)? {
            Yanit::Accepted { .. } => Ok(()),
            Yanit::Rejected { reason } => Err(MuirenHata::Kopru(reason)),
            // `Pong` bir indirme isteğine yanıt değil: protokol ayrışmış
            // demektir ve sessizce başarı saymak yanlış olurdu.
            Yanit::Pong { version } => Err(MuirenHata::Kopru(format!(
                "beklenmeyen yanıt (sürüm {version})"
            ))),
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    /// Gövde biçimi Muiget'in `ExtensionMessage` tanımıyla eşleşmek zorunda.
    /// Ayrışırsa hata çalışma zamanında ve sessiz: host `Rejected` bile
    /// dönmüyor, JSON'u ayrıştıramayıp kanalı kapatıyor.
    #[test]
    fn indirme_mesaji_muigetin_bekledigi_bicimde() {
        let istek = Istek::Download(IndirmeIstegi {
            url: "https://ornek.com/a.zip".into(),
            file_name: Some("a.zip".into()),
            referrer: Some("https://ornek.com/sayfa".into()),
        });
        let json: serde_json::Value =
            serde_json::from_slice(&serde_json::to_vec(&istek).unwrap()).unwrap();

        assert_eq!(json["type"], "download");
        assert_eq!(json["url"], "https://ornek.com/a.zip");
        assert_eq!(json["fileName"], "a.zip");
        assert_eq!(json["referrer"], "https://ornek.com/sayfa");
    }

    #[test]
    fn cerez_ve_ua_alanlari_hic_gonderilmiyor() {
        let istek = Istek::Download(IndirmeIstegi {
            url: "https://ornek.com/a.zip".into(),
            ..Default::default()
        });
        let json: serde_json::Value =
            serde_json::from_slice(&serde_json::to_vec(&istek).unwrap()).unwrap();

        assert!(json.get("cookies").is_none(), "çerez gönderiliyor");
        assert!(json.get("userAgent").is_none(), "oturum başlığı gönderiliyor");
        // Boş `fileName` de gönderilmiyor: Muiget `Option` bekliyor ve
        // `null` göndermek adı "bilmiyoruz" değil "yok" yapardı.
        assert!(json.get("fileName").is_none());
    }

    #[test]
    fn yanit_cozumleniyor() {
        let cerceve = |govde: &str| {
            let mut v = (govde.len() as u32).to_ne_bytes().to_vec();
            v.extend_from_slice(govde.as_bytes());
            v
        };

        let kabul = cerceve(r#"{"type":"accepted","url":"https://a.example/x"}"#);
        assert!(matches!(
            yaniti_oku(&kabul[..]).unwrap(),
            Yanit::Accepted { .. }
        ));

        let ret = cerceve(r#"{"type":"rejected","reason":"desteklenmeyen şema"}"#);
        assert!(matches!(yaniti_oku(&ret[..]).unwrap(), Yanit::Rejected { .. }));
    }

    #[test]
    fn bos_yanit_hata() {
        assert!(yaniti_oku(&[][..]).is_err(), "yanıtsız kapanış hata değil");
    }

    #[test]
    fn devasa_uzunluk_oneki_reddediliyor() {
        // Bozuk (ya da kötü niyetli) bir önek gigabaytlarca bellek
        // ayırtabilirdi; sınır Muiget'inkiyle aynı.
        let mut veri = (u32::MAX).to_ne_bytes().to_vec();
        veri.extend_from_slice(b"{}");
        assert!(yaniti_oku(&veri[..]).is_err());
    }

    #[test]
    fn yarim_onek_hata() {
        assert!(yaniti_oku(&[1u8, 0][..]).is_err());
    }

    #[test]
    fn http_disi_sema_devredilmiyor() {
        let k = Muiget::yeni();
        let sonuc = k.devret(Yuk::Indirme {
            url: "file:///C:/gizli.txt".into(),
            dosya_adi: None,
            kaynak_sayfa: None,
        });
        assert!(sonuc.is_err(), "file:// şeması Muiget'e gitti");
    }
}
