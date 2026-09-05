# Arayüz

## Kural sırası

1. Arayüzde **iş mantığı yok** (CLAUDE.md #2). Sekme sırası, uyutma kararı,
   köprü seçimi backend'de. Arayüz gelen olayı çiziyor.
2. Bileşenler `invoke` görmüyor — `src/ipc/` sarmalayıcıları görüyor.
3. Durum kütüphanesi yok. Backend durumun tek kaynağı; `src/hooks/` onun
   yansıması.
4. Renk/ölçü sabiti yazılmıyor — hepsi jetonlardan.

## Tasarım dili

Kanonik kaynak `..\Muiget\src\styles.css`, jeton listesi
`..\MuiLabs\docs\ui-conventions.md`. Tahmin yok, kopyala:

```css
--bg: #0f1115;          --bg-panel: #181b22;    --bg-elevated: #20242d;
--bg-sunken: #0b0d11;   --border: #262b35;      --border-strong: #333a47;
--text: #e8eaed;        --text-muted: #8b93a3;
--accent: #2dd4bf;      --accent-strong: #5eead4;
--accent-soft: rgb(45 212 191 / 12%);   --accent-line: rgb(45 212 191 / 34%);
--on-accent: #04211d;   /* teal AÇIK bir renk: üstündeki yazı koyu olmak zorunda */
--radius: 12px;         --radius-lg: 16px;      --radius-pill: 999px;
```

Yazı tipi gömülü Outfit. Sınıf adları Türkçe: `.dugme`, `.kart`, `.rozet`,
`.sekme`, `.adres`.

Bu jetonların bir kısmını kullanıcı teması değiştirebiliyor — hangileri ve
sınırları `docs/Temalar.md` içinde. Arayüz kodu her zaman `var(--...)`
okuyacak, sabit yazmayacak; tema o değişkeni değiştirdiğinde her yer birlikte
değişsin.

## Yerleşim

```
┌───────────────────────────────────────────────────────────┐
│ ▣ [sekme][sekme][sekme]…            ⌕  ◧  ⚙   – ▫ ✕      │ sekme şeridi
├───────────────────────────────────────────────────────────┤
│ ← → ⟳   [ 🔒 adres / arama                    ]  ↓ ☆ ⋮   │ gezinme çubuğu
├───────────────────────────────────────────────────────────┤
│                                                           │
│                     w e b v i e w                         │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

Kabuk kasıtlı olarak ince: iki satır. Ekranın geri kalanı sayfanın.

Yan panel (`◧`) üç şey barındırıyor, sekmeli: **dikey sekme listesi**,
**bellek paneli**, **geçmiş**. Kapalıyken hiç çizilmiyor — bileşen
oluşturulmuyor bile: görünmeyen bir panelde 50 sekmelik listeyi tutmanın
maliyeti tam da kaçındığımız türden.

Panel açılınca `.icerik` daralıyor ve `useIcerikAlani` yeni dikdörtgeni
backend'e bildiriyor; sekme webview'i panelin altında kalmıyor. Backend sabit
bir yükseklik tutsaydı bu mümkün olmazdı (`docs/IPC.md`, `icerik_alani`).

Pencere **kenarlıksız**: başlık çubuğunun işini sekme şeridi görüyor. Şeridin
boş alanı `data-tauri-drag-region` taşıyor (sürükleme, çift tıkla büyütme ve
Aero Snap oradan sürüyor), pencere düğmeleri şeridin sağ ucunda. Sebep yer:
ayrı bir başlık satırı, ekranın tepesinde sayfaya ait olması gereken 32
pikseli alıyor.

## Sekme şeridi

Ayrıntılı davranış `docs/Sekmeler.md` içinde.

### Durum halkası — tek görsel dil

Sekmenin bellek durumu **favicon'un çevresindeki halkada**
(`src/components/SekmeIkonu.tsx`):

| Durum | Halka | İkon |
|---|---|---|
| `Etkin` | `--accent`, dolu çizgi | tam |
| `Arkaplan` | yok | tam |
| `Uyuyan` | `--paused`, dolu çizgi | opaklık 0.55 |
| `Atilmis` | `--border-strong`, **kesikli** | opaklık 0.35 + gri tonlama |

Durumun ayrı bir nokta olarak değil halka olarak çizilmesinin sebebi **yer**:
44 piksele inmiş bir sekmede favicon dışında hiçbir şey görünmüyor. Durum ayrı
bir nokta olsaydı tam da en çok sekme açıkken — yani durumu bilmenin en çok işe
yaradığı anda — kaybolurdu. Favicon zaten "bu hangi site" sorusunun cevabı;
halkası "ve şu an bellekte mi" sorusununki.

Aynı bileşen **üç yerde** kullanılıyor: yatay şerit, dikey liste ve sekme
araması. İki yerde iki ayrı çizim, kullanıcının öğrendiği göstergeyi ikinci
panelde yeniden öğrenmesi demek olurdu.

Favicon'u olmayan sekme, adının ilk harfini alıyor (Türkçe büyültmeyle —
CLAUDE.md #10'un simetriği). Boş bir kare, dar sekmeyi tamamen kimliksiz
yapıyordu.

Solmayı **animasyonla** yapmıyoruz; 50 sekmede aynı anda 30 geçiş animasyonu
tam da kaçındığımız türden bir maliyet. Durum değişimi anında. Tek istisna
fare imlecinin altındaki sekmenin zemini: toplu bir maliyeti yok.

### Genişlik ve yoğunluk kademeleri

Sekme 240 px'ten başlıyor, sıkışınca 44 px'e kadar daralıyor (durum halkalı
favicon + ses göstergesi), sonra şerit kaydırmaya geçiyor. Daha fazla daralma
yok — 12 piksellik sekme kimseye yaramıyor.

**Etkin sekmenin alt sınırı ayrı ve yüksek** (`--sekme-etkin-min`, 152 px).
Sebebi: 14 sekmede bile hepsi alt sınıra iniyordu ve kullanıcının **şu an
baktığı** sayfanın adı bile görünmüyordu. Daralması gereken zaten diğerleri.

İçindekilerin elenmesi sekmenin **kendi** genişliğine bakıyor (CSS
`@container`), pencerenin genişliğine değil. Eski kural `@media (max-width:
900px)` idi ve yanlıştı: 1600 piksellik bir pencerede 40 sekme açan
kullanıcının sekmeleri de alt sınıra iniyor, ama kural "pencere geniş" diye
kapatma düğmesini çizmeye devam ediyordu.

Etkin sekme her zaman **görünür** tutuluyor (`scrollIntoView`, yumuşak
değil): kısayolla gezinen ya da yeni sekme açan kullanıcı, şerit kaydırmaya
geçtiğinde açtığı sekmeyi göremiyordu.

Ses göstergesi sekmenin sağında ve **tıklanabilir** (sessize alır). Sessize
alınan sekme koruma kaybediyor (`docs/Bellek.md`).

### Gizli sekme ve gruplar (Faz 3–5)

- **Gizli sekme** ayrı çiziliyor: maske ikonu + `--accent-soft` zemin +
  `--accent-line` kenarlık. Bu bir süs değil: kullanıcının hangi sekmenin
  gizli olduğunu **görmesi** gerekiyor, yoksa gizli sandığı sekmede geçmişe
  yazan bir arama yapıyor.
- **Grup** sekmenin sol kenarında 3 piksellik bir şerit (`inset` gölge,
  `border-left` değil: kalın bir kenarlık kapsülün yuvarlak köşesini kesip
  sekmeyi eğri gösteriyordu). Renk bir enum'dan geliyor (`tabs::GrupRengi`),
  kullanıcının yazdığı bir dizeden değil — tema jetonlarındaki beyaz liste
  kuralıyla aynı gerekçe.
- **Grup başlığı** yalnız grubun **ilk** sekmesinin önünde. Sıra backend'den
  geldiği gibi kullanılıyor: bir grubun sekmeleri bitişik olmayabiliyor
  (kullanıcı araya başka bir sekme sürüklemiş olabilir) ve o durumda başlık
  ikinci kez çıkıyor. Bunu arayüzde "düzeltmek", backend'in sırasıyla
  ayrışmak olurdu.
- **Katlanmış grubun** sekmeleri çizilmiyor ama backend listeyi yine de tam
  gönderiyor: katlama bir görüntü tercihi ve "grubu aç" düğmesinin kaç sekme
  olduğunu söylemesi gerekiyor.
- **Sürükle-bırak ile gruplama yok.** Şeritteki sürükleme zaten sırayı
  taşıyor; ikisini aynı jeste yüklemek, kullanıcının yanlışlıkla grup
  değiştirmesi demek. Gruplama yan paneldeki grup sekmesinden.

## Adres çubuğu

`src/lib/url.ts` — saf, testli, iki soruya cevap veriyor:

- Kullanıcının yazdığı şey URL mi arama mı? (`ornek.com` URL, `ornek com` arama,
  `localhost:3000` URL, `?bir şey` zorla arama)
- Gösterilecek biçim ne? (şema gizlenir, alan adı vurgulanır, geri kalanı
  soluk)

Birinci sorunun cevabı burada **yalnız çizim için**: kilit simgesi ve vurgu
kullanıcı yazarken anında güncellenmek zorunda. Motora ne gideceğine karar
veren yer backend (`tabs::adres_mi`, `docs/IPC.md`) — adres çubuğu tek kapı
değil ve kabuk kapalıyken de doğru çalışması gerekiyor.

**Alan adı vurgusu bir güvenlik özelliği.** `https://banka.com.saldirgan.net/`
adresinde vurgulanan `saldirgan.net` olacak. Uluslararasılaştırılmış alan
adlarında (IDN) karışabilecek karakterler için punycode gösterimi — bu mantık
`url.ts` içinde ve test kapsamı en geniş olan yer.

Öneri listesi: geçmiş + yer imleri + açık sekmeler. **Ağ isteği yok** — arama
motorunun canlı öneri API'si telemetri demek, kullanılmıyor.

## Nabız — gezinme çubuğundaki bellek göstergesi

`src/components/Nabiz.tsx`. Üç renkli kısa bir çubuk (uyanık / uyuyan /
atılmış) ve tek bir rakam; tıklanınca bellek paneli açılıyor.

Var olma sebebi: **iddia saklıydı.** Muiren'in tek iddiası bellek ve o iddiayı
görmek için `Ctrl+Shift+M` ile bir panel açmak gerekiyordu. Panelin kendisi
doğru bir yer ama kapalı bir çekmecede duruyordu; kullanıcı politikanın
çalıştığını hiç görmeden aylarca kullanabiliyordu.

Kurallar bellek paneliyle aynı:

- Hiçbir sekmeye dokunmuyor (CLAUDE.md #5); tek kaynağı `bellek_ozeti` olayı.
- `olcum_yaklasik` true ise rakamın başında `~` var; biçimleme `lib/bicim.ts`
  içinde ve panelle **aynı fonksiyon** — iki yerde iki farklı sayı görünmesin.
- Özet **yokken çizilmiyor**. Gözcü ilk turunu koşmadan sıfır göstermek,
  ölçümün yokluğunu bir ölçüm sonucu gibi sunmak olurdu.
- Atılmış sekmenin dilimi dolu değil, kesikli — sekme şeridindeki kesikli
  halkayla aynı dil.

Gösterge sürekli göründüğü için gözcünün **ilk turu öne alındı**
(`memory/gozcu.rs`, `ILK_BEKLEME_SN`): döngü önce uyuyup sonra ölçtüğü için
ilk özet bir periyot sonra (varsayılan 20 sn) çıkıyordu ve o süre boyunca
çubuğun yeri boş kalıyordu.

## Bellek paneli

Projenin vitrini. Yan panelde, `bellek_ozeti` olayıyla besleniyor:

- Üstte tek büyük rakam: **"~3.2 GB tasarruf"** (`tahmini_kazanc_mb`).
  `olcum_yaklasik` true ise `~` ve ipucu.
- Altında durum dağılımı: `14 uyanık · 22 uyuyan · 9 atılmış`.
- Sistem baskısı çubuğu, `--accent`ten kırmızıya.
- "Hepsini uyut" düğmesi.
- Sekme listesi, belleğe göre sıralı; uyuyanların yanında "uykuda" rozeti ve
  son bilinen değeri.

Rozet renkleri MuiLabs status badge konvansiyonundan.

> Bu panel uyuyan sekmeleri **uyandırmadan** çiziliyor (CLAUDE.md #5). Panelin
> açık olması bellek tüketimini artırmamalı — yoksa bellek panelinin kendisi
> ironik biçimde bellek harcayan şey olur.

## Tam ekran örtüler ve z-sırası — CSS'in çözemediği yer

Sekme webview'i **ayrı bir native pencere** ve kabuğun **üstünde** duruyor
(`lib.rs`, pencere düzeni). Bunun doğrudan bir sonucu var ve gözle
görülmüyor:

> Kabuğun çizdiği tam ekran bir örtü (ayarlar ekranı, sekme arama), açık bir
> sayfa varken **sayfanın altında kalıyor.** `z-index` işe yaramıyor — iki
> ayrı pencerenin sırası CSS'in görebildiği bir şey değil.

Çözüm: örtü açılırken sekme webview'i **gizleniyor** (`ortu_gorunur` komutu),
kapanırken aynı alana geri geliyor. Gizlemek uyutmak değil: sekme durumu ve
sayfanın kendisi olduğu gibi duruyor.

Karar yine backend'de: arayüz yalnız "örtü açık" diye bir **ölçüm**
bildiriyor, sayfayı gizleyip gösteren `tabs/surucu.rs`. `goster` çağrıları da
bu bayrağa bakıyor — yoksa örtü açıkken gelen bir gezinme sayfayı örtünün
üstüne geri getirirdi.

**Bilinen sınır:** adres çubuğu öneri listesi bu kapıdan geçmiyor. Öneriler
kullanıcı yazarken açılıyor ve her tuşta sayfayı gizleyip göstermek,
düzeltmeye çalıştığından daha rahatsız edici bir titreme üretirdi. Liste
içerik alanının üst şeridine taştığında orada sayfanın altında kalıyor.

**Yeni bir tam ekran örtü eklenirse** `ortuAcik` ifadesine eklenmesi
gerekiyor (`App.tsx`), yoksa sessizce görünmez oluyor.

Bugün üç örtü var: **ayarlar**, **sekme arama** ve **teşhis**
(`docs/Medya.md`). Üçü aynı `.ortu` kabuğunu paylaşıyor ve **üst üste
açılmıyorlar**: teşhis ekranı ayarlardan açılırken ayarlar kapanıyor. İki
örtüyü üst üste açmak, zaten gizli olan sayfayı ikinci kez gizlemek ve
kullanıcıya "hangisini kapattım" sorusunu sordurmak olurdu.

## Kısayollar

| Kısayol | İş |
|---|---|
| `Ctrl+T` / `Ctrl+W` | yeni sekme / kapat |
| `Ctrl+Shift+T` | son kapatılanı geri al |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | sonraki / önceki sekme |
| `Ctrl+1..8` / `Ctrl+9` | n. sekme / son sekme |
| `Ctrl+L` | adres çubuğuna odaklan |
| `Ctrl+Shift+A` | sekme arama |
| `Ctrl+Shift+M` | bellek paneli |
| `Ctrl+Shift+Z` | bu sekmeyi uyut |
| `Ctrl+Alt+Z` | hepsini uyut |
| `F11` | tam ekran |
| `Ctrl+Shift+N` | gizli sekme |

`Ctrl+Shift+E` yan paneli açıp kapatıyor (tabloda yoktu, eklendi).

`Ctrl+Shift+N` tabloda "gizli pencere" yazıyordu; karşılığı bir **sekme**.
Muiren tek pencerede çalışıyor (`lib.rs` pencere düzeni) ve ikinci bir pencere
açmak mimarinin dışında. Kas hafızası aynı tuşta kalıyor, sonuç mimariye
uyuyor. Shift'siz `Ctrl+N` bilerek **boşta**: Chrome'da yeni pencere açıyor ve
sessizce yeni sekme açmak, pencere bekleyen kullanıcıya sekme vermek olurdu.

> **Kritik (Muiply'dan devralınan ders):** webview odaktayken tuşlar kabuğa
> ulaşmıyor. Her kısayolun **iki giriş kapısı** olmak zorunda: React
> tarafındaki dinleyici ve `motor/gercek.rs` içindeki hızlandırıcı kaydı.
> Biri unutulursa kısayol sayfa üzerindeyken sessizce çalışmıyor ve bu
> "bazen çalışmıyor" diye rapor ediliyor.

### Tablo tek yerde — iki kapı, tek karar

İki kapı **iki tablo demek değil.** İki ayrı tablo yazılsaydı zamanla
sessizce ayrışırlardı ve yukarıdaki hata bu kez fark edilmesi daha zor bir
biçimde geri gelirdi. Bu yüzden:

```text
  React keydown  ──┐
                   ├──► kisayol::coz()  ──► Surucu::kisayol_uygula()
  AcceleratorKey ──┘     (saf, testli)
```

- Tablo: `src-tauri/src/kisayol.rs`. Saf — girdi tuş + değiştirici, çıktı bir
  eylem. Sistem çağrısı, motor çağrısı, durum yok.
- Kabuk odaktayken React `kisayol_bas` komutunu çağırıyor. Kendi tablosunu
  **tutmuyor**; yalnız "bu kombinasyonun tarayıcı varsayılanını bastır" diye
  bir yüklem var, o da hangi tuşun ne yaptığını bilmiyor.
- Sayfa odaktayken `ICoreWebView2Controller::add_AcceleratorKeyPressed`
  aynı fonksiyonu çağırıyor. Sanal tuş kodu → tuş adı çevirisi de tek yerde
  (`kisayol::vk_adi`), ve bir test iki yolun aynı sonucu verdiğini
  sabitliyor.
- Bir kısmı **arayüz işi** (adres çubuğuna odaklan, paneli aç, tam ekran).
  Backend onları yapamıyor; `muiren://kisayol` olayıyla kabuğa gönderiyor ve
  `App.tsx` içinde **tek** yerde uygulanıyorlar — hangi kapıdan geldiğinden
  bağımsız.

Yeni kısayol eklerken dokunulacak tek yer `kisayol.rs`. Arayüz işiyse
`Kisayol::arayuz_isi()` içine ve `App.tsx`in `arayuzKisayolu` anahtarına
girmesi yetiyor.

`Ctrl+Alt` kolu kasıtlı olarak dar tutuluyor: `AltGr` Windows'ta Ctrl+Alt
olarak görünüyor ve Türkçe klavyede karakter üretiyor. Genişletilirse
kullanıcı yazarken kısayol tetikler.

## Kancalar (`src/hooks/`)

- `useSekmeler()` — `muiren://sekme-degisti` + `sekme-guncellendi` dinliyor.
- `useBellek()` — `muiren://bellek-ozeti`. İlk değer `null` olabiliyor: gözcü
  henüz turunu koşmamış demek. Panel o durumda "ölçülüyor" diyor — sıfır
  göstermek yanlış olurdu, çünkü sıfır bir ölçüm sonucu değil ölçümün
  yokluğu.
- `useGecikme()` — uyanma gecikmesi dağılımı. **Kendi olayı yok**: gecikme
  kullanıcı sekme değiştirdikçe değişiyor ve her uyanma için ayrı bir yayın
  eklemek, sekme değiştirmenin sıcak yoluna bir iş daha koymak olurdu. Onun
  yerine gözcünün turuna binildi — `muiren://bellek-ozeti` zaten gelen tek
  düzenli nabız ve ikinci bir zamanlayıcı, bellek iddiası olan bir programda
  karşılığı olmayan bir uyandırma demekti. Bedeli panelin gecikmeyi bir tur
  geriden göstermesi; bu sayı anlık bir gösterge değil, bir ölçüm oturumunun
  sonucu. Kanca `BellekPaneli` içinde çağrılıyor (`GecmisPaneli` kalıbı):
  panel kapalıyken hiç sorulmuyor.
- `useOneriler()` — adres çubuğu önerileri, **yalnız yerel geçmişten**.
  Sorgu 120 ms geciktiriliyor: her tuşta bir IPC turu + SQLite sorgusunun
  karşılığı yok.
- `useYerImi()` — etkin adresin yer imlerinde olup olmadığı + ekle/çıkar.
- `useTema()` — `muiren://tema-degisti`, jetonları `documentElement.style`
  üzerine yazıyor. **`innerHTML` ya da `<style>` enjeksiyonu yok.**
- `useKopruler()` — `muiren://kopru-durumu`; kurulu olmayan kardeşin menü
  öğesi hiç çizilmiyor.
- `useFavicon()` — kimlik → `data:` adresi. Önbellek **modül düzeyinde**,
  bileşen düzeyinde değil: sekme listesi saniyede birkaç kez yeniden
  çizilebiliyor ve okunmuş ikonların kaybolmaması gerekiyor. Kimlik içeriğin
  karması olduğu için önbellek hiç geçersizleşmiyor.
- `useIndirme()` — `muiren://indirme-onerisi`. Öneri **kaybolmuyor**, sonuç
  bildirimi kayboluyor: bildirimi kaçırmak bir şey kaybettirmiyor, öneriyi
  kaçırmak indirmeyi kaybettiriyor.
- `useEngel()` — `muiren://engellendi` + sekme değişince sayacı bir kez
  okuyor. Olaylar yalnız ileriye doğru geliyor ve sekme değiştiren kullanıcı
  o sekmede daha önce engellenenleri de görmeli.
- `useGruplar()` — grup listesi. **Ayrı bir olay yok**: sekmenin `katli` ve
  `grupUykuEsigiSn` alanları gruptan türetiliyor ve grup değişince backend
  zaten `sekme-degisti` yayınlıyor. İkinci bir olay, ikisinin sessizce
  ayrışabildiği bir yol açardı.

Her kanca aynı iskelet: ilk değeri komutla çek, sonra olayla güncelle,
`unlisten` ile temizle.

## Erişilebilirlik ve pencere

- **Sekme listelerinde dolaşan odak** (roving tabindex). `role="tablist"`
  içindeki her sekmeye `tabIndex=0` vermek, 50 sekmelik bir şeritte Tab
  tuşuna 50 kez basmak demekti: klavyeyle adres çubuğuna geçmek pratikte
  imkânsızdı. Şerit tek bir durak; içinde ok tuşlarıyla geziliyor, `Home`/
  `End` uçlara gidiyor, `Delete` sekmeyi kapatıyor.

  > Ok tuşları sekmeyi **etkinleştirmiyor**, yalnız odağı taşıyor
  > (WAI-ARIA'nın "elle etkinleştirme" kalıbı). Bu tercih burada mecburi:
  > otomatik etkinleştirme, uyuyan bir sekmenin üstünden geçerken onu
  > uyandırırdı ve klavyeyle şeridi taramak projenin tek iddiasını
  > çökertirdi (CLAUDE.md #5). Aynısı dikey liste için de geçerli.

- **Escape açık örtüyü kapatıyor** ve backend'e gitmiyor (`App.tsx`,
  `ortuKapat`). Backend tablosunda Escape'in karşılığı `Durdur`
  (`src-tauri/src/kisayol.rs`) ve ikisi aynı tuşta çakışıyordu: ayarlar
  ekranındayken Escape sayfanın yüklenmesini durduruyor, ekran açık
  kalıyordu. Ayrım kabukta veriliyor çünkü "hangi örtü üstte" yalnız kabuğun
  bildiği bir şey.
- Odak halkası `--accent`; her etkileşimli öğede `:focus-visible`.
- `prefers-reduced-motion` isteğine uyuluyor: kalan küçük geçişler de
  kapanıyor. Sekme durumu zaten animasyonsuz.
- Pencere kenarlıksız (özel başlık çubuğu) ama sürükleme alanı ve pencere
  düğmeleri Windows davranışına uyuyor — çift tıkla büyüt, kenara sürükle
  yasla dahil.
- Tam ekranda kabuk tamamen çekiliyor (`docs/Medya.md`).

## Geliştirme: kabuğu tarayıcıda açmak

Arayüz hatasını ayıklamanın en ucuz yolu kabuğu düz bir tarayıcıda açmak —
`PencereDugmeleri` bunu bilerek mümkün kılıyor (Tauri bağlamı yokken
fırlatmıyor). Ama Tauri bağlamı olmadan her komut düşüyor ve ekranda boş bir
şerit kalıyordu: görülecek bir şey yok.

`src/dev/sahte.ts` o boşluğu dolduruyor. `window.__TAURI_INTERNALS__` yerine
geçip komutlara bellek içi bir defterden cevap veriyor; sekmeler açılıyor,
kapanıyor, gezinme oluyor, bellek özeti akıyor.

```
npm run dev  →  http://localhost:1420/?sahte
```

**Bu bir test değil ve bir sözleşme değil.** Kararların doğruluğu Rust
tarafında test ediliyor; buradaki mantık yalnız ekranda gerçekçi bir tablo
çizmek için var. `docs/IPC.md` ile ayrışırsa hata sahte tarafındadır.

Üretim paketine **girmiyor**: çağıran yer `import.meta.env.DEV` ile korunuyor
ve dinamik içe aktarım derlemede eleniyor (`npm run build` çıktısında `sahte`
dizesi hiç geçmiyor).

## Test

`vitest` ile saf modüller: `lib/url.ts`, `lib/suz.ts`, `lib/bicim.ts`.
Bileşen testi yazılmıyor — arayüzde mantık olmadığı için test edilecek karar
da yok. Karar testleri Rust tarafında (`tabs/agac.rs`, `memory/esik.rs`).
