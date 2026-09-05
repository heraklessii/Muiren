//! Ölçüm koşucusu — planı uygulayan iş parçacığı.
//!
//! Saf olan her şey [`super::plan`] ve [`super::rapor`] içinde; burada
//! yalnız sıra var: aç, bekle, ölç, yaz. `memory/gozcu.rs` ile aynı kalıp —
//! döngü burada, karar başka yerde.
//!
//! **Gözcü kapatılmıyor.** `docs/olcumler/README.md` bunu açıkça şart
//! koşuyor: "Sekmeler açılırken bellek gözcüsü çalışmaya devam ediyor.
//! Ölçtüğümüz şey politikanın çalıştığı hâl." Bu yüzden koşucu sekmeleri
//! açıyor ve bekliyor, politikaya hiç dokunmuyor.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::Runtime;

use super::plan::{self, Asama, Plan};
use super::rapor::{self, Ornek, Ortam};
use crate::memory::{gozcu, olcum};
use crate::tabs::surucu::Surucu;

/// Planı okur ve koşuyu başlatır.
///
/// Hata durumunda **koşu hiç başlamıyor** ve sebep yazılıyor: yarım koşan bir
/// ölçüm, koşmayan bir ölçümden kötü — raporu var ama sayısı yok.
pub fn baslat<R: Runtime>(surucu: &Arc<Surucu<R>>, yol: PathBuf) {
    let p = match oku(&yol) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("muiren: ölçüm planı okunamadı ({}): {e}", yol.display());
            return;
        }
    };
    if let Err(e) = plan::dogrula(&p) {
        eprintln!("muiren: ölçüm planı geçersiz: {e}");
        return;
    }

    let cikti = cikti_yolu(&p, &yol);
    let toplam = plan::toplam_sn(&p);
    eprintln!(
        "muiren: ölçüm başlıyor — {} adres, yaklaşık {} dk. Rapor: {}",
        p.adresler.len(),
        toplam / 60 + 1,
        cikti.display()
    );
    eprintln!("muiren: ölçüm sürerken makineye dokunmayın (protokol).");

    let zayif = Arc::downgrade(surucu);
    let sonuc = std::thread::Builder::new()
        .name("muiren-olcum".into())
        .spawn(move || {
            let Some(s) = zayif.upgrade() else { return };
            kos(&s, p, cikti);
        });
    if let Err(e) = sonuc {
        eprintln!("muiren: ölçüm iş parçacığı başlatılamadı: {e}");
    }
}

fn oku(yol: &Path) -> Result<Plan, String> {
    let veri = std::fs::read(yol).map_err(|e| e.to_string())?;
    let ham: Plan =
        serde_json::from_slice(crate::bom_kirp(&veri)).map_err(|e| e.to_string())?;
    Ok(plan::duzelt(ham))
}

/// Raporun yazılacağı dosya.
///
/// Plan `cikti` yazmadıysa **planın yanına** yazılıyor, çalışma dizinine
/// değil: `npm run tauri dev` ile koşarken çalışma dizini `src-tauri` oluyor
/// ve raporun oraya düşmesi, protokolün istediği `docs/olcumler/` yerine
/// kaybolması demekti.
fn cikti_yolu(p: &Plan, plan_yolu: &Path) -> PathBuf {
    if !p.cikti.is_empty() {
        return PathBuf::from(&p.cikti);
    }
    let dizin = plan_yolu.parent().unwrap_or(Path::new("."));
    dizin.join(format!("{}-{}.md", rapor::tarih(simdi()), p.ad))
}

fn simdi() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn kos<R: Runtime>(surucu: &Arc<Surucu<R>>, p: Plan, cikti: PathBuf) {
    let mut notlar: Vec<String> = Vec::new();
    let mut ornekler: Vec<Ornek> = Vec::new();

    // 1. Boş tarayıcı. Protokol: sıfır sekme, yeni sekme sayfası, bekle.
    //    Açık bir sekme varsa taban maliyet ölçülemez ve bu **not düşülüyor**,
    //    sekmeler kapatılmıyor: kullanıcının açık sekmelerini bir ölçüm
    //    aracının kapatması kabul edilemez.
    let acik = surucu.sekme_listesi().len();
    if acik > 0 {
        notlar.push(format!(
            "Koşu başlarken {acik} sekme açıktı: \"Boş tarayıcı\" satırı taban maliyeti değil."
        ));
    }
    bekle(p.yerlesme_sn);
    ornekler.push(olc(surucu, Asama::Bos, acik == 0));

    // 2. Sekmeleri aç. Arka planda: elli sekmeyi teker teker öne almak, her
    //    açılışta bir `Resume`/gizle turu demekti ve ölçülen şey o tur olurdu.
    let mut acilan = 0u32;
    for adres in &p.adresler {
        match surucu.sekme_ac(Some(adres.clone()), None, true, false) {
            Ok(_) => acilan += 1,
            Err(e) => notlar.push(format!("Açılamadı: {adres} ({e})")),
        }
        std::thread::sleep(Duration::from_millis(p.acilis_araligi_ms));
    }
    eprintln!("muiren: {acilan} sekme açıldı, {} sn yerleşme.", p.yerlesme_sn);

    bekle(p.yerlesme_sn);
    ornekler.push(olc(surucu, Asama::Acildi, true));

    // 3. Boşta bekleme — uyutmanın işini yaptığı yer.
    if p.bosta_sn > 0 {
        eprintln!("muiren: {} dk boşta bekleme başladı.", p.bosta_sn / 60);
        bekle(p.bosta_sn);
        ornekler.push(olc(surucu, Asama::Bosta, true));
    } else {
        notlar.push("Boşta bekleme atlandı (`bostaSn: 0`).".into());
    }

    let metin = rapor::olustur(&p, &ortam(surucu), &ornekler, &notlar);
    match yaz(&cikti, &metin) {
        Ok(()) => eprintln!("muiren: ölçüm bitti, rapor yazıldı: {}", cikti.display()),
        Err(e) => {
            // Rapor yazılamadıysa sayılar **kaybolmuyor**: 15 dakika bekleyen
            // bir koşunun sonucunu bir dosya izni yüzünden çöpe atmak olurdu.
            eprintln!("muiren: rapor yazılamadı ({}): {e}\n{metin}", cikti.display());
        }
    }

    if p.kapat_bitince {
        // Oturum dosyası kapanışta yazılıyor (`RunEvent::Exit`), yani ölçümün
        // açtığı elli sekme bir sonraki açılışta geri geliyor. Bu bilinçli:
        // ölçüm aracının kullanıcının oturumunu sessizce silmesi, kazandığı
        // zamandan çok daha pahalı olurdu.
        eprintln!("muiren: plan `kapatBitince` diyor, kapanılıyor.");
        surucu.app().exit(0);
    }
}

/// Bekleme, saniyelik parçalar hâlinde.
///
/// Tek bir uzun `sleep` yerine parçalı: uygulama kapanırken 15 dakika
/// bekleyen bir iş parçacığı, kapanmayı o kadar geciktirmemeli.
fn bekle(saniye: u64) {
    for _ in 0..saniye {
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// Bir aşamanın ölçümü.
///
/// Gözcünün bir turu **elle** koşturuluyor, son yayınlanan özet beklenmiyor:
/// panel özeti `gozcu_periyodu_sn` kadar eski olabilir ve protokolün istediği
/// şey "şu anda" değeri.
fn olc<R: Runtime>(surucu: &Arc<Surucu<R>>, asama: Asama, gecerli: bool) -> Ornek {
    gozcu::tur(surucu);
    Ornek {
        asama,
        ozet: surucu.bellek_ozeti(),
        gecerli,
    }
}

fn ortam<R: Runtime>(surucu: &Arc<Surucu<R>>) -> Ortam {
    let ayarlar = surucu.ayarlar();
    let sistem = olcum::sistem();
    let cekirdek = std::thread::available_parallelism()
        .map(|n| n.get().to_string())
        .unwrap_or_else(|_| "?".into());

    Ortam {
        tarih: rapor::tarih(simdi()),
        muiren: env!("CARGO_PKG_VERSION").to_string(),
        isletim: format!(
            "{} · {cekirdek} mantıksal çekirdek · {} GB RAM",
            std::env::consts::OS,
            sistem.toplam_mb / 1024
        ),
        profil: format!("{:?}", ayarlar.profil),
        surec_politikasi: match ayarlar.surec_politikasi {
            crate::settings::SurecPolitikasi::Birlesik => "process-per-site açık".into(),
            crate::settings::SurecPolitikasi::Varsayilan => "Chromium varsayılanı".into(),
        },
        webview2: surucu.calisma_zamani_surumu(),
    }
}

fn yaz(yol: &Path, metin: &str) -> std::io::Result<()> {
    if let Some(dizin) = yol.parent() {
        if !dizin.as_os_str().is_empty() {
            std::fs::create_dir_all(dizin)?;
        }
    }
    std::fs::write(yol, metin)
}
