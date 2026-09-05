# Kurulum ve Geliştirme

## Gereksinimler

| Şey | Sürüm | Not |
|---|---|---|
| Rust | kararlı, `x86_64-pc-windows-msvc` | `rustup default stable` |
| Visual Studio Build Tools | 2022 | "Desktop development with C++" iş yükü |
| Node.js | 20+ | |
| WebView2 Runtime | Evergreen | **Windows 11'de zaten kurulu** |
| WebView2 SDK | NuGet paketi | `webview2-com-sys` build script'i çekiyor |

Windows 10 kullanıcılarında WebView2 Runtime olmayabiliyor. Kurulum paketi
Evergreen Bootstrapper'ı içeriyor (Tauri'nin `webviewInstallMode` ayarı) —
dağıtım boyutuna ~2 MB ekliyor, çevrimdışı kurulum senaryosunda çalışmıyor;
kabul edilen bir sınır.

## İlk kurulum

```bash
npm install
cd src-tauri && cargo fetch
```

## Komutlar

| Komut | Ne yapıyor |
|---|---|
| `npm run tauri dev` | Tam uygulama, hot reload |
| `npm run dev` | Sadece arayüz (Vite, localhost:1420) — pencere boş kalır |
| `npm test` | vitest |
| `npm run tauri build` | Dağıtım paketi |
| `cd src-tauri && cargo test` | Rust testleri (motorlu) |
| `cd src-tauri && cargo test --no-default-features` | **Motorsuz** — CI'nin koştuğu |
| `cd src-tauri && cargo clippy -- -D warnings` | |

> Kardeş projelerde olduğu gibi Tauri geliştirmede `dist/` yerine
> `devUrl`den (localhost:1420) yüklüyor. Vite çalışmıyorken pencere boş kalır —
> bu bir hata değil.

### Port 1420 aile içinde ortak

Muiget, Muiply ve Muiren aynı `devUrl`i kullanıyor. İki Mui projesi aynı anda
geliştirme modundayken ikincisi ya "Port 1420 is already in use" diyor ya da
—daha kötüsü— **öteki projenin arayüzünü yüklüyor**: pencerenin başlığında
Muiren yazarken içinde Muiply açılıyor. Bir kere yaşandığında beş dakika
kaybettiren, sebebi hiç akla gelmeyen bir durum.

Çalışan sunucuyu bulmak için:

```bash
powershell "Get-NetTCPConnection -LocalPort 1420 -State Listen | ForEach-Object { (Get-Process -Id $_.OwningProcess).Path }"
```

Öteki projeyi kapatmadan geçici bir portla koşmak için:

```bash
npx vite --port 1431 --strictPort
```

```bash
npx tauri dev --config '{"build":{"devUrl":"http://localhost:1431","beforeDevCommand":""}}'
```

Bu yalnız o koşuya özel; `tauri.conf.json` değişmiyor. **Dikkat:** `--config`
ile üretilen ikili dosyada `devUrl` gömülü kalıyor, sonrasında `cargo run`
demek 1420'ye geri dönmek demek.

## Cargo özellikleri

```toml
[features]
default = ["motor"]
motor = ["dep:webview2-com", "dep:windows"]
```

`--no-default-features` ile `motor/yok.rs` devreye giriyor. Bu derleme:

- Windows dışında da geçiyor (CI'da Linux runner yeter),
- `tabs/`, `memory/`, `history/`, `theme/` modüllerinin motora sızmadığını
  derleyiciyle kanıtlıyor.

> **Kural (CLAUDE.md #4):** `motor/gercek.rs` içine eklenen her metot
> `motor/yok.rs` içinde de olmalı. Bu iki dosya birbirinden ayrışırsa
> `--no-default-features` kırılıyor ve genelde en kötü anda fark ediliyor.

## WebView2'ye ham erişim

Bellek politikasının ihtiyaç duyduğu `TrySuspend` / `Resume` ve bellek hedefi
çağrıları Tauri'nin sardığı API'de yok. Ham `ICoreWebView2` şöyle alınıyor:

```rust
// motor/gercek.rs — şema; gerçek imzalar Tauri sürümüne göre değişebilir
webview.with_webview(|w| unsafe {
    let core: ICoreWebView2 = w.controller().CoreWebView2()?;
    // ICoreWebView2_3: TrySuspend / Resume
    let v3: ICoreWebView2_3 = core.cast()?;
    v3.TrySuspend(&handler)?;
})?;
```

`unsafe` bloklar **yalnız bu dosyada**. Başka hiçbir modülde `unsafe` yok;
bir gün olursa mimari bozulmuş demektir.

Arayüz sürümleri (`ICoreWebView2_3`, `_7`, `_19`…) kullanıcının kurulu
Runtime sürümüne bağlı. `cast()` başarısız olabilir — bu bir hata değil, bir
**yetenek yokluğu**. `motor` bunu `Yetenekler` yapısında tutuyor ve bellek
politikası eksik yeteneğe göre geri çekiliyor:

| Yetenek yoksa | Politikanın cevabı |
|---|---|
| `TrySuspend` yok | `Uyuyan` durumu atlanıyor, doğrudan `Atilmis` kullanılıyor |
| Bellek hedefi yok | `HedefDusur` eylemi üretilmiyor |
| Süreç bilgisi yok | `BellekOzeti.olcum_yaklasik = true` |

Yani eski Runtime'da tarayıcı çalışmaya devam ediyor, sadece daha kaba
davranıyor. Çökmüyor.

## `EK_BAYRAKLAR` — Chromium anahtarları

WebView2 ortamı kurulurken `AdditionalBrowserArguments` ile geçirilen anahtar
listesi. **Tek yerde**, `motor/gercek.rs` içinde bir sabit ve her satırın
yanında gerekçesi var:

```rust
// motor/gercek.rs
const EK_BAYRAKLAR: &[&str] = &[
    // Aynı sitenin sekmeleri tek render sürecini paylaşsın. Süreç başına
    // 30-60 MB taban maliyet var; aynı siteden 10 sekme açan kullanıcı
    // varsayılan modelde yarım GB'ı boşa harcıyor. Siteler ARASI izolasyon
    // korunuyor — bkz. docs/Bellek.md.
    "--process-per-site",

    // Toplam renderer tavanı. Değeri settings.surec_politikasi'ndan geliyor,
    // burada sabit değil (CLAUDE.md #6).
    // "--renderer-process-limit=N"
];
```

Kurallar:

- **Bayrak eklemek bir karar.** Gerekçesiz satır birleştirilmiyor; gerekçe
  yorumda ve gerekiyorsa `docs/Roadmap.md` karar kaydında.
- **`--disable-site-isolation-trials` ve türevleri yasak.** Çok kazandırır,
  Spectre savunmasını söker. `docs/Bellek.md` içinde gerekçesi yazılı.
- **GPU hızlandırmayı kapatan bayrak yasak** — video takılmasının bir numaralı
  sebebi (`docs/Medya.md`).
- WebView2 bazı anahtarları sessizce yok sayıyor. Eklenen her bayrak, etkisi
  **ölçülerek** doğrulanıyor: aynı siteden 10 sekme aç, süreç say. Ölçüm
  yapılmadan "eklendi" denmiyor (Faz 0/R4).

Süreç sayımı için:

```bash
powershell "Get-Process msedgewebview2 | Measure-Object | Select-Object Count"
```

## Çoklu webview ve `unstable`

Tauri v2'de bir pencerede birden çok webview `unstable` özelliği arkasında:

```toml
tauri = { version = "2", features = ["unstable"] }
```

Bunun anlamı: **bu API minor sürümlerde değişebilir.** Tauri yükseltmesi
yaparken `motor/gercek.rs` ilk bakılacak dosya. Soyutlamanın (`docs/Architecture.md`)
bir sebebi de bu — kırılma tek dosyada kalıyor.

### Bütün webview'ler aynı bayrak dizesini almak zorunda

Bu, projenin en pahalı hata ayıklamasına mal olan kural (`docs/Roadmap.md`,
Faz 1 → "ÇÖZÜLDÜ"):

> wry **her webview için ayrı bir** `CreateCoreWebView2EnvironmentWithOptions`
> çağırıyor. WebView2 aynı kullanıcı veri klasörünü paylaşan ikinci ortamı
> yalnız **seçenekleri birebir aynıysa** yaratıyor; ayrışırsa
> `ERROR_INVALID_STATE` (0x8007139F) dönüyor.

Yani "ortam ilk webview'de kuruluyor, gerisine vermeye gerek yok" **yanlış**.
Bir webview'e `additional_browser_args` verip diğerine vermemek, ikincisinin
hiç doğmaması demek — üstelik sessizce, çünkü `tauri-runtime-wry` wry'nin
hatasını `log::error!` ile yazıp `add_child`e yine de `Ok` döndürüyor.

Uygulamadaki karşılığı: dize `lib.rs` içinde açılışta bir kez hesaplanıyor,
`Kurulu::yeni`ye veriliyor ve motor onu donduruyor. Yeni bir webview türü
eklenirse (gizli pencere, önizleme) o da **aynı dizeyi** almalı.

Bayrak veren ayarların "yeniden başlatma gerektiriyor" olmasının sebebi de bu:
çalışırken değiştirilseydi, o andan sonra açılan sekmeler kabuktan farklı bir
ortam ister ve hiç açılmazdı.

### Yutulan hataları görmek

`src-tauri/src/gunluk.rs` ince bir `log::Log` uygulaması kuruyor. Muiren'in
kendi mesajları `eprintln!` ile gidiyor; buranın işi **bağımlılıkların**
`log::error!` satırlarını yakalamak — yukarıdaki hata tam da orada
kayboluyordu.

```bash
MUIREN_LOG=debug npm run tauri dev    # error | warn (varsayılan) | info | debug | trace | off
```

## Dizin düzeni

```
Muiren/
├── CLAUDE.md
├── docs/
│   ├── Architecture.md   Bellek.md    Sekmeler.md   Medya.md
│   ├── Temalar.md        Kopruler.md  Depolama.md
│   ├── IPC.md            Frontend.md  Setup.md      Roadmap.md
│   └── olcumler/            ← Faz 2'den itibaren ham ölçüm çıktıları
├── public/icons/muiren.svg
├── src/                     ← React
│   ├── ipc/       index.ts  tipler.ts
│   ├── hooks/     useSekmeler.ts  useYetenekler.ts  useIcerikAlani.ts
  │               useBellek.ts    useOneriler.ts    useYerImi.ts
│   ├── lib/       url.ts  suz.ts  bicim.ts (+ .test.ts)
│   ├── components/
│   ├── assets/fonts/        ← gömülü Outfit
│   ├── styles.css
│   └── App.tsx  main.tsx
└── src-tauri/
    ├── icons/kaynak.svg
    ├── capabilities/varsayilan.json
    ├── Cargo.toml
    └── src/
        ├── main.rs  lib.rs  hata.rs  olaylar.rs  gunluk.rs
        ├── kisayol.rs            ← kısayol tablosu, saf + testli
        ├── motor/     gercek.rs  yok.rs  mod.rs
        ├── tabs/      agac.rs  durum.rs  oturum.rs  surucu.rs  mod.rs
        ├── memory/    esik.rs    olcum.rs  gozcu.rs  mod.rs
        ├── history/   depo.rs    gecisler.rs  mod.rs
        ├── theme/     paket.rs   jeton.rs  mod.rs
        ├── bridge/    muiget.rs  muiply.rs  muiwatch.rs  muifly.rs  mod.rs
        ├── settings/  mod.rs
        └── commands/  mod.rs  tabs.rs  gezinme.rs  ayarlar.rs
                       bellek.rs  gecmis.rs  theme.rs  bridge.rs
```

`memory/` ve `history/` **yazıldı** (Faz 2 ve Faz 3). `theme/` ile `bridge/`
ve onların komut dosyaları hâlâ ileriye dönük: sırasıyla Faz 4 ve Faz 3'ün
açık maddeleri (`docs/Roadmap.md`).

Dört dosya `docs/Architecture.md` çizimindeki katmanlarda görünmüyor ama var:

- `hata.rs` — tek hata tipi (`docs/IPC.md`).
- `olaylar.rs` — olay adları. Tek yerde duruyor çünkü bir yerde
  `sekme-degisti`, başka yerde `sekme_degisti` yazmak sessizce çalışmayan bir
  dinleyici demek ve derleyici bunu yakalamıyor.
- `gunluk.rs` — bağımlılıkların yuttuğu hataları görünür kılan `log`
  uygulaması (yukarıda).
- `kisayol.rs` — kısayol tablosu. Saf ve testli; iki giriş kapısı da (React
  ve motorun hızlandırıcı kaydı) buraya çıkıyor (`docs/Frontend.md`).

`tabs/surucu.rs` deponun, motorun ve olayların buluştuğu yer. Karar vermiyor —
sırayı `agac`a, durumu `durum`a, eşiği (Faz 2) `memory/esik`e soruyor. Buranın
işi kilidi kısa tutmak, motoru çağırmak ve olayı yayınlamak. Muiply'daki
`playlist/surucu.rs` ile aynı rol.

## İkon

Üç yerde **aynı** çizim (CLAUDE.md, tutarlılık bölümü):

- `public/icons/muiren.svg`
- `index.html` favicon'u
- `src-tauri/icons/kaynak.svg` → buradan PNG/ICO üretiliyor

Çizim: koyu yuvarlak kare (`--bg-sunken`), içinde teal pusula — çember + iğne,
**iğnenin yalnız kuzey yarısı dolu**. Tek dolu öğe kuralı ailenin iskeleti.

Biri değişirse üçü birden değişir; ayrışırsa uygulama listesinde bir ikon,
görev çubuğunda başka bir ikon görünür.

## Ölçüm ortamı

`docs/Bellek.md` içindeki başarı kriterleri ölçülmeden "başardık" denmiyor.
Ölçüm için:

- Temiz Windows 11, ekran kartı sürücüsü güncel.
- Karşılaştırma tarayıcısı aynı makinede, aynı 50 sekmelik oturum, aynı sırayla.
- Ölçüm aracı: `Get-Process` üzerinden süreç ailesi toplamı (Muiren için
  `Muiren` + `msedgewebview2`), Görev Yöneticisi'nin özet rakamı değil.
- Her koşu üç kez, ortanca alınıyor.
- Ham çıktı `docs/olcumler/YYYY-AA-GG-<not>.md` olarak işleniyor.

Ölçüm yapılmadan kapatılan bir performans işi, yapılmamış sayılıyor.
