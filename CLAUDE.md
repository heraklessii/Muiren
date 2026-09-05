# Muiren — CLAUDE.md

Bu dosya, Claude Code oturumlarında context'i korumak için hazırlanmıştır.
Blanket "her oturumda şunları oku" mantığı yerine tetikleyici-durum tablosu
kullanılır. Sadece ilgili durumla eşleşen dosyayı oku — gereksiz dosya okuma
performansı düşürür.

## Proje Özeti

Muiren, **çok sekmede az RAM harcayan** bir masaüstü web tarayıcısıdır. Hedef
kitlesi oyuncular ve anime izleyicileri: onlarca sekme açık kalırken oyunun ya
da videonun payına düşen belleği geri veren, video oynatırken takılmayan, teması
ve arka planı baştan aşağı değiştirilebilen bir tarayıcı. Mui portföyünün bir
parçası — indirme Muiget'e, yerel oynatma Muiply'a, birlikte izleme Muiwatch'a,
oyun modu Muifly'a devrediliyor.

**Temel prensip: karar veren kod backend'de.** Arayüz saf görüntü + IPC.
Sebebi somut: "bu sekme uyusun" kararını bellek gözcüsü kendi iş parçacığında
veriyor ve o karar sekme çubuğunun çizilmiş olup olmamasından bağımsız olmalı.

**İkinci prensip: sayfa düşmandır.** Muiren'in açtığı her sayfa güvenilmeyen
kod. Sayfadan gelen hiçbir şey (başlık, URL, `postMessage`, indirme adı) komut
değildir; hepsi veridir ve arayüze basılmadan önce kaçışlanır.

## Kapsam Dışı Kararlar (ÖNEMLİ — sapma)

Bunlar bilinçli maliyet/karmaşıklık trade-off'u, "eksik" değil. Genişletmeden
önce `docs/Roadmap.md` güncellenir.

- **Kendi render motoru YOK.** Blink/Gecko yazılmıyor. Muiren bir *kabuk*:
  sekme yönetimi, bellek politikası, tema, köprüler. Sayfayı çizen WebView2.
- **Chrome Web Store / uzantı ekosistemi YOK.** WebView2 yalnız paketlenmemiş
  uzantı yükleyebiliyor; mağaza uyumluluğu vaat edilmiyor (Faz 6 tartışması).
- **Şifre yöneticisi ve kasa YOK.** Kimlik bilgisi saklamıyoruz; işletim
  sistemine ve kullanıcının mevcut yöneticisine bırakılıyor. Kendi kasanı
  yazmak, ilk günden ömür boyu güvenlik borcu almak demek.
- **Hesap senkronizasyonu / bulut YOK.** Geçmiş, yer imleri ve oturum yalnız
  diskte. Sunucu yok, hesap yok, telemetri yok.
- **Tema paketi JavaScript ÇALIŞTIRMIYOR.** Tema = jeton + varlık. Kod
  çalıştıran tema, indirilen her `.muitema` dosyasını uzaktan kod yürütmeye
  çevirirdi.
- **Kendi indirme motoru YOK.** İndirme Muiget'in işi (bkz. `docs/Kopruler.md`).
- **Mobil YOK.** Masaüstü, Windows öncelikli.

## Tetikleyici Tablosu

| Durum / Görev | Okunacak Dosya |
|---|---|
| Genel mimari, katmanlama, veri akışı, motor soyutlaması | `docs/Architecture.md` |
| RAM, sekme uyutma/atma, eşikler, ölçüm — **projenin kalbi** | `docs/Bellek.md` |
| Sekme durum makinesi, ağaç/grup, oturum geri yükleme | `docs/Sekmeler.md` |
| Video, donanım kod çözme, codec, tam ekran, DRM | `docs/Medya.md` |
| Tema paketi, arka plan, jeton eşleme, `.muitema` biçimi | `docs/Temalar.md` |
| Muiget / Muiply / Muiwatch / Muifly entegrasyonu | `docs/Kopruler.md` |
| Geçmiş, yer imleri, SQLite şeması, geçiş | `docs/Depolama.md` |
| Komut ve olay sözleşmesi | `docs/IPC.md` |
| Arayüz, tasarım dili, kancalar, kısayollar | `docs/Frontend.md` |
| Kurulum, WebView2 SDK, `webview2-com`, komutlar | `docs/Setup.md` |
| Ölçüm koşturma, rapor iskeleti, `MUIREN_OLCUM` | `docs/olcumler/README.md` |
| "Bu özellik hangi fazda", kapsam dışı kararlar | `docs/Roadmap.md` |
| "Neden Chromium, daha hafif motor yok mu" | `docs/Roadmap.md` karar #6 |
| Sadece bugfix / küçük stil değişikliği | Hiçbiri — direkt koda bak |

## Stack (ezberle, yeniden okuma)

- **Masaüstü:** Tauri v2 (`unstable` özelliği açık — çoklu webview gerekiyor)
- **Backend:** Rust
- **Motor:** WebView2 (Edge Chromium) — **Cargo özelliği**, varsayılan açık.
  Ham `ICoreWebView2` erişimi `webview2-com` + `windows` crate'leriyle.
- **Veritabanı:** SQLite (`rusqlite`, bundled, WAL)
- **Arayüz:** React 19 + Vite + TypeScript, durum kütüphanesi YOK
- **Testler:** `cargo test` (Rust) · `vitest` (arayüz)

## Kod Konumu

**Rust (`src-tauri/src/`)** — üç katman: komut sarmalayıcıları, modüller, dış
dünya.

- `motor/gercek.rs` — **en kritik dosya.** WebView2 sarmalayıcısı: webview
  yaratma/yıkma, `TrySuspend`/`Resume`, bellek hedefi seviyesi, olay köprüsü.
  Bozulunca sekmeler sessizce ölü doğuyor. Dizin adı motorun adını taşımıyor
  çünkü motor değiştirilebilir olmak zorunda (bkz. `docs/Architecture.md`).
- `motor/yok.rs` — motorsuz derlemenin karşılığı. `gercek.rs` ile yüzeyi
  **birebir aynı** olmak zorunda, yoksa `--no-default-features` kırılır.
  CI'da ve saf mantık testlerinde tek koşan bu.
- `tabs/agac.rs` — saf sekme sırası/ağacı kararları + testler.
- `tabs/durum.rs` — sekme durum makinesi (`Etkin` / `Arkaplan` / `Uyuyan` /
  `Atilmis`). Saf: girdi olarak durum + olay, çıktı olarak yeni durum.
- `memory/esik.rs` — **saf eşik kararı**: hangi sekme ne zaman uyur, ne zaman
  atılır. Tamamen test edilebilir, hiçbir Win32 çağrısı içermez.
- `memory/olcum.rs` — süreç bellek ölçümü, Win32'ye özel.
- `memory/esleme.rs` — **saf** süreç → sekme eşlemesi. Motorun verdiği
  PID → sekme listesiyle Win32'nin verdiği PID → MB'yi birleştirip sekme
  başına rakama çeviren defter. Durum tutuyor (atılmış sekmenin son bilinen
  değeri) ama **karar vermiyor**: çıktısı yalnız panele gidiyor.
- `memory/gecikme.rs` — uyanma gecikmesi defteri. **Saf**, `esik.rs` ile aynı
  ayrım: `Instant` parametre olarak geliyor, dosya hiç saat okumuyor.
  `docs/Bellek.md` kabul tablosunun son iki satırının ölçüm ucu.
- `memory/gozcu.rs` — periyodik döngü; `olcum` okur, `esik`e sorar, `motor`a der.
- `history/depo.rs`, `history/gecisler.rs` — geçmiş + yer imleri, SQLite.
  `history/mod.rs` içindeki `turkce_kucult` #10'un kod karşılığı.
- `kisayol.rs` — **kısayol tablosunun tek kaynağı.** Saf. İki kapı buraya
  çıkıyor: `kisayol_bas` komutu (kabuk odaktayken) ve motorun
  `AcceleratorKeyPressed` kaydı (sayfa odaktayken). İkinci bir tablo yazmak
  "bazen çalışmayan kısayol" demek (`docs/Frontend.md`).
- `gunluk.rs` — `log::Log` uygulaması. Tauri ve wry hatalarını `log::error!`
  ile yazıp `Ok` dönebiliyor; bu dosya olmadan o satırlar hiçbir yere
  gitmiyor.
- `theme/paket.rs` — `.muitema` okuma **ve doğrulama**. Doğrulanmamış tema
  uygulanmaz.
- `tabs/grup.rs` — sekme grupları. Saf: ad temizleme, renk enum'ı, eşik
  düzeltme. Grup eşiği `esik.rs`e **tablo olarak değil**, sekme özetinde
  hazır bir değer olarak giriyor.
- `favicon/mod.rs` — favicon deposu. Dosya adı **içeriğin karması**; kabuğa
  yalnız kimlik gidiyor, baytlar `data:` adresi olarak isteniyor. Uzak adres
  kabuğa **hiç** basılmıyor.
- `engel/liste.rs` — filtre kuralı ayrıştırma ve eşleştirme. **Saf**, tıpkı
  `theme/jeton.rs` gibi. Anlaşılmayan satır sessizce atılmıyor, sayılıp geri
  gösteriliyor.
- `engel/mod.rs` — pop-up politikası + istek filtresi. Liste ayarlar
  yazıldığında **bir kez** derleniyor, her istekte değil.
- `bridge/muiget.rs` · `bridge/muiply.rs` · `bridge/muiwatch.rs` ·
  `bridge/muifly.rs` — kardeş uygulamalara devir. `bridge/oyun.rs` tam ekran
  oyun algılaması (karar saf, Win32 ölçümü ayrı — `memory/olcum.rs` ile aynı
  ayrım).
- `settings/mod.rs` — kalıcı tercihler, düzeltme ve motora uygulama.
  `Settings` üzerinde `#[serde(default)]` var: eklenen yeni bir alan
  kullanıcının ayar dosyasını `.bozuk` yapıp sıfırlamıyor.
- `olcum/` — **ölçüm modu**, yalnız `MUIREN_OLCUM` tanımlıysa çalışıyor
  (`docs/olcumler/README.md`, karar #8). `plan.rs` ve `rapor.rs` **saf ve
  testli**; `kosucu.rs` sırayı tutan iş parçacığı. Komut yüzeyi yok,
  arayüzde düğmesi yok.
- `commands/` — ince sarmalayıcılar. **Buraya mantık yazılmaz.**

**Arayüz (`src/`)**

- `ipc/` — `invoke` sarmalayıcıları ve tipler. Bileşenler `invoke` görmez.
- `hooks/` — backend durumunun yansımaları. **Kısayol tablosu burada yok**:
  `App.tsx` tuşu olduğu gibi `kisayol_bas`a veriyor.
- `lib/` — saf ve testli: `url.ts` (adres/arama ayrımı), `suz.ts`, `bicim.ts`,
  `teshis.ts` (GPU/codec/sürüm yorumlaması; ölçüm `hooks/useTeshis.ts` içinde
  — `esik.rs`/`olcum.rs` ayrımının arayüzdeki karşılığı).
- `components/TeshisPaneli.tsx` — GPU, codec, DRM ve ölçüm raporu başlığı
  (`docs/Medya.md`). Üçüncü tam ekran örtü; `ortuAcik` içinde (#21).
- `styles.css` — Mui jetonları, gömülü Outfit.

Komutlar: `npm run tauri dev` · `npm test` ·
`cd src-tauri && cargo test --no-default-features`

## Yazarken Dikkat

1. **`commands/`ye iş mantığı EKLEME.** Aynı karara bellek gözcüsünün
   döngüsünden de gelinebiliyor; iki kopya birbirinden sessizce ayrışır.
2. **Arayüze iş mantığı EKLEME.** Uyutma, atma, sekme sırası ve köprü kararları
   backend'de. Arayüz kapalıyken de doğru çalışmalılar.
3. **Yeni komut → önce `docs/IPC.md`.** Sözleşme orada; Rust ve TypeScript
   ondan sonra.
4. **`motor/yok.rs` dosyasını güncellemeyi unutma.** `gercek.rs` içine yeni bir
   metot eklediğinde karşılığı orada da olmalı.
5. **Uyuyan sekmeyi ölçmek için UYANDIRMA.** Bellek raporu, arama, önizleme —
   hiçbiri uyuyan sekmeye dokunmaz; son bilinen değeri kullanır. Bir kez
   uyandırdığın anda projenin tek iddiası çöker.
6. **Bellek eşiği sabitini koda gömme.** Hepsi `settings` içinden gelir;
   makineye göre değişiyor (8 GB'lık dizüstü ile 32 GB'lık masaüstü aynı
   politikayı kaldırmıyor).
7. **Başarısız gezinmeyi başarılı sayma.** `NavigationCompleted` olayının
   `IsSuccess` alanı `false` ise geçmişe yazılmaz, sekme başlığı eskisiyle
   kalmaz, oturum dosyasına o URL işlenmez.
8. **Sayfadan gelen metni ham basma.** `document.title` saldırganın yazdığı bir
   dize; sekme çubuğuna, geçmişe, köprü mesajına girmeden önce kırpılır ve
   kaçışlanır.
9. **Renk/ölçü sabiti yazma.** Hepsi `src/styles.css` başındaki jetonlardan.
10. **Türkçe küçük harf `toLocaleLowerCase("tr")`.** Düz `toLowerCase()`
    "İSTANBUL" dizesini `"istanbul"` yapmıyor; geçmiş araması eşleşmiyor.
    **Kapsamı kullanıcı metni**: teknik ASCII tanımlayıcıda (GPU sürücü
    dizesi, ortam değişkeni, MIME) tersi doğru — Türkçe küçültme
    "SWIFTSHADER" dizesini "swıftshader" yapıp eşleşmeyi düşürüyor
    (`src/lib/teshis.ts`).
11. **Türkçe ek üretme.** "Sekme {ad}'de" çalışmıyor (Ayşe'de ama Onur'da).
    Ada ek gerektirmeyen kalıp kullan.
12. **`history` tablosuna sütun eklerken `history::gecisler` içine de ekle.**
    `CREATE TABLE IF NOT EXISTS` var olan tabloyu değiştirmiyor; kullanıcının
    diskindeki veritabanı eski şemada kalıp ilk sorguda patlıyor.
13. **Yeni ayar → hem `Settings` hem `duzelt` hem `oku`/`yaz`.** Anahtar
    eklenip `duzelt` unutulduğunda geçersiz bir değer doğrudan motora gidiyor
    ve hata çalışma zamanına kadar görünmüyor.
14. **Tema alanı eklerken `paket::dogrula` içine de ekle.** Doğrulanmayan alan,
    dışarıdan indirilen bir dosyanın arayüzü ele geçirmesi demek.
15. **Chromium bayrağı yalnız `EK_BAYRAKLAR` içinde, gerekçesiyle birlikte.**
    Site izolasyonunu kapatan ve GPU hızlandırmayı devre dışı bırakan bayraklar
    yasak — ilki Spectre savunmasını söküyor, ikincisi video takılmasının bir
    numaralı sebebi. Eklenen her bayrağın etkisi ölçülerek doğrulanıyor;
    WebView2 bir kısmını sessizce yok sayıyor (`docs/Setup.md`).
16. **Her webview aynı bayrak dizesini alır.** wry her webview için ayrı bir
    WebView2 ortamı kuruyor ve WebView2 aynı kullanıcı veri klasörünü
    paylaşan ikinci ortamı yalnız seçenekleri **birebir aynıysa** yaratıyor.
    Ayrışırlarsa webview hiç doğmuyor ve hata `add_child`in `Ok`u altında
    kayboluyor. Dize `lib.rs` içinde bir kez hesaplanıp motorda donuyor;
    yeni bir webview türü eklenirse (önizleme, ayrı pencere) o da aynı
    kopyayı alır (`docs/Setup.md`).

    > Gizli sekme bu kuralı **bozmuyor**: InPrivate bir *denetleyici*
    > seçeneği (`SetIsInPrivateModeEnabled`), ortam seçeneği değil. Bu yüzden
    > gizli sekmeye ayrı bir kullanıcı veri klasörü açılmadı — açılsaydı
    > ikinci bir ortam kurulur ve tam da bu tuzağa düşülürdü.
17. **Gizli sekmede üç kapıyı da kapat.** Motorun InPrivate kipi çerezi ve
    önbelleği bellekte tutuyor ama **geçmiş, oturum dosyası ve favicon deposu
    bizim**. Motorun gizliliği onları bilmiyor; üçü `tabs/surucu.rs` içinde
    ayrı ayrı kontrol ediliyor. Yeni bir kalıcı depo eklendiğinde dördüncü
    kapı da kapatılır, yoksa "gizli" iddiası sessizce çöker.
18. **Sayfadan gelen dosya adını köprüye ham verme.** `Content-Disposition`
    saldırganın yazdığı bir dize ve karşı tarafta bir dosya yoluna dönüşüyor.
    `bridge::dosya_adi_temizle` geçmeden hiçbir köprüye gitmiyor —
    `baslik_temizle` ile aynı kural, farklı hedef (#8).
19. **Köprüden yalnız adres ya da dosya yolu geçer.** `bridge::Yuk` bunu tip
    düzeyinde zorluyor. Çerez, oturum başlığı ve sayfa içeriği geçmiyor;
    "bir de şu başlığı gönderelim" değişikliği derleyicinin önüne çıkıyor
    (`docs/Kopruler.md`).
20. **Engelleme sessiz olmaz.** Engellenen pop-up, yönlendirme ya da istek
    `muiren://engellendi` ile duyuruluyor ve adres çubuğunda rozet çıkıyor.
    Bazı siteler ödeme akışını pop-up'ta açıyor; sessiz engelleme kullanıcıya
    "tarayıcı bozuk" dedirtiyor.
21. **Yeni tam ekran örtü → `App.tsx` içindeki `ortuAcik` ifadesine ekle.**
    Sekme webview'i ayrı bir native pencere ve kabuğun **üstünde**; kabuğun
    çizdiği örtü sayfanın altında kalıyor ve `z-index` işe yaramıyor.
    Eklenmeyen örtü sessizce görünmez oluyor (`docs/Frontend.md`).
22. **`WebResourceRequested` süzgecini filtre kapalıyken kaydetme.** Süzgeç
    sayfadaki **her** alt kaynağı kabuk sürecinden geçiriyor. Karar webview
    yaratılırken bir kez veriliyor (`Motor::istek_suzgeci_acik`); bedeli, ayar
    değiştiğinde yalnız yeni/yenilenen sekmelerin etkilenmesi ve arayüz bunu
    rozetle söylüyor.
23. **Sekme başına belleği `SekmeOzeti` içine taşıma.** Rakam `BellekOzeti`
    içinde (`sekme_mb`) ve arayüz iki listeyi `id` üzerinden birleştiriyor.
    Sebebi: `SekmeOzeti` aynı zamanda `esik.rs`in girdisi ve eşik kararı
    sekme başına rakama **dayanmıyor** — boşta kalma süresine ve sistem
    baskısına dayanıyor (`docs/Bellek.md`). Alanı oraya koymak, saf karara
    bir gün sızacak bir veriyi elinin altına bırakmak olurdu.

## Diğer Mui Projeleriyle Tutarlılık

- Dokümantasyon-first: önce `docs/`, sonra implementasyon.
- CLAUDE.md blanket instruction yerine trigger-table (Muivly'den öğrenilen
  ders; Muiply ve Muiwatch'ta da böyle).
- Tauri v2 tercihi Muiget/Muivly/Muiwatch/Muiply stack pattern'iyle uyumlu.
- **Tasarım dili kardeş projelerden birebir devralındı:** teal `#2dd4bf`
  vurgu, koyu `#0f1115` zemin, gömülü Outfit, Türkçe sınıf adları
  (`.dugme`, `.kart`, `.rozet`). Kanonik kaynak `..\Muiget\src\styles.css`;
  jeton listesi `..\MuiLabs\docs\ui-conventions.md`.
  **İstisna:** kullanıcı teması yalnız `docs/Temalar.md` içinde sayılan
  jetonları değiştirebilir; Mui kimliği (ikon, hakkında ekranı) temadan
  etkilenmez.
- Uygulama ikonu ailenin iskeleti: koyu yuvarlak kare + teal glyph + **tek
  dolu öğe**. Üç yerde aynı çizim: `public/icons/muiren.svg`, `index.html`
  favicon'u, `src-tauri/icons/kaynak.svg`. Biri değişirse üçü de değişir.
  Muiply'ın glyph'i çember + üçgen (işi: dosyayı açıp çalmak); Muiren'inki
  **pusula** — çember + iğne, iğnenin yalnız kuzey yarısı dolu, çünkü işi tek
  şey: adresi alıp oraya gitmek.
