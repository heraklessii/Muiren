# Yol Haritası

| Faz | Konu | Durum |
|---|---|---|
| Faz 0 | Fizibilite — dört riskin kapatılması | R1/R3 kısmen, R2/R4 **ölçüm aracı hazır**, sayı yok |
| Faz 1 | Çalışan tarayıcı (sekme, gezinme, oturum) | **Kod tamam** · kabul denemesi bekliyor |
| Faz 2 | Bellek politikası — **projenin tezi** | **Kod tamam** (4. adım dahil) · ölçüm **koşturulabilir**, rapor yok |
| Faz 3 | Kabuk (geçmiş, yer imi, indirme devri, ayarlar) | **Kod tamam** · gerçek Muiget kurulumuyla deneme bekliyor |
| Faz 4 | Temalar + oyun modu + engelleme | **Kod tamam** · oyun algılaması gerçek oyunla denenmedi |
| Faz 5 | Muiwatch köprüsü, dikey sekmeler, gruplar | **Muiren tarafı tamam** · Muiwatch'ın kendi Faz 3'ü bekliyor |
| Faz 6 | Tartışmalı: uzantılar, çoklu platform | Karar verilmedi |

> **"Kod tamam" ≠ "faz kapandı".** Faz 1'in kabul kriteri 20 sekmeyle bir
> günlük gerçek kullanım, Faz 2'ninki `docs/Bellek.md` içindeki ölçüm
> tablosu. İkisi de makine başında zaman geçirmeyi gerektiriyor ve ölçüm
> `docs/olcumler/` altına yazılmadan faz kapanmıyor. Aşağıdaki durum
> bölümleri neyin yazıldığını ve neyin ölçülmediğini ayrı ayrı söylüyor.

---

## Faz 0 — Fizibilite (önce bu)

Bu proje dört varsayımın üstünde duruyor. **Kod yazmadan önce dördü de
kanıtlanacak.** Çıktı: çalışan bir çivi (spike) ve `docs/olcumler/` altına bir
rapor.

**R1 — Askıya alma çalışıyor mu?**
Tauri v2 + `webview2-com` ile ham `ICoreWebView2` alınabiliyor mu,
`TrySuspend`/`Resume` çağrılabiliyor mu, çağrıldığında **ölçülebilir** bellek
düşüşü var mı. Yoksa `Uyuyan` durumu düşüyor, politika yalnız `Atilmis`
üzerinden yürüyor (daha kaba ama hâlâ işe yarıyor).

**R2 — DRM çalışıyor mu?**
Crunchyroll'da bir bölüm baştan sona oynuyor mu. Oynamıyorsa karar:
(A) DRM'li siteleri sistem tarayıcısına devret — dürüst ama zayıf,
(B) motoru CEF'e çevir — ~150 MB dağıtım, `motor/` soyutlaması bunun için var.
**Bu kararı ertelemek en pahalı seçenek** (`docs/Medya.md`).

**R3 — Çoklu webview taşıyor mu?**
Tauri'nin `unstable` çoklu webview'ı 30+ webview ile ayakta kalıyor mu, sekme
değiştirme gecikmesi kabul edilebilir mi, webview yaratma/yıkma sızdırıyor mu.

> **Kısmi cevap var (Faz 1).** API'nin kendisi çalışıyor: dört sekme webview'i
> aynı pencerede eşzamanlı ayakta, gezinme ve olay köprüsü dahil (aşağıda
> Faz 1 → "ÇÖZÜLDÜ"). Uzun süre R3'ün olumsuz cevabı sanılan tablo aslında
> **WebView2 ortam seçeneği uyumsuzluğuydu**, Tauri'nin API'si değil.
> Ölçek (30+) ve sızıntı sorusu hâlâ açık.

**R4 — Süreç bayrakları geçiyor mu?**
WebView2 bazı Chromium anahtarlarını yok sayıyor. `--process-per-site`,
`--renderer-process-limit=N` ve `--js-flags` `AdditionalBrowserArguments`
üzerinden gerçekten uygulanıyor mu? Ölçüm: aynı siteden 10 sekme açıp render
süreç sayısını say. 10 süreç kalıyorsa bayrak geçmemiş; ~1 süreç görüyorsan
geçmiş.

Bu, `docs/Bellek.md` içindeki **2. kazanç kaynağının** tamamı. Geçmiyorlarsa
uyanık sekmelerin taban maliyetini düşüremiyoruz ve bu, CEF'e geçişin (R2 ile
aynı karar) ikinci gerekçesi olur — CEF süreç modelini tamamen açıyor. İki
riskin aynı çözümü işaret etmesi, kararı R2'yle birlikte vermeyi gerektiriyor.

Kabul kriteri: dört sorunun da yazılı cevabı var ve mimari o cevaplara göre
güncellendi.

### Faz 0 — ölçüm araçları hazır (2026-09-05)

Dört riskin hiçbiri "cevaplandı" değil, ama üçünün **ölçüm ucu** artık yazılı
ve elle yapılacak iş birkaç dakikaya indi:

| Risk | Araç | Kalan iş |
|---|---|---|
| R1 — `TrySuspend` kazandırıyor mu | ölçüm modu + bellek paneli | 20 sekme aç, "Hepsini uyut", iki rakamı karşılaştır |
| R2 — DRM | **teşhis paneli** (Widevine/PlayReady sorgusu) | anahtar sistemi varsa gerçek bir bölüm; yoksa A/B kararı **hemen** verilebilir |
| R3 — 30+ webview | ölçüm modu (plan dosyasındaki adres sayısı) | sızıntı denemesi elle |
| R4 — süreç bayrakları | ölçüm modu raporundaki "WebView2 süreci" | aynı siteden 10 adresli bir plan koşturmak |

Codec tablosu da (aşağıda "doğrulanacak" diye duruyordu) artık teşhis
panelinde ölçülüyor: `MediaCapabilities` her codec için donanım/yazılım
ayrımını veriyor.

**Ölçüm modu uçtan uca koşturuldu** (2026-09-05, hata ayıklama derlemesi, iki
adreslik duman testi): plan okundu, sekmeler açıldı, üç aşama ölçüldü, rapor
iskeleti diske yazıldı ve uygulama kendini kapattı. Bu bir **kabul ölçümü
değil** — dev derleme, iki adres, oturumdan gelen sekmeler — ve `docs/olcumler/`
altına yazılmadı. Koşu bir kusur da gösterdi ve düzeltildi: oturumdan gelen
sekmeler varken alınan "boş tarayıcı" satırı taban maliyet değil, dolayısıyla
hedefle karşılaştırılmıyor (rakam yazılıyor, "geçersiz" etiketiyle).

---

## Faz 1 — Çalışan tarayıcı

Bellek politikası **yok**. Amaç: gezinilebilir bir tarayıcı.

- `motor/` soyutlaması + `gercek.rs` + `yok.rs`.
- Sekme kaydı, ağaç sırası (`tabs/agac.rs` + testleri).
- Adres çubuğu, geri/ileri/yenile, sekme şeridi.
- Oturum kaydetme/geri yükleme (açılışta hepsi `Atilmis`).
- `src/lib/url.ts` — URL/arama ayrımı, alan adı vurgusu, IDN (+ testleri).

Kabul: 20 sekmeyle bir gün boyunca gerçek kullanım, çökme yok, oturum kaybı yok.

### Faz 1 — durum

Yazıldı ve derleniyor (`cargo test` + `cargo test --no-default-features` +
`npm test` geçiyor):

- `motor/` soyutlaması — `Motor` trait'i, `gercek.rs` (WebView2), `yok.rs`.
  `TrySuspend`/`Resume`, bellek hedefi, `SetIsMuted` ve olay köprüsü (başlık,
  adres, gezinme, geçmiş, ses, yeni pencere) bağlı.
- `tabs/agac.rs` — sıra ve ağaç, saf + testler.
- `tabs/durum.rs` — dört durumlu makine, saf + testler.
- `tabs/oturum.rs` — atomik yazım, `.bak`a düşme, açılışta `Atilmis` doğuş.
- `tabs/surucu.rs` — depo + motor + olay yayını.
- `settings/` — `Settings` / `duzelt` / `oku` / `yaz` + testler.
- Arayüz: sekme şeridi, adres çubuğu, yeni sekme sayfası, sekme arama
  (`Ctrl+Shift+A`), `src/lib/url.ts` (adres/arama ayrımı, alan adı vurgusu,
  punycode + IDN homograf savunması) ve testleri.

**ÇÖZÜLDÜ — Faz 1'i bloke eden sorun.**

*Belirti:* Sekme webview'i `Window::add_child` ile yaratılıyor, çağrı `Ok`
dönüyor, pencerede HWND beliriyor ve `app.webviews()` içinde görünüyor. Ama
sayfa açılmıyor: sekme 1×1 boyutta, gezinmeden, olaysız kalıyor. Kabuk
webview'i aynı pencerede sorunsuz çalışıyor.

*Sebep:* İki webview'in **Chromium bayrak dizeleri ayrışıyordu.** Varsayım
şuydu: "WebView2 ortamı ilk webview'de kuruluyor, sekmelere ayrıca bayrak
vermenin etkisi yok." Yanlış. wry her webview için ayrı bir
`CreateCoreWebView2EnvironmentWithOptions` çağırıyor
(`wry/src/webview2/mod.rs`, `new_in_hwnd`), WebView2 ise aynı kullanıcı veri
klasörünü paylaşan ikinci ortamı **yalnız seçenekleri birebir aynıysa**
yaratıyor; ayrışırsa `ERROR_INVALID_STATE` (0x8007139F) dönüyor.

Kabuk `EK_BAYRAKLAR`ı alıyordu; sekmeye bayrak verilmediği için wry kendi
varsayılanını koyuyordu ve o varsayılan fazladan
`--autoplay-policy=no-user-gesture-required` içeriyor. İki dize ayrıştı, sekme
ortamı kurulamadı.

Hatanın **görünmemesinin** sebebi ayrı: `tauri-runtime-wry` içindeki
`Message::CreateWebview` işleyicisi wry'nin hatasını `log::error!` ile yazıp
çağırana yine de `Ok` dönüyor (`lib.rs`, ~4057. satır) — ve `log` crate'inin
kurulu bir uygulaması yoktu, yani o satır hiçbir yere gitmiyordu. HWND'nin
yine de belirmesi de tesadüf değil: wry kap pencereyi ortamdan **önce**
yaratıyor.

*Düzeltme:*

1. Bayrak dizesi `lib.rs` içinde açılışta **bir kez** hesaplanıp motorda
   donduruluyor; kabuk da sekmeler de o tek kopyayı alıyor
   (`motor/gercek.rs`, `sekme_ac`). Ayarlar çalışırken değişse bile dize
   değişmiyor — bayrak ayarlarının "yeniden başlatma gerektiriyor" olmasının
   asıl sebebi bu.
2. `src/gunluk.rs` — ince bir `log::Log` uygulaması. Bağımlılıkların yuttuğu
   hatalar bir daha sessiz kalmıyor. Eşik `MUIREN_LOG` ile ayarlanıyor,
   varsayılan `warn`.
3. `--autoplay-policy=no-user-gesture-required` **bilinçli olarak alınmadı**:
   uygulama kabuğu için makul, tarayıcı için değil.

*Faz 0/R3'e katkısı:* Tauri'nin `unstable` çoklu webview'ı ayakta — sorun API
değil, WebView2'nin ortam kuralıydı. Karar #1 ve CEF seçeneği yerinde kalıyor.
R3'ün asıl sorusu (30+ webview taşınıyor mu, sızıntı var mı) hâlâ ölçülmedi.

*Doğrulama (2026-09-04):* Oturum dosyasına dört sabitlenmiş sekme yazılıp
uygulama açıldı. Dördü de webview aldı, gezindi ve **başlıkları oturum
dosyasına geri yazıldı** — yani webview yaratma, gezinme, WebView2 olay
köprüsü, dinleyici ve oturum yazımı zinciri uçtan uca çalışıyor. Kanıtın
dosyaya düşmüş hâli:

```
id  url                     baslik
 1  https://example.com/    Example Domain
 2  https://example.org/    Example Domain
 3  https://rust-lang.org/  Rust Programming Language
 4  https://tauri.app/      Tauri 2.0
```

(3 numaranın adresindeki `www` düşmesi sitenin kendi yönlendirmesi;
`SourceChanged` olayının doğru bağlandığının ikinci kanıtı. Dört webview'in
aynı anda ayakta durması, çoklu ortam kuralının artık sağlandığını da
gösteriyor.)

**Faz 1'de eksik kalanlar** — hepsi kapandı, biri hariç:

1. ~~Kısayolların motor tarafı.~~ **Tamam.** Tablo tek yerde
   (`src-tauri/src/kisayol.rs`, saf + testli) ve ona iki kapıdan giriliyor:
   kabuk odaktayken `kisayol_bas` komutu, sayfa odaktayken
   `motor/gercek.rs` içindeki `AcceleratorKeyPressed` kaydı. İki ayrı tablo
   yazılmadı — "bazen çalışmıyor" hatasının kaynağı tam olarak o olurdu
   (`docs/IPC.md`).
2. ~~Sürükle-bırak ile sekme taşıma.~~ **Tamam.** Arayüz yalnız hedef sırayı
   bildiriyor; taşımayı `agac::tasi` yapıyor ve çocukları birlikte
   götürüyor.
3. ~~Favicon.~~ **Tamam.** `favicon/` deposu yazıldı: motorun `GetFavicon`
   baytları diske, dosya adı **içeriğin karması**. Kabuğa yalnız bir kimlik
   gidiyor ve baytlar `favicon_oku` ile `data:` adresi olarak alınıyor —
   uzak bir adresi kabuğa basmak, açılan her sitenin kabuk penceresine ağ
   isteği yaptırabilmesi demek olurdu. Aynı ikonu kullanan 30 sekme tek
   dosya, tek okuma.
4. ~~Kaydırma konumu.~~ **Tamam.** Oran olarak saklanıyor (piksel değil:
   sekme geri geldiğinde pencere ya da sayfa değişmiş olabiliyor). Sorulma
   anı önemli — webview **hâlâ ayaktayken**, yani sekme arka plana düşerken
   ve uyumadan önce. Atılma anında sormak yarış olurdu.
5. ~~Kenarlıksız pencere / özel başlık çubuğu.~~ **Tamam.** Başlık çubuğunun
   işini sekme şeridi görüyor; sürükleme ve Aero Snap
   `data-tauri-drag-region` üzerinden.
6. **20 sekmelik gerçek kullanım denemesi** — kabul kriterinin kendisi,
   **yapılmadı.** Kod tarafında bilinen bir engel yok; bu madde makine
   başında bir gün geçirmeyi gerektiriyor.

---

## Faz 2 — Bellek politikası

Projenin sebebi.

- `tabs/durum.rs` — dört durumlu makine + testleri.
- `memory/esik.rs` — saf karar + koruma kuralları + testleri.
- `memory/olcum.rs` — sistem baskısı, süreç ölçümü.
- `memory/gozcu.rs` — arka plan döngüsü.
- **Süreç birleştirme** — `EK_BAYRAKLAR`, `surec_politikasi` /
  `renderer_tavani` ayarları, yeniden başlatma rozeti. R4 olumluysa buraya,
  olumsuzsa CEF kararına.
- Bellek paneli, sekme çubuğunda durum göstergeleri.
- RAM profillerine göre varsayılan eşikler.

**Kabul kriteri `docs/Bellek.md` içindeki tablo.** Özellikle: 50 sekme,
15 dakika boşta, Chrome'un yarısından az. Ölçüm `docs/olcumler/` altına
işlenmeden faz kapanmıyor.

Bu faz bittiğinde proje anlatılabilir hâle geliyor; öncesinde anlatacak bir şey
yok.

### Faz 2 — durum

**Kod tamam, ölçüm yok.**

Yazıldı ve testleri geçiyor:

- `memory/esik.rs` — **saf** karar. Girdi sekme özetleri + baskı + ayarlar,
  çıktı eylem listesi; sistem çağrısı yok, zaman okuma yok (boşta kalma
  süresi `SekmeOzeti.bosta_sn` içinde hazır geliyor). Sekiz koruma
  kuralından yedisi burada ve her birinin testi var; yedincisi (Muiwatch
  oturumu) Faz 5'e ait ve bilinçli olarak yok.
- `memory/olcum.rs` — `GlobalMemoryStatusEx` ile sistem baskısı,
  Toolhelp32 + `GetProcessMemoryInfo` ile süreç bellekleri. WebView2
  süreçleri **ada göre değil soy ağacına göre** filtreleniyor: aynı isimli
  süreçler Teams ve Office'te de koşuyor ve hepsini toplamak kullanıcıya
  kendi tarayıcısının üç katı bellek yediğini gösterirdi.
- `memory/gozcu.rs` — kendi iş parçacığında döngü; ölçüyor, `esik`e
  soruyor, `durum_degistir` kapısından uyguluyor, `muiren://bellek-ozeti`
  yayınlıyor. Oyun modu baskıyı `Yuksek` sayıp uyanık sınırını 2'ye
  çekiyor.
- **Pencere simge durumu.** `docs/Bellek.md` bunu baştan beri tarif ediyordu
  ("kullanıcı tarayıcıya bakmıyorsa 14 sekmenin uyanık kalmasının hiçbir
  karşılığı yok") ama **kodda karşılığı yoktu**: gözcü pencerenin durumunu
  hiç bilmiyordu. Bağlandı.

  Karşılığı yeni bir eşik değil, var olan baskı tablosu: gizli pencere en az
  `Yuksek` sayılıyor (`esik::BellekBaskisi::etkin`, saf + 5 test).
  Birleştirme `max` ile — `Kritik` ölçülmüşken pencereyi küçültmek
  politikayı **gevşetmiyor**; testin varlık sebebi bu.

  Periyot yarıya iniyor ama turun bedeli de düşüyor: gizliyken süreç tablosu
  taranmıyor ve özet yayınlanmıyor. Paneli görmeyen kullanıcı için
  Toolhelp32 anlık görüntüsünün karşılığı yok ve iki katı sıklıkta tam ölçüm,
  bedeli tam da oyun anına yığardı.

  Uyanık üst sınırı **düşürülmüyor**: oyun modundan farkı kullanıcının bir
  şey söylememiş olması. Sekmenin uyuma sebebi arayüze `PencereGizli` olarak
  gidiyor, `SistemBaskisi` olarak değil.

  > **Aynı bölümdeki açık kapandı (2026-09-05).** Oyun modunun 4. adımı
  > (`docs/Bellek.md`) yazıldı: `oyun_pencere_gizle` ayarı, **varsayılan
  > kapalı**, ve pencere gizlenmiyor **küçültülüyor** — gizlenen pencerenin
  > görev çubuğu düğmesi de kaybolur ve tam ekran bir oyun ekranı kaplarken
  > kullanıcının geri dönecek görünür bir yolu kalmazdı. Geri getirme
  > garantili: pencereyi biz küçülttüysek (`oyun_kucultu` bayrağı) oyun
  > bitince biz geri getiriyoruz, kullanıcının kendi küçülttüğü pencereye
  > dokunulmuyor ve gözcünün her turu bu denetimi tekrarlıyor. Karar kaydı #7.
- `memory/gecikme.rs` — **uyanma gecikmesi defteri**, saf ve testli.
  Kabul tablosunun son iki satırı (< 300 ms, < 2 sn) uzun süre ölçülemiyordu:
  gecikmeyi sayan bir yer yoktu ve elle kronometre tek örnek verir, tablo ise
  dağılım istiyor. Kronometreler `tabs/surucu.rs` içindeki dört anda
  başlıyor/duruyor; hesabın tamamı saf tarafta (`esik.rs` ile aynı ayrım,
  `Instant` parametre olarak geliyor).

  İki ölçüm **farklı şeyler sayıyor ve öyle etiketleniyor**: `Uyuyan`
  gerçekten "etkileşime hazır" (sayfanın JS iş parçacığı cevap verdi),
  `Atilmis` ise ilk boya **değil** onun üst sınırı — WebView2 bir ilk boya
  olayı vermiyor ve olmayan bir olay varmış gibi gösterilmiyor.

  Motor yüzeyine ölçüm için yeni bir metot eklenmedi (CLAUDE.md #4): uyanma
  turu `kaydirma_iste`nin zaten yaptığı `ExecuteScript` turuna binmiş
  durumda.

- Bellek paneli (yan panelde), sekme çubuğunda durum göstergeleri ve
  koruma sebebini yazan ipuçları. Panel gecikme dağılımını da gösteriyor;
  örnek yokken "henüz ölçülmedi" yazıyor, sıfır değil.
- `docs/olcumler/README.md` — **ölçüm protokolü.** Kabul tablosunun altı
  satırının da adım adım yöntemi, ortam tablosu (sürümler boş bırakılmıyor:
  WebView2 Runtime kendi güncelleniyor ve iki ölçüm arasındaki farkın sebebi
  bizim kodumuz olmayabilir) ve rapor iskeleti. Faz 0/R1, R3, R4'ün ölçüm
  yöntemi de burada.
- Ayarlar ekranı: profil, eşikler, uyanık üst sınır, istisna alanları,
  süreç politikası (yeniden başlatma rozetiyle).

- **Ölçüm modu** (`src-tauri/src/olcum/`, `MUIREN_OLCUM`). Protokolün ilk
  dört satırı artık koşturuluyor: plan dosyasındaki adresler açılıyor,
  aşamalar ölçülüyor, rapor `docs/olcumler/` iskeletinde diske yazılıyor.
  Plan ve rapor **saf ve testli** (`plan.rs`, `rapor.rs`); koşucu yalnız
  sıra tutuyor — `esik.rs`/`gozcu.rs` ayrımının aynısı.

  Mod normal kullanımda **yok**: değişken tanımlı değilse tek satırı
  çalışmıyor, arayüzde düğmesi yok, IPC yüzeyine komut eklemiyor. Sekme
  kapatmıyor ve gözcüyü durdurmuyor (protokol: "ölçtüğümüz şey politikanın
  çalıştığı hâl"). Chrome'u ölçmüyor ve uyanma gecikmesi üretmiyor; rapor
  iki sınırı da kendi içinde yazıyor.

**Ölçülmedi ve bu fazın kapanmasını engelliyor:**

> Artık eksik olan **araç değil, makine başında geçirilecek zaman** — ve o
> zaman da kısaldı: ilk dört satır bir plan dosyasıyla koşuluyor, geriye
> Chrome'u yanında koşturmak ve son iki satır için sekmelere tıklamak
> kalıyor. Dizin hâlâ raporsuz ve faz o yüzden açık.

- `docs/Bellek.md` başarı tablosunun tamamı — 50 sekmelik oturumla Chrome
  karşılaştırması, boş tarayıcı < 150 MB hedefi, uyanma gecikmeleri.
  (Gecikmelerin **ölçüm aracı** yazıldı: bellek panelinde medyan, ipucunda
  `n`/p95/en kötü duruyor ve "Ölçümü sıfırla" ile temiz bir oturum
  başlatılıyor. Ama panelde bir sayı görmek rapor demek değil.)
- **Faz 0/R4** — `--process-per-site` ve `--renderer-process-limit`
  WebView2'den geçiyor mu. Ölçüm aracı hazır: bellek paneli "WebView2
  süreci" sayısını gösteriyor; aynı siteden 10 sekme açıp bakmak yetiyor.
- **Faz 0/R1** — `TrySuspend`in ölçülebilir bellek düşüşü yaratıp
  yaratmadığı. Yetenek algılama çalışıyor (`Yetenekler.askiyaAlma`), ama
  kazancın büyüklüğü ölçülmedi.

**Kapanan sınır — süreç → sekme eşlemesi.** Uzun süre yoktu ve
`olcumYaklasik` **her zaman** `true` idi; panel sekme başına rakam
gösteremiyor, kazancı uyanık sekmelerin ortalamasından hesaplıyordu.
`docs/Bellek.md` iki yol bırakmıştı ve 1. yol yürüdü:
`ICoreWebView2Environment13::GetProcessExtendedInfos` +
`ICoreWebView2_20::FrameId`. Hesap saf tarafta (`memory/esleme.rs`, 14 test),
motor yalnız PID → sekme listesi veriyor; bellek rakamı motora hiç girmiyor
ki birleştirme `--no-default-features` derlemesinde de test edilebilsin.

Üç şey değişti:

- Panel sekme başına MB gösteriyor; paylaşılan süreç eşit bölünüp öyle
  etiketleniyor, bu turda ölçülmemiş değer (atılmış sekme) "son bilinen"
  olarak ayrı işaretleniyor.
- **Kazanç rakamı düzeldi.** Uyuyan bir sekme sıfır yer tutmuyor ve eskiden
  tuttuğu yer kazançtan düşülmüyordu. Artık hesap "uyanıkken tuttuğu −
  bugün tuttuğu".
- `olcumYaklasik` artık gerçekten bir ölçüm sonucu: eşleme tam **ve** her
  pasif sekme uyanıkken en az bir kez ölçülmüşse `false`.

Eşik kararı buna **dayanmıyor ve dayanmayacak** (`docs/Bellek.md`): sekme
başına rakam `SekmeOzeti` içine değil `BellekOzeti` içine kondu, tam da o
sızıntı olmasın diye.

**Kalan sınır:** eski bir WebView2 Runtime'da (Environment13 ya da
`ICoreWebView2_20` yoksa) eşleme kapalı ve panel eskisi gibi "yaklaşık"
diyor. Bu bir yetenek yokluğu, bir hata değil.

---

## Faz 3 — Kabuk

- Geçmiş + yer imleri (SQLite, geçişler, Türkçe küçültme).
- Adres çubuğu önerileri (yerel; ağ isteği yok).
- **Muiget köprüsü** — indirme devri, native messaging protokolü.
- **Muiply köprüsü** — yerel medya devri.
- Ayarlar ekranı (bellek profili görünür ve ayarlanabilir).
- Gizli pencere (ayrı profil klasörü).
- Veri temizleme ekranı.

### Faz 3 — durum

**Yarısı tamam.**

Yazıldı ve testleri geçiyor:

- `history/gecisler.rs` — şema + sürüm yükseltme. Adımlar tekrar
  çalıştırılabilir (yarıda kesilen geçiş) ve `surum` tablosu her adımdan
  önce var oluyor (CLAUDE.md #12).
- `history/depo.rs` — geçmiş, yer imleri, klasörler, budama. Aynı adrese
  ikinci kez gidildiğinde yeni satır açılmıyor, `sayac` artıyor: günde otuz
  kez açılan bir site geçmişi tek başına doldurmamalı.
- `history/mod.rs` — **Türkçe küçültme** (`turkce_kucult`). CLAUDE.md #10'un
  kod karşılığı ve testi var: `"İSTANBUL"` → `"istanbul"`, `"ISPARTA"` →
  `"ısparta"`. Rust'ın kendi `to_lowercase`i burada yanlış cevap veriyor
  (`'İ'` → `i` + birleşen nokta) ve test o farkı sabitliyor.
- Adres çubuğu önerileri — yalnız yerel geçmişten, **ağ isteği yok**
  (karar #5). Sorgu 120 ms geciktiriliyor.
- Geçmiş paneli (yan panelin üçüncü sekmesi) + aralıklı temizleme.
- Adres çubuğunda yer imi yıldızı.
- Ayarlar ekranı — bellek profili, eşikler, istisna alanları, süreç
  politikası. Düzeltilmiş değer gösteriliyor (CLAUDE.md #13) ve bayrak
  ayarlarında "yeniden başlatma gerekiyor" rozeti çıkıyor.

**Sonradan yazılanlar — hepsi tamam:**

- **Favicon deposu** (Faz 1'in 3. maddesiyle aynı iş). Yukarıda.
- **Muiget köprüsü** — `DownloadStarting` yakalanıyor, politikaya göre
  iptal edilip devrediliyor, `muiren://indirme-onerisi` yayınlanıyor.
  Muiget'in native host protokolü **olduğu gibi** konuşuluyor; karşı tarafta
  değişiklik gerekmedi (`docs/Kopruler.md`, "DOĞRULANDI").
  `IndirmePolitikasi`: `sor` (varsayılan) · `muiget` · `motorda`.
- **Muiply köprüsü** — yerel medya devri. `bridge::muiply::yerel_dosya` saf
  ve testli; uzak adres devredilmiyor çünkü Muiply ağ akışı oynatmıyor.
- **Gizli sekme** — gizli *pencere* değil. Muiren tek pencerede çalıştığı
  için karşılığı bir sekme (`Ctrl+Shift+N`). Ayrı veri klasörü **açılmadı**:
  InPrivate bir *denetleyici* seçeneği ve ortam seçeneklerine dokunmuyor, yani
  CLAUDE.md #16 kırılmıyor. Geçmiş, oturum ve favicon üç ayrı kapı olarak
  ayrıca kapatıldı — motorun gizliliği bizim depolarımızı bilmiyor.
- **Veri temizleme ekranı** — altı kalem: geçmiş, favicon, çerez, önbellek,
  site verisi, otomatik doldurma. Motorun profili
  (`ICoreWebView2Profile2::ClearBrowsingData`) ve Muiren'in kendi depoları
  **birlikte** siliniyor; ayrım kullanıcıya görünmüyor. Yer imleri hiçbir
  kalemde silinmiyor.

**Açık kalan (kod değil, deneme):**

- Muiget köprüsü gerçek bir Muiget kurulumuyla **uçtan uca denenmedi**.
  Protokol karşı tarafın kaynağından okundu ve mesaj biçimi testle
  sabitlendi, ama iki süreç arasındaki stdio turu makinede koşturulmadı.

---
## Faz 4 — Kişiselleştirme ve oyun

- `.muitema` biçimi, `theme/paket.rs` doğrulaması, yerleşik üç tema.
- Arka plan görseli, karartma, kontrast uyarısı.
- **Oyun modu** — kendi algılamamız (varsayılan kapalı) + Muifly köprüsü.
- İstek filtreleme altyapısı, kullanıcının koyduğu listeler.
- Pop-up ve yönlendirme engelleme (varsayılan açık).

### Faz 4 — durum

**Tema sistemi ve oyun modu tamam; engelleme açık.**

Yazıldı ve testleri geçiyor:

- `theme/jeton.rs` — **güvenlik çekirdeği**, saf. Renk yalnız
  `#rgb`/`#rrggbb`/`#rrggbbaa`, ölçü yalnız `0..=32px`. `url(`, `var(`,
  `expression(`, `image-set(`, adlandırılmış renkler, `rgb()`, `calc()`,
  kaçış ve yorum dizileri — hepsi reddediliyor ve her biri için ayrı bir
  test var. Yaklaşım **kara liste değil beyaz liste**: kara liste her yeni
  CSS özelliğinde güncellenmesi gereken bir borç.
- Çıktı **normalleştiriliyor**: girdi ne biçimde gelirse gelsin CSS'e giden
  dize `#rrggbbaa`. Enjeksiyon için bir yüzey kalmıyor.
- `theme/paket.rs` — `.muitema` (ZIP) okuma, doğrulama, kurma, dışa
  aktarma. Dizin taşması (`../`, mutlak yol, şema, alt dizin), zip bomb
  (sıkıştırılmış **ve** açılmış boyut sınırı) ve görsel olmayan dosyaların
  kopyalanması kapatılmış; her biri testli.
- Kontrast denetimi: tema **reddedilmiyor**, uyarı üretiliyor ve ayarlar
  ekranında görünüyor (`docs/Temalar.md`).
- Yerleşik üç tema — Mui, Mürekkep, Kağıt. Hepsi **aynı** doğrulamadan
  geçiyor; istisna yok.
- Arayüz: ayarlar ekranında tema seçici, `useTema` jetonları
  `documentElement.style` üzerine tek tek yazıyor. `<style>` enjeksiyonu ve
  `innerHTML` yok, ikinci bir süzgeç de yok (iki süzgeç, biri gevşediğinde
  fark edilmeyen bir açık demek).
- Oyun modu — `oyun_modu` komutu, gözcü baskıyı `Yuksek` sayıp uyanık
  sınırını 2'ye çekiyor. Oyun bitince sekmeler **kendiliğinden
  uyandırılmıyor**: 30 sekmenin birden yüklenmesi tam da kaçındığımız ani
  bellek sıçraması olurdu.

**Kontrast testi iki gerçek hata yakaladı** ve ikisi de düzeltildi:

- "Mürekkep" temasında `text-muted`/`bg-panel` 3.98:1 — eşiğin altında.
- "Kağıt" temasında `on-accent`/`accent` 3.74:1. Bu tam olarak
  `docs/Temalar.md` içinde not düşülen `on-accent` tuzağının açık tema hâli:
  beyaz yazı için aile teal'i (`#0d9488`) yeterince koyu değil. Hem tema
  hem `src/styles.css` içindeki açık tema bloğu düzeltildi — ikisi aynı
  değeri taşımak zorunda.

**Sonradan yazılanlar — hepsi tamam:**

- **Arka plan görseli.** Çizim yazıldı. `convertFileSrc` **kullanılmadı**:
  o yol Tauri `asset` protokolünü açmayı gerektiriyor ve bu, kabukta çalışan
  her satırın kullanıcının diskinden okuyabilmesi demek. Bunun karşılığı bir
  duvar kâğıdı değil. Görsel backend'de okunup `data:` adresi olarak
  gidiyor (`theme::arkaplan_veri`, 8 MB üst sınır), favicon deposuyla aynı
  gerekçe. Karartma **ayrı bir katman**: `blur` görselin kendisine
  uygulanıyor ve perdeyi de bulanıklaştırsaydı kenarlarda halka çıkardı.
- **Oyun modu kendi algılaması.** `bridge/oyun.rs` — karar saf ve testli,
  Win32 ölçümü ayrı (`memory/olcum.rs` ile aynı ayrım). Varsayılan kapalı.
  Muifly köprüsü **yok ve olmayacak**: bakıldı, `muifly://durum` süreç içi
  bir Tauri olayı ve süreçler arası bir uç bulunmuyor
  (`docs/Kopruler.md`, "DOĞRULANDI").
- **Pop-up ve yönlendirme engelleme** (varsayılan **açık**). Sinyal
  WebView2'nin kendi ayrımı (`IsUserInitiated`, `IsRedirected`); kendi
  sezgimizi yazmıyoruz. Kullanıcının **tıkladığı** bağlantı her zaman
  açılıyor — pop-up engelleyicinin kullanıcıyı engellemesi, özelliği
  kapattıran hata.
- **İstek filtreleme altyapısı** (varsayılan **kapalı**, kutudan çıkan liste
  yok — karar #5). `engel/liste.rs` saf ve testli; dört biçim tanınıyor
  (alan adı, `||alan^`, yol parçası, `@@` istisnası). Seçenekli ABP kuralları
  (`$third-party`) ve öğe gizleme (`##`) **uygulanmıyor** ve
  *anlaşılmayan* sayılıyor — kabul edip yok saymak, kullanıcının çalıştığını
  sandığı bir kuralla dolaşması demek.

  Süzgeç yalnız filtre açıkken kaydediliyor: `WebResourceRequested` her alt
  kaynağı kabuk sürecinden geçiriyor ve kapalı bir filtre için o bedeli
  ödemenin karşılığı yok. Bedeli: ayar değişince yalnız sonradan açılan ya da
  yenilenen sekmeler etkileniyor ve arayüz bunu rozetle söylüyor.

**Sonradan yazılan — teşhis paneli (2026-09-05).** `docs/Medya.md` üç yerde
tarif ediyordu ve `docs/IPC.md` kabuk sayfaları arasında sayıyordu, ama
**kodda yoktu**. Ayarlar → "Teşhis ekranını aç": GPU oluşturucu (yazılımsal
mı), codec başına donanım/yazılım, Widevine/PlayReady sorgusu, motor
yetenekleri ve `docs/olcumler/` başlık tablosunu panoya veren düğme.
Yorumlama saf ve testli (`src/lib/teshis.ts`), ölçüm ayrı
(`src/hooks/useTeshis.ts`) ve hiçbir soru sekmeye inmiyor (CLAUDE.md #5).

**Açık kalan (kod değil, deneme):**

- Oyun algılaması **gerçek bir oyunla denenmedi**. Karar saf ve testli ama
  `bridge/oyun.rs::win::on_plan` çıktısı (hangi oyunun hangi pencere stilini
  kullandığı) makinede ölçülmedi. Varsayılanın kapalı olması tam da bunun
  için.

---

## Faz 5 — Sosyal ve cila

- **Muiwatch köprüsü** — `nav_event` / `video_event` / `sync_heartbeat`.
  Muiwatch'ın kendi Faz 3'ü de buna bağlı; ikisi birlikte planlanacak.
- Dikey sekme çubuğu (yan panel).
- Sekme grupları, katlama, grup bazlı uyku eşiği.
- Sekme arama iyileştirmeleri.
- Video arka plan **değerlendirmesi** — ölçülmüş maliyeti yazılmadan eklenmiyor
  (`docs/Temalar.md`).

### Faz 5 — durum

**Muiren tarafı tamam.**

- Dikey sekme çubuğu — Faz 1'de yazılmıştı.
- **Sekme grupları, katlama, grup bazlı uyku eşiği.** `tabs/grup.rs` saf ve
  testli; renk bir **enum** (kullanıcının yazdığı dize CSS'e gitmiyor —
  `theme/jeton.rs` ile aynı gerekçe), ad `baslik_temizle` kalıbından
  geçiyor, eşik `settings::duzelt` kalıbından.

  Grup eşiği `esik.rs`e **grup tablosu olarak değil**, sekme özetinde hazır
  bir değer olarak giriyor (`SekmeOzeti::grup_uyku_esigi_sn`): o dosya saf
  kalmak zorunda. Eşik atma eşiğini de ölçekliyor — gerekçesi
  `docs/IPC.md`, "Komutlar — Gruplar".

  Gruplar oturum dosyasında sekmelerle **aynı yerde** duruyor: ayrı bir
  dosyaya yazılsalardı yarım kalmış bir yazımda ayrışabilirlerdi (grubu olan
  ama grubu olmayan sekmeler).

- **Muiwatch köprüsü** — Muiren tarafı yazıldı, karşı uçta bugün alacak bir
  kapı yok (`docs/Kopruler.md`, "DURUM (Faz 5)"). Yazılan asıl iş **koruma
  kuralı #7**: bağlı sekme uyutulmuyor ve atılmıyor. O kural protokolden
  bağımsız ve bugün test ediliyor.

**Açık kalanlar:**

- **Sekme arama iyileştirmeleri** — arama bugün `lib/suz.ts` üzerinden başlık
  ve adreste eşleşiyor. "İyileştirme"nin ne olduğu yazılmadı; ölçülecek bir
  şikâyet olmadan genişletilmiyor.
- **Video arka plan** — değerlendirme. `docs/Temalar.md` ölçülmüş maliyet
  yazılmadan eklenmemesini şart koşuyor ve o ölçüm yapılmadı.

---

## Faz 6 — Karar verilmemiş

Bunlar "yapılacak" değil, "tartışılacak". Her biri ayrı bir karar kaydı
gerektiriyor.

- **Uzantılar.** WebView2 yalnız paketlenmemiş uzantı yükleyebiliyor; uBlock
  Origin çalışır mı, MV3 kısıtları ne durumda? Kullanıcıların en çok isteyeceği
  şey bu ve en zor cevaplanacak olan da bu.
- **Linux / macOS.** WebView2 Windows'a bağlı. Taşıma, motoru CEF ya da
  WebKitGTK'ya çevirmek demek.
- **Profil desteği** (iş/kişisel ayrı çerez alanı). Her profil ayrı browser
  süreci = bellek maliyeti. Tezle gerilim var.
- **Senkronizasyon.** Şu an kapsam dışı ve kalması muhtemel; istenirse
  hesapsız/sunucusuz bir yol (dosya tabanlı dışa/içe aktarma) tercih edilir.

---

## Kapsam Dışı (kalıcı)

`CLAUDE.md` içindeki liste kanonik. Özet:

kendi render motoru · Chrome Web Store · şifre yöneticisi · hesap/bulut
senkronizasyonu · JavaScript çalıştıran tema · kendi indirme motoru · mobil.

Bunlardan birini yapmaya karar verilirse **önce bu dosya güncellenir**, sonra
kod yazılır.

---

## Karar Kayıtları

Kardeş projelerde (`Muiget/docs/decisions.md`) olduğu gibi, geri döndürülmesi
pahalı kararlar numaralanarak buraya eklenecek. Şimdilik başlangıç kararları:

**#1 — Motor: WebView2, soyutlama arkasında.**
Aile stack'iyle uyumlu, küçük dağıtım, Win11'de runtime hazır. Bedeli:
Windows'a bağlılık ve `unstable` çoklu webview API'si. `motor/` soyutlaması
CEF'e geçişi tek dosyaya indiriyor. Karar Faz 0/R2'ye bağlı olarak gözden
geçirilebilir.

**#2 — Sekme bir veri kaydı, webview onun eki.**
"Çok sekmede az RAM" iddiası ancak sekme sayısı ile webview sayısı ayrıldığında
mümkün. Mimarinin tamamı bu ayrımın üstünde duruyor.

**#3 — Tema veri, kod değil.**
Tarayıcı arayüzünde çalışan kod, açılan her sayfayı görebilir. İndirilen tema
dosyasına bu yetkiyi vermemek için jeton beyaz listesi + doğrulama. Bedeli:
tema yazarları sınırlı.

**#4 — İndirme Muiget'e devrediliyor, kendi motorumuz yok.**
Muiget'in native messaging protokolü hazır ve test edilmiş; ikinci bir indirme
motoru yazmak portföyde tekrar demek. Muiget kurulu değilse motorun kendi
indirmesi devrede kalıyor.

**#5 — Telemetri, hesap ve bulut yok.**
Hem gizlilik hem bellek gerekçesi: senkronizasyon servisi kalıcı süreç ve
kalıcı bellek demek. Filtre listeleri bile kullanıcı tarafından konuyor.

**#6 — "Daha az RAM yiyen bir motor" araştırıldı; kazanç motorda değil
süreç politikasında.**

*Bağlam:* Chromium'un bellek tüketimi projenin var oluş sebebi olduğuna göre,
daha hafif bir motorla başlamak akla gelen ilk çözüm.

*İncelenenler:*

| Aday | Windows'ta gömülebilir | Bugünün webini açıyor | Sonuç |
|---|---|---|---|
| Blink / WebView2 | evet, runtime hazır | evet | **seçili** |
| Blink / CEF | evet, ~150 MB | evet | R2/R4'ün B planı |
| Gecko (Firefox motoru) | **hayır** — Mozilla gömme API'sini kaldırdı; GeckoView yalnız Android | evet | elendi |
| Firefox çatallaması | gömme değil, fork | evet | başka bir proje (aşağıda) |
| Servo | kısmen, gömülebilirlik hedefi var | **hayır** | izlenecek |
| Ladybird | hayır | hayır | izlenecek |
| Ultralight / Sciter | evet | hayır — uygulama arayüzü motoru | elendi |
| WebKit (WinCairo) | üretim için bakımlı değil | — | elendi |

*Karar:* Motor değişmiyor. Windows'ta bugünün webini açan iki motor var ve az
yiyen olan Gecko gömülemiyor. Servo/Ladybird'ün düşük tüketimi kısmen webin
zor kısımlarını henüz uygulamamış olmalarından geliyor.

*Asıl bulgu:* Firefox'un çok sekmede avantajı motor kalitesinden değil, içerik
süreçlerini sınırlı bir havuzda paylaştırmasından geliyor. Bu bir **politika**
ve Chromium üzerinde kurulabiliyor (`--process-per-site`,
`--renderer-process-limit`). Dolayısıyla motor değiştirmeden alınabilecek bir
kazanç; `docs/Bellek.md` içine 2. kazanç kaynağı olarak eklendi, geçerliliği
R4'e bağlı.

*Firefox çatallaması neden değil:* Zen/Floorp/LibreWolf emsali var ve teknik
olarak mümkün. Ama fork demek Tauri, Rust kabuk ve React arayüzün tamamının
düşmesi, arayüzün Firefox'un kendi katmanında yazılması ve Mozilla'nın her
sürümünde rebase edilen kalıcı bir yama seti demek. Ayrı bir proje olarak
değerlendirilebilir; Muiren olarak değil.

*Yeniden açılma koşulu:* Servo günlük kullanılabilir web uyumluluğuna
ulaşırsa. O noktada `motor/` soyutlaması sayesinde değerlendirme tek modülle
sınırlı kalır.

**#7 — Oyun modu pencereyi gizlemiyor, simge durumuna alıyor.**

*Bağlam:* `docs/Bellek.md` oyun modunun 4. adımını "kabuk penceresini gizler"
diye yazmıştı ve madde uzun süre **yazılmadan** durdu — kullanıcının
penceresini kendiliğinden yok etmek geri alınması zor bir davranış.

*Karar:* Adım yazıldı ama `hide()` değil `minimize()` ile, ayrı bir ayarın
(`oyun_pencere_gizle`) arkasında ve **varsayılan kapalı**.

*Gerekçe:* Gizlenen pencerenin görev çubuğu düğmesi de kayboluyor. Tam ekran
bir oyun ekranı kaplarken kullanıcının tarayıcıya dönecek görünür bir yolu
kalmıyor ve "tarayıcım kayboldu" diye rapor edilen bir hata, kazandığı
bellekten çok daha pahalı. Simge durumundaki pencere her an geri geliyor ve
kazanç aynı: pencere kompozisyona girmiyor, Chromium örtülü pencerenin render
süreçlerini geri plana atıyor. Ayrıca var olan "pencere gizli" baskı kuralı
(`esik::BellekBaskisi::etkin`) bu durumu **zaten** biliyor, yani 4. adım 1.
adımı ikinci bir yoldan doğruluyor.

*Bedeli:* Kullanıcı oyun modunu elle açtığında da pencere küçülüyor
(ayar açıksa). Geri getirme bize ait ve garantili: bayrak + gözcünün her
turunda tekrarlanan denetim.

**#8 — Ölçüm otomasyonu bir mod, özellik değil.**

*Bağlam:* Üç fazın kapanması `docs/olcumler/` altına yazılacak rapora bağlı ve
ölçüm elle koşturulduğu için hiç yapılmamıştı.

*Karar:* Ölçüm modu yazıldı (`src-tauri/src/olcum/`), ama **ortam
değişkeniyle** (`MUIREN_OLCUM`) — IPC komutu, ayarlar seçeneği ya da düğmesi
yok.

*Gerekçe:* Elli sekme açan bir düğme yanlışlıkla tıklanabilir; ölçüm aracının
kullanıcıya sunulan bir özellik hâline gelmesi, kabuk yüzeyine bakım borcu
eklemek demekti. Değişken tanımlı değilken modülün tek satırı çalışmıyor.

*Sınırı:* Chrome'u ölçmüyor, sekme kapatmıyor, uyanma gecikmesi üretmiyor.
Üçü de raporun kendi içinde yazılı — üretilen dosyanın eksiğini söylemesi,
okuyanın eksiği bulmasından iyi.
