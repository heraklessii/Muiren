# Bellek — Projenin Kalbi

Muiren'in var olma sebebi bu dosya. Başka her şey (tema, köprü, geçmiş) bu
iddia doğruysa anlamlı.

## Önce dürüst olalım: RAM nereye gidiyor

"Tarayıcılar çok RAM yiyor" doğru ama sebebi çoğu insanın sandığı gibi
"şişkinlik" değil. Chromium'da bir sekmenin maliyeti kabaca şu:

| Kalem | Yaklaşık | Kimin |
|---|---|---|
| Render süreci taban maliyeti | 30–60 MB | motorun |
| Sayfanın JS yığını | 10 MB – 1 GB+ | sitenin |
| DOM + stil + layout ağacı | 5–100 MB | sitenin |
| Compositor katmanları / doku | 10–200 MB | motorun + GPU |
| Site izolasyonu (iframe başına ek süreç) | 20–40 MB × n | motorun |

Buradaki kritik gerçek: **biz de aynı motoru kullanıyoruz.** Muiren'de açık bir
YouTube sekmesi, Chrome'daki YouTube sekmesiyle aşağı yukarı aynı belleği
tüketir. Bunu değiştirmenin tek yolu kendi motorunu yazmak, ki yazmıyoruz.

> **Vaat etmediğimiz şey:** "Muiren tek sekmede Chrome'dan az RAM yer."
> Yemez. Bunu pazarlamada da söylemeyeceğiz.

### "Daha az RAM yiyen bir motor kullansak?"

Sorulmuş ve kapatılmış bir soru; gerekçesi `docs/Roadmap.md` karar #6'da.
Özeti: Windows'ta bugünün webini açan iki motor var — Blink ve Gecko. **Gecko
gömülemiyor** (Mozilla gömme API'sini yıllar önce kaldırdı; GeckoView yalnız
Android). Servo ve Ladybird gerçekten az yiyor ama web uyumlulukları günlük
kullanıma yetmiyor — Crunchyroll'u açmayan bir motorun 40 MB'da kalması başarı
değil. Ultralight ve Sciter ise tarayıcı motoru değil, uygulama arayüzü motoru.

Ama sorunun içinde doğru bir sezgi var: Firefox çok sekmede gerçekten az yiyor.
Sebebi motor kalitesi değil, **süreç politikası** — ve o politika Chromium
üzerinde de kurulabiliyor. Aşağıdaki 2. kazanç kaynağı tam olarak bu.

## Kazanç nereden geliyor

Kazanç dört kaynaktan ve dördü de *politika*, motor değil.

### 1. Boştaki sekmelerin maliyetini sıfıra indirmek (asıl kazanç)

50 sekme açan bir insanın aynı anda baktığı sekme sayısı: 1. Diğer 49'unun
render süreci, JS yığını ve compositor dokusu **hiçbir işe yaramadan** RAM'de
duruyor.

Chrome'un "Memory Saver" özelliği bunu yapıyor ama muhafazakâr: varsayılan
eşiği uzun, kullanıcıya kontrol vermiyor, sabitlenmiş ve ses çalan sekmeleri
ayırt etmesi kaba. Muiren'in farkı politikanın **agresif ve ayarlanabilir**
olması, çünkü hedef kitlemiz onlarca sekmeyle çalıştığını biliyor ve bunu
istiyor.

Ölçek: 49 boştaki sekme × ortalama 80 MB ≈ **3–4 GB**. Asıl rakam bu.

### 2. Süreç birleştirme — uyanık sekmelerin taban maliyeti

Chromium'un varsayılan modeli **site örneği başına bir render süreci**. Aynı
anime sitesinden 10 sekme açtığında 10 ayrı süreç doğuyor ve her birinin taban
maliyeti 30–60 MB — sayfa ne kadar hafif olursa olsun, sırf süreç olduğu için.

Firefox'un çok sekmede Chrome'dan az yemesinin asıl sebebi bu: içerik
süreçlerini sınırlı bir havuzda paylaştırıyor (varsayılan sekiz civarı). Fark
motor kalitesinden değil süreç politikasından geliyor — dolayısıyla Chromium
üzerinde de kurulabiliyor:

| Bayrak | Ne yapıyor | Beklenen kazanç | Bedeli |
|---|---|---|---|
| `--process-per-site` | Aynı sitenin tüm sekmeleri tek süreci paylaşır | 50 sekmede ~40 → ~12 süreç, kabaca **1+ GB** | O sitenin bir sekmesi çökerse hepsi çöker |
| `--renderer-process-limit=N` | Süreç sayısına tavan koyar | Üstteki tavanı garantiye alır | Sekmeler CPU'da sıraya girer |
| `--js-flags=--max-old-space-size=N` | V8 yığınına tavan | Kaçak sekmeyi sınırlar | Ağır sitede sekme çöker — **varsayılan kapalı** |

Bu kaynak 1. kaynakla çakışmıyor, onu tamamlıyor: **uyutma boştaki içeriği
siliyor, süreç birleştirme uyanık sekmelerin taban maliyetini düşürüyor.**
Uyutmanın yapısal olarak ulaşamadığı tek yer burası — kullanıcının şu an
kullandığı sekmeler.

`--process-per-site` bedeli kabul edilmiş bir bedel: bir site çöktüğünde o
sitenin bütün sekmeleri gidiyor. Pratikte kayıp küçük, çünkü sekme zaten URL ve
kaydırma konumundan geri geliyor (`docs/Sekmeler.md`) — atılmış sekmeyi geri
getirmekten farkı yok. Yine de ayarlardan kapatılabiliyor
(`surec_politikasi`, bkz. `docs/Depolama.md`).

#### Site izolasyonu kapatılmıyor

`--disable-site-isolation-trials` bu listenin en çok kazandıranı ve
**kullanılmıyor**. Site izolasyonu, Spectre türü yan kanal saldırılarına karşı
tarayıcının tek gerçek savunması: onsuz, bir sekmedeki kötü niyetli JS aynı
süreci paylaştığı başka bir sitenin belleğini okuyabiliyor. İçinde bankacılık
sekmesi açılan bir programda bu takas yapılmaz — kaç MB kazandırırsa kazandırsın.

`--process-per-site` bununla karıştırılmasın: o *aynı* sitenin sekmelerini
birleştiriyor, siteler arası sınır yerinde kalıyor.

#### Doğrulanacak

WebView2 bazı Chromium anahtarlarını yok sayıyor. Bu üç bayrağın
`AdditionalBrowserArguments` üzerinden gerçekten geçip geçmediği ve süreç
tablosunda ölçülebilir fark yaratıp yaratmadığı Faz 0/R4'ün konusu
(`docs/Roadmap.md`). Geçmiyorlarsa bu kazanç kaynağı CEF'e geçişin
gerekçelerinden biri olur — CEF süreç modelini tamamen açıyor.

### 3. Kabuğun kendi maliyetini küçük tutmak

Rust + tek WebView2 ortamı. Electron tabanlı bir tarayıcı daha hiçbir sayfa
açılmadan ~200 MB'la başlıyor. Hedefimiz: **boş Muiren < 150 MB** (kabuk +
browser süreci + GPU, sıfır sekme).

### 4. Şişkinlik eklememek

Senkronizasyon servisi yok, telemetri yok, alışveriş/kupon paneli yok, yan
panel asistanı yok, önyükleme servisi yok, yeni sekme sayfasında reklam akışı
yok. Bunların her biri Chrome/Edge'de kalıcı bellek ve süreç demek.

Bu kalem bir özellik değil, bir **direnç**: her yeni fikirde "bu kaç MB" diye
sorulacak.

## Sekme durum makinesi

Dört durum. Geçişler `tabs/durum.rs` içinde, saf ve testli.

```
                 tıklandı
      ┌──────────────────────────────┐
      │                              ▼
 ┌─────────┐  odak gitti      ┌──────────┐
 │Arkaplan │◄─────────────────│  Etkin   │
 └────┬────┘                  └──────────┘
      │ boşta süresi > uyku eşiği          ▲
      ▼                                    │
 ┌─────────┐                               │
 │ Uyuyan  │───────────────────────────────┤ tıklandı (anında)
 └────┬────┘                               │
      │ boşta süresi > atma eşiği          │
      │ VEYA sistem bellek baskısı yüksek  │
      ▼                                    │
 ┌─────────┐                               │
 │ Atilmis │───────────────────────────────┘ tıklandı (URL'den yeniden yükle)
 └─────────┘
```

| Durum | Webview | Render belleği | Geri dönüş |
|---|---|---|---|
| `Etkin` | var, görünür | tam | — |
| `Arkaplan` | var, gizli | tam (compositor kısmen boşalır) | anında |
| `Uyuyan` | var, askıda (`TrySuspend`) | zamanlayıcılar durur, yığın kısmen boşalır | anında, sayfa durumu korunur |
| `Atilmis` | **yok** | sıfır | URL + kaydırma konumundan yeniden yükleme |

`Uyuyan` ile `Atilmis` arasındaki fark kullanıcı için çok önemli:

- **`Uyuyan`** — sayfanın durumu (form içeriği, video pozisyonu, giriş yapılmış
  oturumun JS durumu) yerinde. Uyanma göz kırpması kadar.
- **`Atilmis`** — sayfa sıfırdan yükleniyor. Doldurulmuş form gider. Bu yüzden
  atma kararı ihtiyatlı verilir ve **form girdisi olan sekme asla atılmaz**
  (bkz. koruma kuralları).

## Eşik kararı — `memory/esik.rs`

Fonksiyon imzası (saf, sistem çağrısı yok, zaman parametre):

```rust
pub fn karar(
    sekmeler: &[SekmeOzeti],      // id, durum, son_etkinlik, sabit?, ses?, form?
    baski: BellekBaskisi,         // Dusuk | Orta | Yuksek | Kritik
    ayarlar: &BellekAyarlari,
    simdi: Instant,
) -> Vec<Eylem>
```

`Eylem`: `Uyut(id)` · `At(id)` · `HedefDusur(id)` · `Uyandir(id)`.

### Koruma kuralları (bunlar eşiklerden önce gelir)

Bir sekme aşağıdakilerden biri doğruysa **uyutulmaz ve atılmaz**:

1. `Etkin` (bakılan sekme).
2. **Ses çalıyor.** Arka planda müzik/podcast dinlemek birinci sınıf kullanım.
3. **Sabitlenmiş** (kullanıcı pin'lemiş) — kullanıcı "bu hep dursun" demiş.
4. **Doldurulmuş form var.** Atılmaz; uyutulabilir (uyku durumu korur).
5. **Kullanıcı bu sekmeyi "uyutma" diye işaretlemiş** (sekme bazlı istisna).
6. **Site alan adı istisna listesinde** (ayarlardan; ör. web posta, panel).
7. **Muiwatch oturumuna bağlı sekme** — senkron kopar (`docs/Kopruler.md`).
8. **Tam ekran / Picture-in-Picture** aktif.

Bu liste `esik.rs` içinde tek bir `korumali()` fonksiyonu ve testlerinin
tamamı orada. Yeni koruma eklenince test de eklenecek.

**Sekizi de yazıldı.** #7 uzun süre eksikti (köprü yazılmadan kontrol edilecek
bir alan da yoktu) ve Faz 5'te kapandı; bağlı sekme listesi gözcü turunun
başında bir kez kopyalanıp `BellekAyarlari::muiwatch_sekmeleri` ile geliyor —
`esik.rs`e bir kilit geçirilemez, o dosya saf kalmak zorunda.

#8 (tam ekran) alanı uzun süre `SekmeOzeti` içinde **vardı ama hiç
doldurulmuyordu**: kural yazılı olduğu hâlde çalışmıyordu. Faz 4'te motorun
`ContainsFullScreenElementChanged` olayı bağlandı.

### Grup bazlı eşik (Faz 5)

Bir gruba kendi uyku eşiği verilebiliyor (`tabs/grup.rs`). Gerekçe tezin
kendisi:

> Katlanmış bir grup, kullanıcının "buraya şimdilik bakmıyorum" demesi. Bunu
> bilen bir tarayıcının o sekmeleri genel eşikte bekletmesi için sebep yok.

Üç kural:

- Grup eşiği **koruma kurallarının üstünde değil**: bir hızlandırıcı. Ses
  çalan sekme grubu ne derse desin uyumuyor.
- Baskı bölenleri grup eşiğine de uygulanıyor.
- **Atma eşiği birlikte ölçekleniyor**, ayarlardaki uyku/atma oranı
  korunarak. Ölçeklenmeseydi "1 dakikada uyusun" diyen grubun sekmeleri
  saatlerce uyur durumda beklerdi.

Katlama ile eşik **birleştirilmedi**: "katlanınca hemen uyusun" varsayılan bir
davranış olarak dayatılmıyor, ayrı bir tercih olarak duruyor. Kullanıcı grubu
düzen için katlıyor olabilir.

### Varsayılan eşikler

Makinenin toplam RAM'ine göre profil seçiliyor (`ayarlar` ilk açılışta
belirliyor, kullanıcı sonra değiştirebiliyor):

| Profil | Toplam RAM | Uyku eşiği | Atma eşiği | Aynı anda uyanık üst sınır |
|---|---|---|---|---|
| Sıkı | ≤ 8 GB | 2 dk | 20 dk | 6 |
| Dengeli (varsayılan) | 8–24 GB | 8 dk | 60 dk | 14 |
| Rahat | > 24 GB | 20 dk | 3 sa | 30 |

Baskı bu eşikleri kısaltır:

| Baskı | Kaynak | Etki |
|---|---|---|
| `Dusuk` | boş RAM > %40 | eşikler olduğu gibi |
| `Orta` | %20–40 | eşikler ÷ 2 |
| `Yuksek` | %10–20 | eşikler ÷ 4, "uyanık üst sınır" zorlanır |
| `Kritik` | < %10 | en eski uyuyanlar sırayla atılır |

> **Kural:** bu sayıların hiçbiri koda gömülmez. Hepsi `settings` içinden
> gelir ve ayarlar ekranında görünür. Kullanıcı "Sıkı" moddayken 3 GB kazanıp
> bunu görebilmeli.

## Ölçüm — `memory/olcum.rs`

İki ayrı şey ölçülüyor, karıştırılmayacak:

**Sistem baskısı.** `GlobalMemoryStatusEx` ile toplam/kullanılabilir fiziksel
bellek. Ucuz, sık çağrılabilir. Baskı seviyesi bundan çıkıyor.

**Sekme başına bellek.** Zor kısım. WebView2 render süreçlerini bize sekme
kimliğiyle vermiyor; elimizde PID listesi var. İki yol vardı:

1. Süreç listesi + hangi webview'e ait olduğu bilgisi.
2. Olmuyorsa: toplam WebView2 bellek tüketimini ölç, uyanık sekme sayısına
   böl, sekme başına *tahmin* göster. Dürüst etiketle: "yaklaşık".

Sekme başına rakam **karar için şart değil** — eşik kararı boşta kalma süresine
ve sistem baskısına dayanıyor. Sekme başına bellek sadece kullanıcıya
gösterilen bilgi (bellek paneli). Yani 1. yol çıkmasa proje durmuyordu, panel
"yaklaşık" diyordu.

### 1. yol yürüdü — `memory/esleme.rs`

Eşleme **iki** WebView2 çağrısının kesişiminde:

- `ICoreWebView2Environment13::GetProcessExtendedInfos` her sürecin taşıdığı
  çerçeveleri veriyor (`AssociatedFrameInfos`),
- `ICoreWebView2_20::FrameId` bir sekmenin **ana çerçeve** kimliğini.

İkisi olmadan biri işe yaramıyor: çerçeve listesi kimliklerden ibaret ve o
kimliklerin hangi sekme olduğunu söyleyen tek yer ikinci çağrı. Bu yüzden
`Yetenekler::surec_bilgisi` tek bayrak — "yarı çalışan eşleme" diye bir durum
yok.

Ana çerçeve kimliği **gezinme bittiğinde** tazeleniyor, ölçüm anında
sorulmuyor. Sebebi doğrudan aşağıdaki kural: ölçüm anında sormak, her turda
uyuyan sekmelerin webview'lerine dokunmak demek olurdu. Gezinme zaten sekmenin
uyanık olduğu andır.

Alt çerçeveler (iframe) tabloda yok; zincir `ParentFrameInfo` ile yukarı
yürütülüp üst düzey çerçeveye çıkılıyor. Gömülü bir oynatıcının ayrı süreci,
onu gösteren sayfanın maliyetinin parçası — "tanımadım" deyip ortak gidere
yazmak, site izolasyonu açıkken sekme başına rakamı sistematik olarak eksik
gösterirdi.

**Kabuğun kendi çerçevesi de tanınıyor** (`ICoreWebView2_20::FrameId`, kabuk
webview'i üzerinden). Kabuğun arayüzü de bir render sürecinde koşuyor ve o
süreç listeye giriyor; tanınmasaydı her turda "sahibi bulunamayan render
süreci" sayılır, eşleme **hiçbir zaman** tam çıkmaz ve `olcum_yaklasik` kalıcı
olarak true kalırdı — yani sekme başına ölçümün bütün amacı boşa giderdi.
Kabuk sahip sayılıyor ama sekme sayılmıyor: payı **ortak gidere** yazılıyor,
çünkü kullanıcı kabuğu kapatarak yer açamaz. Kabuk ayrı bir kökende ve
normalde ayrı süreçte, ama süreç sınırına dayanılmışsa bir sekmeyle aynı
sürece düşebiliyor; o durumda pay bölünüyor ve rakam "paylaşımlı"
etiketleniyor. Bölme kalanı kabuğun payında kalıyor, düşürülmüyor: panelde
sekme rakamlarının toplamı + ortak gider, ölçülen toplamı tutmak zorunda.

Hesabın tamamı saf tarafta (`memory/esleme.rs`, `esik.rs` ile aynı ayrım):
motor bir PID ve sekme listesi veriyor, Win32 PID başına MB, birleştirme
testli bir defterde. Defterin durum tutmasının sebebi atılmış sekme: süreci
yok, dolayısıyla ölçülecek bir şeyi de yok, ama panelin göstermesi gereken
sayı tam da "bu sekme uyanık olsaydı ne kadar yer tutardı".

Bunun bir yan sonucu **kazanç rakamının düzelmesi**: eskiden uyuyan sekmelerin
hâlâ tuttuğu yer kazançtan düşülmüyordu (uyuyan sekme sıfır yer tutmuyor,
askıya alınmış render süreci ayakta). Artık her pasif sekme için hesap
"uyanıkken tuttuğu − bugün tuttuğu".

**Yaklaşık ne zaman biter:** iki koşul birden. Eşlemenin tam olması (her
render sürecinin sahibi bulunmuş) *ve* her pasif sekmenin uyanıkken en az bir
kez ölçülmüş olması. Oturumdan `Atilmis` doğan, bir kez bile webview almamış
bir sekme ikinci koşulu düşürüyor ve o sekme için eski tahmine dönülüyor —
ama yalnız o sekme için, ölçülmüş otuz sekmenin gerçek rakamı atılmıyor.

Paylaşılan süreç (`--process-per-site` açıkken aynı siteden on sekme) **eşit**
bölünüyor ve rakam öyle etiketleniyor. Eşit bölmek gerçeği tam yansıtmıyor;
paylaşılan bir rakamı tek sekmenin maliyeti gibi göstermek ise kullanıcıyı
yanlış sekmeyi kapatmaya iterdi.

> **Kural (CLAUDE.md #5):** ölçüm hiçbir koşulda uyuyan sekmeyi uyandırmaz.
> Uyuyan sekmenin gösterilen değeri, uykuya dalmadan önceki son ölçüm ve
> yanında "uykuda" rozeti var. `GetProcessExtendedInfos` bu kuralı bozmuyor:
> çağrı ortam düzeyinde, sayfaya hiç inmiyor ve betik çalıştırmıyor.

### Uyanma gecikmesi — `memory/gecikme.rs`

Aşağıdaki başarı tablosunun iki satırı ("etkileşime hazır < 300 ms", "ilk boya
< 2 sn") uzun süre **ölçülemiyordu**: gecikmeyi sayan bir yer yoktu. Elle
kronometre tutmak da bir cevap değil, çünkü tabloda istenen tek bir örnek değil
bir dağılım.

Defter `esik.rs` ile aynı ayrımda: **hesap saf, saat dışarıda.** Kronometreyi
başlatan/durduran anlar `Instant` olarak parametre geliyor, dosya hiç
`Instant::now()` çağırmıyor — böylece ölçüm mantığı `--no-default-features`
derlemesinde de test edilebiliyor.

| Kaynak | Kronometre başlıyor | Duruyor | Ölçülen |
|---|---|---|---|
| `Uyuyan` → `Etkin` | `Resume` çağrısından **önce** | sayfanın JS ana iş parçacığı betiğimizi çalıştırınca | gerçekten "etkileşime hazır" |
| `Atilmis` → `Etkin` | webview yaratılmadan önce | `NavigationCompleted` | ilk boya **değil**, onun üst sınırı |

İkinci satır kasıtlı olarak temkinli. WebView2 bize bir ilk boya olayı vermiyor;
olmayan bir olayı varmış gibi göstermektense dürüst bir üst sınır veriliyor. Bu
sayı eşiği geçiyorsa kriter **kesin olarak** sağlanmış oluyor. Ağ süresi dahil
(aşağıdaki tablo bunu zaten şart koşuyor).

Birincisi için motor yüzeyine ayrı bir "yoklama" metodu **eklenmedi**: bu tam
olarak `kaydirma_iste`nin yaptığı `ExecuteScript` turu ve ölçüm için ikinci bir
kapı açmak, `motor/yok.rs` ile birlikte iki dosyada bakılacak yeni bir yüzey
demekti (CLAUDE.md #4). `Resume`in dönmesini beklemek yetmezdi — o yalnız
çağrının kabul edildiğini söylüyor, render sürecinin kullanıcının tuşunu
işleyecek hâle geldiğini değil.

Üç şey **sayılmıyor** ve üçü de kuralla, sessizce değil:

- Başarısız gezinme (CLAUDE.md #7). Sayılsaydı DNS hatasıyla anında dönen bir
  sekme tabloyu olduğundan iyi gösterirdi.
- 30 saniyeyi aşan uyanma. Ağı kopan bir kullanıcının tek örneği medyanı
  kalıcı olarak zehirlerdi.
- Uyanmayı tamamlamadan geri uyutulan/atılan sekme. Geç gelen betik cevabı,
  arada geçen bütün uyku süresini "uyanma gecikmesi" diye yazardı.

Son 64 örnek tutuluyor (halka tampon, birkaç KB): bellek iddiası olan bir
programda ölçüm aracının kendisi bir bellek kalemi olmamalı. Ortalama kasten
verilmiyor — tek bir 8 saniyelik uyanma otuz iyi örneğin ortalamasını bozar;
medyan tipik hâli, p95 ve en kötü değer kuyruğu söylüyor.

Örnek yokken panel **"henüz ölçülmedi"** yazıyor, sıfır değil: sıfır bir ölçüm
sonucu değil, ölçümün yokluğu — `bellek_ozeti`nin `None` olduğu andaki kuralla
aynı.

## Gözcü — `memory/gozcu.rs`

Kendi iş parçacığında koşan basit döngü:

```
her N saniyede (varsayılan 10):
    baski   = olcum::sistem_baskisi()
    ozetler = tabs::ozetler()              // kilit kısa tutulur
    eylemler = esik::karar(ozetler, baski, ayarlar, Instant::now())
    her eylem için: motor'a uygula + tabs::durum güncelle + olay yayınla
```

Neden ayrı iş parçacığı: arayüz kapalıyken (pencere simge durumunda, kullanıcı
oyunda) politikanın çalışması gereken **asıl an** o. Muiply'da mpv olay
döngüsünün kuyruğu ilerletmesiyle aynı gerekçe.

### Pencere simge durumundayken

Kullanıcı tarayıcıya bakmıyorsa 14 sekmenin uyanık kalmasının hiçbir karşılığı
yok. Politikanın çalışması gereken asıl an bu ve gözcünün ayrı bir iş
parçacığında koşmasının sebebi de tam olarak bu.

Karşılığı **yeni bir eşik değil**, var olan baskı tablosu: gizli pencere en az
`Yuksek` sayılıyor, yani eşikler ÷ 4 (`esik::BellekBaskisi::etkin`, saf ve
testli). Ayrı bir "gizli eşiği" ayarı eklenmedi — ikinci bir sayı, kullanıcının
ayarlar ekranında anlamlandırması gereken ikinci bir kavram demek.

Sekmeler **anında uyumuyor**: profilin uyku eşiğinin dörtte birini geçmiş
olanlar uyuyor. Beş saniyeliğine küçültülen pencere bir uyku dalgası
başlatmıyor.

Birleştirme `max` ile yapılıyor: `Kritik` ölçülmüşse pencere gizlendi diye
`Yuksek`e **düşmüyor**. Atama olsaydı, bellek gerçekten tükenmişken pencereyi
küçültmek politikayı gevşetirdi.

Üç şey daha:

- **Gözcü periyodu yarıya iniyor.** Turun bedeli de aynı anda düşüyor:
  gizliyken süreç tablosu hiç taranmıyor ve bellek özeti yayınlanmıyor —
  paneli görmeyen bir kullanıcı için Toolhelp32 anlık görüntüsü almanın
  karşılığı yok. Yani daha sık koşan tur **daha ucuz** bir tur. Tersi olsaydı
  (iki katı sıklıkta tam ölçüm) bedeli tam da kullanıcının oyunda olduğu ana
  denk gelirdi.
- **Uyanık üst sınırı düşürülmüyor.** Oyun modundan farkı, kullanıcının bir şey
  söylememiş olması: simge durumuna almak "sekmelerim gitsin" demek değil, "şu
  an bakmıyorum" demek.
- **Koruma kuralları aynen geçerli.** Arka planda müzik dinlemek birinci sınıf
  kullanım ve pencereyi küçültmek onu bitirmek demek değil.

Durum değiştiği anda bir tur koşuluyor; pencere geri geldiğinde aynı tur,
gizliyken atlanan bellek özetini de tazeliyor.

Sekmenin uyuma sebebi arayüzde `PencereGizli` olarak görünüyor, `SistemBaskisi`
olarak değil (`docs/IPC.md`): ikisi kullanıcıya farklı şey söylüyor ve tek
sebebe indirilseydi pencereyi küçülten kullanıcı makinesinde olmayan bir bellek
sorunu arardı.

## Oyun modu

Muifly oyun başlattığını bildirdiğinde (ya da Muiren tam ekran bir oyun
algıladığında) Muiren tek hamlede:

1. Baskıyı `Yuksek` kabul eder — eşikler ÷ 4.
2. Korumalılar dışında her sekmeyi uyutur.
3. Uyanık üst sınırı 2'ye çeker.
4. Kabuk penceresini **simge durumuna alır** — ayrı bir ayarla, varsayılan
   kapalı (`oyun_pencere_gizle`).

Oyun bitince eski politikaya döner ama **sekmeleri kendiliğinden uyandırmaz** —
kullanıcı hangisine tıklarsa o uyanır. Ayrıntı: `docs/Kopruler.md`.

### 4. adım neden "gizlemek" değil "simge durumuna almak"

Bu madde uzun süre yazılmadan durdu, çünkü kullanıcının penceresini
kendiliğinden yok etmek geri alınması zor bir davranış. Yazılırken üç kural
kondu:

- **`hide()` değil `minimize()`.** Gizlenen pencerenin görev çubuğu düğmesi de
  kayboluyor; tam ekran bir oyun ekranı kaplarken kullanıcının tarayıcıya geri
  dönecek görünür bir yolu kalmıyor. Simge durumundaki pencere her an geri
  geliyor ve kazanç aynı: pencere kompozisyona girmiyor, Chromium örtülü
  pencerenin render süreçlerini geri plana atıyor.
- **Ayrı bir ayar, varsayılan kapalı.** Oyun modunun kendisi elle de
  açılabiliyor; kullanıcının istemediği bir pencere hareketi, özelliğin
  tamamını kapattırıyor.
- **Geri gelme garantili.** Pencereyi biz küçülttüysek oyun modu kapanınca biz
  geri getiriyoruz (`oyun_kucultu` bayrağı) — kullanıcının kendi küçülttüğü
  pencereye dokunulmuyor. Algılama iş parçacığı ölse bile gözcünün her turu
  aynı denetimi tekrarlıyor: oyun modu kapalıyken küçültülmüş bırakılmış bir
  pencere yok.

Bu adımın **ölçülebilir** karşılığı ayrıca var: pencere simge durumuna
girdiğinde baskı zaten en az `Yuksek` sayılıyor (yukarıdaki "Pencere gizli"
kuralı), yani 4. adım 1. adımı ikinci bir yoldan doğruluyor.

## Başarı kriterleri (ölçülmeden "başardık" denmez)

Faz 2 sonunda, temiz Windows 11 makinede, aynı 50 sekmelik oturum
(10 sosyal medya, 10 haber, 10 forum, 10 anime/video sitesi, 10 dokümantasyon):

| Ölçüt | Hedef |
|---|---|
| Boş tarayıcı (0 sekme) | < 150 MB |
| 50 sekme açıldıktan hemen sonra | Chrome'un **altında** (süreç birleştirme buradan itibaren çalışıyor) |
| 50 sekmede render süreç sayısı | Chrome'un **yarısından az** |
| 50 sekme, 15 dk boşta beklendikten sonra | Chrome'un **yarısından az** |
| Uyuyan sekmeye tıklayıp etkileşime hazır olma | < 300 ms |
| Atılmış sekmeye tıklayıp ilk boya | < 2 sn (ağ hariç tutulmaz, not düşülür) |

Bu tablo `docs/Roadmap.md` içindeki Faz 2'nin kabul kriteri. Ölçüm yöntemi ve
ham sayılar `docs/olcumler/` altına tarih atılarak yazılacak — "hızlı olduğunu
hissediyorum" kabul edilmiyor.

**Altı satırın da adım adım yöntemi `docs/olcumler/README.md` içinde**, rapor
iskeletiyle birlikte. Son iki satırı Muiren kendi ölçüyor (yukarıda "Uyanma
gecikmesi"); ilk dördü Chrome'la karşılaştırma gerektirdiği için elle
koşuluyor.
