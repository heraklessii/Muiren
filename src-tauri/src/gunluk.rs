//! Günlük — bağımlılıkların **yuttuğu** hataları görünür kılan ince katman.
//!
//! Buranın varlık sebebi somut bir olay: `tauri-runtime-wry` içindeki
//! `Message::CreateWebview` işleyicisi, wry'nin webview yaratma hatasını
//! `log::error!` ile yazıp çağırana yine de `Ok` döndürüyor. `log` crate'i bir
//! uygulama kurulmadığında hiçbir yere yazmıyor. Sonuç, Faz 1'i bloke eden
//! tablo oldu: `add_child` `Ok` diyor, sekme kaydı oluşuyor, webview yok ve
//! ortada tek satır hata mesajı yok.
//!
//! Bir daha aynı yere düşmemek için burada bir `log::Log` uygulaması var.
//! Kasıtlı olarak ince: dosyaya yazmıyor, döndürmüyor, sayfaya bir kanal
//! açmıyor (`docs/IPC.md`). Muiren'in kendi mesajları `eprintln!` ile
//! gidiyor; buranın işi **başkasının** mesajlarını yakalamak.
//!
//! Varsayılan eşik `warn`: sessiz ama hata yutmayan. `MUIREN_LOG` ortam
//! değişkeniyle (`error` / `warn` / `info` / `debug` / `trace` / `off`)
//! değiştirilebiliyor.

use std::io::Write;

struct Gunlukcu {
    seviye: log::LevelFilter,
}

impl log::Log for Gunlukcu {
    fn enabled(&self, veri: &log::Metadata) -> bool {
        veri.level() <= self.seviye
    }

    fn log(&self, kayit: &log::Record) {
        if !self.enabled(kayit.metadata()) {
            return;
        }
        // `stderr` çünkü `stdout` bazı kabuklarda tamponlanıyor ve çökme
        // anında yazılmamış kalıyor — tam da okumak istediğimiz an.
        let mut cikti = std::io::stderr().lock();
        let _ = writeln!(
            cikti,
            "[{} {}] {}",
            kayit.level(),
            kayit.target(),
            kayit.args()
        );
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}

/// `MUIREN_LOG` değerini eşiğe çevirir; tanınmayan değer varsayılana düşer.
///
/// Karşılaştırma ASCII: bunlar kullanıcı metni değil, sabit anahtar sözcükler
/// (CLAUDE.md #10 Türkçe **kullanıcı** metni için).
fn seviye_oku(ham: Option<&str>) -> log::LevelFilter {
    match ham.map(str::trim) {
        Some(s) if s.eq_ignore_ascii_case("off") => log::LevelFilter::Off,
        Some(s) if s.eq_ignore_ascii_case("error") => log::LevelFilter::Error,
        Some(s) if s.eq_ignore_ascii_case("warn") => log::LevelFilter::Warn,
        Some(s) if s.eq_ignore_ascii_case("info") => log::LevelFilter::Info,
        Some(s) if s.eq_ignore_ascii_case("debug") => log::LevelFilter::Debug,
        Some(s) if s.eq_ignore_ascii_case("trace") => log::LevelFilter::Trace,
        _ => log::LevelFilter::Warn,
    }
}

/// Günlükçüyü kurar. İkinci çağrı sessizce yok sayılıyor.
pub fn kur() {
    let seviye = seviye_oku(std::env::var("MUIREN_LOG").ok().as_deref());
    // `Box::leak`: `set_logger` `&'static` istiyor ve günlükçü zaten sürecin
    // ömrü boyunca yaşıyor.
    let gunlukcu: &'static Gunlukcu = Box::leak(Box::new(Gunlukcu { seviye }));
    if log::set_logger(gunlukcu).is_ok() {
        log::set_max_level(seviye);
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn tanimsiz_ve_bozuk_deger_varsayilana_duser() {
        assert_eq!(seviye_oku(None), log::LevelFilter::Warn);
        assert_eq!(seviye_oku(Some("gurultu")), log::LevelFilter::Warn);
        assert_eq!(seviye_oku(Some("")), log::LevelFilter::Warn);
    }

    #[test]
    fn buyuk_kucuk_harf_ve_bosluk_onemsiz() {
        assert_eq!(seviye_oku(Some(" DEBUG ")), log::LevelFilter::Debug);
        assert_eq!(seviye_oku(Some("Error")), log::LevelFilter::Error);
        assert_eq!(seviye_oku(Some("off")), log::LevelFilter::Off);
    }
}
