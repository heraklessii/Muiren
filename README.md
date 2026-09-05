# Muiren

**Çok sekmede az RAM harcayan masaüstü tarayıcı.** Onlarca sekme açık kalırken
oyunun ya da videonun payına düşen belleği geri veren, video oynatırken
takılmayan, teması ve arka planı baştan aşağı değiştirilebilen bir tarayıcı.

[![CI](https://github.com/heraklessii/Muiren/actions/workflows/ci.yml/badge.svg)](https://github.com/heraklessii/Muiren/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/durum-erken%20geli%C5%9Ftirme-orange.svg)](docs/Roadmap.md)

> ⚠️ **Bu proje erken geliştirme aşamasında.** Sekmeler, bellek politikası,
> geçmiş, temalar, engelleme ve köprüler yazıldı ve 431 otomatik testle
> doğrulanıyor (361 Rust + 70 arayüz). Ama **projenin asıl iddiası —
> "az RAM" — henüz ölçülmedi.** Ölçüm aracı hazır, rakam yok. Durum tablosu:
> [`docs/Roadmap.md`](docs/Roadmap.md).

**[muiren sayfası](https://heraklessii.github.io/Muiren/)** ·
**[sürümler](https://github.com/heraklessii/Muiren/releases)**

---

## Ne Yapar

- **Boştaki sekmeleri uyutur, sonra atar.** Bir sekme yeterince uzun boşta
  kalınca WebView2'nin `TrySuspend` çağrısıyla dondurulur; baskı sürerse
  webview'i tamamen yıkılır. Sekme çubuğundaki yeri, başlığı, adresi ve
  gezinme geçmişi durur — tıklandığında geri gelir.
- **Kararı sistem verir, kullanıcı değil.** Eşikler makineye göre ayarlanıyor:
  8 GB'lık bir dizüstü ile 32 GB'lık bir masaüstü aynı politikayı kaldırmıyor.
  Hiçbir eşik koda gömülü değil, hepsi ayarlardan geliyor.
- **Sabitlenmiş, ses çalan ve tam ekran sekmeye dokunulmaz.** Uyutma kararı
  bunları hesaba katıyor; müzik çalarken sekmenin susması bir hata olurdu.
- **Bellek paneli.** Sekme başına yaklaşık MB, uyandırma gecikmesi, WebView2
  süreç sayısı. **Uyuyan sekme ölçmek için uyandırılmaz** — son bilinen değer
  gösterilir; bir kez uyandırılsaydı projenin tek iddiası çökerdi.
- **Oyun modu.** Tam ekran bir oyun algılandığında tarayıcı kendi payını
  düşürüyor: arka plan sekmeleri daha erken uyuyor, animasyonlar duruyor.
- **Sekme ağacı, grupları ve dikey sekme çubuğu.** Gruba eşik verilebiliyor —
  "bu on sekme araştırma, acele uyusunlar".
- **Oturum geri yükleme.** Kapanan pencere sekmeleriyle birlikte geri geliyor;
  uyuyan sekmeler uyuyarak açılıyor, hepsi birden yüklenmiyor.
- **Tema paketi (`.muitema`).** Renk jetonları ve arka plan görseli
  değiştirilebiliyor. **Tema JavaScript çalıştırmaz** — çalıştırsaydı indirilen
  her tema dosyası uzaktan kod yürütmeye dönüşürdü. Paket doğrulanmadan
  uygulanmıyor.
- **Reklam ve pop-up engelleme.** Filtre listesi ayarlar yazıldığında bir kez
  derleniyor, her istekte değil. **Engelleme sessiz değil:** engellenen
  pop-up, yönlendirme ya da istek adres çubuğunda rozetle duyuruluyor — bazı
  siteler ödeme akışını pop-up'ta açıyor.
- **Gizli sekme.** Motorun InPrivate kipi çerezi ve önbelleği bellekte tutuyor;
  geçmiş, oturum dosyası ve favicon deposu ise bizim ve üçü de ayrıca
  kapatılıyor.
- **Geçmiş ve yer imleri.** SQLite, diskte, WAL. Arama Türkçe küçük harfe
  duyarlı ("İSTANBUL" araması "istanbul" ile eşleşiyor).
- **Mui ailesine devir.** İndirme [Muiget](https://github.com/heraklessii/Muiget)'e,
  yerel oynatma Muiply'a, birlikte izleme
  [Muiwatch](https://github.com/heraklessii/Muiwatch)'a, oyun modu
  [Muifly](https://github.com/heraklessii/Muifly)'a gidiyor. Köprüler
  **isteğe bağlı**: kardeş uygulama kurulu değilse özellik sessizce gizleniyor.
  Köprüden yalnız bir adres ya da dosya yolu geçiyor — çerez, oturum başlığı
  ve sayfa içeriği geçmiyor.
- **Telemetri yok, hesap yok, bulut yok.** Geçmiş, yer imleri ve oturum yalnız
  diskte duruyor.

## Ne Yapmaz

Bunlar bilinçli kararlar, eksik değil — gerekçeleri
[`docs/Roadmap.md`](docs/Roadmap.md) içinde.

- **Kendi render motoru yok.** Blink yazmıyoruz. Muiren bir *kabuk*: sekme
  yönetimi, bellek politikası, tema, köprüler. Sayfayı WebView2 çiziyor.
  Dolayısıyla **tek sekmede Chrome'dan az RAM yemez** — aynı motor.
  Kazanç, açık duran ama bakılmayan sekmelerden geliyor.
- **Chrome Web Store / uzantı ekosistemi yok.** WebView2 yalnız paketlenmemiş
  uzantı yükleyebiliyor; mağaza uyumluluğu vaat edilmiyor.
- **Şifre yöneticisi yok.** Kimlik bilgisi saklamıyoruz; işletim sistemine ve
  kullanıcının mevcut yöneticisine bırakılıyor.
- **Hesap senkronizasyonu yok.** Sunucu yok, hesap yok.
- **Kendi indirme motoru yok.** İndirme Muiget'in işi.
- **Mobil yok.** Masaüstü, Windows öncelikli.

## Kurulum

**Windows 10 / 11 (x64).** [Sürümler](https://github.com/heraklessii/Muiren/releases)
sayfasından:

- `.exe` — NSIS kurulumu (önerilen, daha küçük)
- `.msi` — kurumsal dağıtım için

Kurulum paketleri **imzasız**: Windows SmartScreen uyarı gösterecek. Kod
imzalama sertifikası henüz yok.

WebView2 Runtime Windows 11'de zaten kurulu. Windows 10'da yoksa kurulum
paketi indirip kuruyor (çevrimiçi bağlantı gerekiyor).

## Geliştirme

```bash
npm install
npm run tauri dev
```

| Komut | Ne yapıyor |
|---|---|
| `npm run tauri dev` | Tam uygulama, hot reload |
| `npm test` | Arayüz testleri (vitest) |
| `npm run tauri build` | Dağıtım paketi |
| `cd src-tauri && cargo test` | Rust testleri (motorlu) |
| `cd src-tauri && cargo test --no-default-features` | **Motorsuz** — karar veren modüllerin motora sızmadığını derleyiciyle kanıtlar |

Ayrıntı: [`docs/Setup.md`](docs/Setup.md).

## Mimari

Tek cümlelik özet: **karar veren kod backend'de, arayüz saf görüntü + IPC.**
Sebebi somut — "bu sekme uyusun" kararını bellek gözcüsü kendi iş parçacığında
veriyor ve o karar sekme çubuğunun çizilmiş olup olmamasından bağımsız olmalı.

| Konu | Dosya |
|---|---|
| Katmanlama, veri akışı, motor soyutlaması | [`docs/Architecture.md`](docs/Architecture.md) |
| RAM, uyutma/atma eşikleri, ölçüm — projenin kalbi | [`docs/Bellek.md`](docs/Bellek.md) |
| Sekme durum makinesi, ağaç, oturum | [`docs/Sekmeler.md`](docs/Sekmeler.md) |
| Video, donanım kod çözme, DRM | [`docs/Medya.md`](docs/Medya.md) |
| Tema paketi ve `.muitema` biçimi | [`docs/Temalar.md`](docs/Temalar.md) |
| Mui ailesi köprüleri | [`docs/Kopruler.md`](docs/Kopruler.md) |
| Geçmiş, yer imleri, SQLite şeması | [`docs/Depolama.md`](docs/Depolama.md) |
| Komut ve olay sözleşmesi | [`docs/IPC.md`](docs/IPC.md) |
| Arayüz, tasarım dili, kısayollar | [`docs/Frontend.md`](docs/Frontend.md) |
| Yol haritası ve kapsam dışı kararlar | [`docs/Roadmap.md`](docs/Roadmap.md) |

**Yığın:** Tauri v2 (`unstable` — bir pencerede çoklu webview) · Rust ·
WebView2 · SQLite (`rusqlite`, bundled) · React 19 + Vite + TypeScript.

## Mui Ailesi

[Muiget](https://github.com/heraklessii/Muiget) (indirme) ·
[Muivly](https://github.com/heraklessii/Muivly) (canlı duvar kağıdı) ·
[Muiwatch](https://github.com/heraklessii/Muiwatch) (birlikte izleme) ·
[Muifly](https://github.com/heraklessii/Muifly) (oyun performansı) ·
[MuiLabs](https://github.com/heraklessii/MuiLabs) (vitrin)

## Lisans

[Apache-2.0](LICENSE).
