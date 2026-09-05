# Medya — Video, Codec, Takılmama

Hedef kitlenin ikinci yarısı anime izleyen insanlar. "Video izlerken kasmasın"
somut bir mühendislik iddiası; bu dosya onu somutlaştırıyor.

## Video neden takılır — dört ayrı sebep

Kullanıcı "kasıyor" der ama arkasında dört farklı arıza var ve çözümleri
birbirinden bağımsız:

| Belirti | Gerçek sebep | Muiren'in cevabı |
|---|---|---|
| Kare atlıyor, CPU %100 | Yazılımsal kod çözme (donanım devre dışı) | GPU hızlandırmayı zorla, düşerse kullanıcıya söyle |
| Ses/görüntü kayıyor | Ses cihazı örnekleme uyuşmazlığı, sistem yükü | Motorun işi; teşhis paneline yaz |
| Ara ara donuyor | **Arka plan sekmeleri GC/JS koşturuyor** | Bellek politikası — asıl kazancımız |
| Tam ekranda pencere kaybı | Kabuk penceresi ile webview çakışması | Tam ekran devri (aşağıda) |

Üçüncü satır bu projenin doğal avantajı: 40 sekmeyi uyutan bir tarayıcıda
video sekmesi CPU'yu paylaşmak zorunda kalmıyor. Yani "kasmama" iddiası
"az RAM" iddiasının yan ürünü, ayrı bir özellik değil.

## Donanım kod çözme

WebView2 = Edge Chromium, dolayısıyla donanım kod çözme motorun kendi
mekanizması. Bize düşen üç şey:

1. **Devre dışı bırakmamak.** GPU hızlandırmayı kapatan Chromium bayrağını
   kazara geçirmemek. Bayrak listesi tek yerde (`motor/gercek.rs` içindeki
   `EK_BAYRAKLAR` sabiti) ve yorumla gerekçelendirilmiş.
2. **Düştüğünü fark etmek.** "Donanım kod çözme kapalı" durumunu **teşhis
   panelinde** göstermek. Kullanıcı sürücü/GPU sorununu bilmiyorsa tarayıcıyı
   suçluyor.
3. **Ölçüm sunmak.** Codec başına donanım/yazılım ayrımı — teşhis panelinde.

### Teşhis paneli — YAZILDI

Ayarlar → "Teşhis ekranını aç". Dört bölüm, dördü de bu belgedeki bir açık
soruya karşılık geliyor:

| Bölüm | Kaynak | Karşılığı |
|---|---|---|
| Görüntü | WebGL `UNMASKED_RENDERER_WEBGL` | yukarıdaki tablonun 1. satırı |
| Codec | `MediaCapabilities.decodingInfo` | aşağıdaki "doğrulanacak" tablosu |
| DRM | `requestMediaKeySystemAccess` | "DRM — projenin en büyük riski" |
| Ölçüm raporu | ölçülen alanlar + `?` | `docs/olcumler/README.md` başlığı |

Üç karar burada duruyor:

- **`chrome://gpu` okunmuyor.** WebView2 o sayfayı açmıyor ve açsaydı bile
  çıktısı sürümden sürüme değişen bir HTML tablosu olurdu. Yerine standart
  API'ler: oluşturucu dizesi GPU hızlandırmanın düşüp düşmediğini,
  `decodingInfo` ise codec başına **`powerEfficient`** bayrağını veriyor —
  "bunu donanım çözüyor" cevabının standarttaki karşılığı bu.
- **Ölçüm sekmeye inmiyor** (CLAUDE.md #5). Sorular kabuk webview'inin kendi
  içinde koşuyor; cevap temsili, çünkü bütün webview'ler aynı WebView2
  ortamını ve aynı bayrak dizesini paylaşıyor (CLAUDE.md #16). Sekmeye betik
  göndermek uyuyan bir sekmeyi uyandırma riski demekti ve buradaki soruların
  hiçbiri buna değmez.
- **Atılan kare sayısı (`VideoPlaybackQuality`) yazılmadı.** O rakam sekmedeki
  `<video>` öğesine ait ve okumak için sayfaya betik göndermek gerekiyor —
  yani motor yüzeyine yeni bir "betik çalıştır ve cevabı getir" metodu
  (CLAUDE.md #4) ve uyuyan sekme sorusu. Karşılığı: teşhisin bugünkü hâli
  "bu makinede donanım kod çözme var mı" sorusunu **zaten** cevaplıyor;
  kare sayısı onun üstüne bir şey eklemiyor. Sekme başına ölçüm istendiğinde
  yazılacak yer bellidir ve gerekçesi o gün tartılır.

Yorumlama saf ve testli (`src/lib/teshis.ts`), ölçüm ayrı
(`src/hooks/useTeshis.ts`) — `esik.rs`/`olcum.rs` ayrımının arayüzdeki
karşılığı. Testlerden biri gerçek bir tuzağı sabitliyor: sürücü dizesi ASCII
ve Türkçe küçültme "SWIFTSHADER" dizesini "swıftshader" yapıp eşleşmeyi
düşürüyordu (CLAUDE.md #10'un kapsamı **kullanıcı metni**, teknik tanımlayıcı
değil).

## Codec durumu (ölçüm aracı hazır, sayı bekliyor — Faz 0)

WebView2 Edge'in codec setini taşıyor. Beklenen tablo aşağıda; **ölçülmüş
hâli** artık teşhis panelinde duruyor ve makineden makineye değişiyor (AV1 ve
HEVC donanım desteği GPU'ya bağlı). Panelin cevabı `docs/olcumler/` altına
yazılınca bu tablonun "beklenti" sütunu ölçülen değerle değiştirilecek.

| Codec | Beklenti | Not |
|---|---|---|
| H.264 / AAC | var | anime sitelerinin çoğunluğu |
| VP9 / Opus | var | YouTube |
| AV1 | var (yeni sürümlerde donanım destekli) | YouTube, bazı yayıncılar |
| HEVC | **yok, ek gerektiriyor** | Windows'ta ayrı HEVC uzantısı |

HEVC'nin yokluğu web'de nadiren sorun (siteler HEVC yayınlamıyor) ama **yerel
dosya açılırsa** sorun. Cevabımız zaten hazır: yerel medya dosyası Muiply'a
gidiyor (`docs/Kopruler.md`) ve Muiply'ın libmpv'si HEVC dahil hepsini açıyor.
Bu, portföyün en temiz kesişimi — Muiren'in eksiğini Muiply kapatıyor.

## DRM — projenin en büyük riski

**Crunchyroll, Netflix, Disney+ Widevine istiyor.** Hedef kitle "animeciler"
olduğuna göre Crunchyroll'un açılıp açılmaması pazarlık konusu değil.

Durum: WebView2'de Widevine/PlayReady desteği **varsayılan olarak açık
değil**; ortam seçenekleriyle etkinleştirilmesi ve lisans/dağıtım koşullarının
kontrol edilmesi gerekiyor. Bu belge bunu **doğrulanmamış** ilan ediyor.

> **Faz 0 kabul kriteri:** Crunchyroll'da bir bölüm, Muiren'in içinde,
> baştan sona oynuyor. Oynamıyorsa iki seçenek var ve karar `docs/Roadmap.md`
> içine karar kaydı olarak yazılacak:
>
> - **A.** DRM'li siteler için "Muiply/tarayıcıda aç" devri — dürüst ama zayıf.
> - **B.** Motoru CEF'e çevirmek — Widevine'ı çözer, dağıtım boyutunu
>   ~150 MB'a çıkarır, `motor/` soyutlaması bunun için var.
>
> Bu kararı ertelemek en pahalı seçenek. Faz 0'da kapanacak.

**İlk sinyal artık ölçülebiliyor.** Teşhis paneli Widevine ve PlayReady
anahtar sistemlerini `requestMediaKeySystemAccess` ile soruyor; cevap
"bulundu" ya da "bulunamadı". Bu, kabul kriterinin **yerine geçmiyor** ve
panel bunu açıkça yazıyor: anahtar sisteminin var olması bir bölümün baştan
sona oynayacağı anlamına gelmiyor (lisans sunucusu, sürüm, L1/L3 seviyesi).
Ama "bulunamadı" cevabı kesin bir bilgi — o durumda A/B kararı bir bölüm
denenmeden verilebilir ve saatlerce Crunchyroll hesabı aramaya gerek kalmaz.

## Tam ekran

Sayfa tam ekran isteyince (`requestFullscreen`) kabuk çekiliyor: sekme çubuğu
ve adres çubuğu gizleniyor, webview pencerenin tamamını kaplıyor. Çıkışta eski
düzen geri geliyor.

Dikkat edilecekler:

- Tam ekran sekme **korumalıdır** — uyutulmaz, atılmaz (`docs/Bellek.md`).
- `Esc` tuşu webview'e gidiyor, kabuğa değil; kabuk kendi `Esc` işlemesini bu
  sırada askıya alıyor.
- Tam ekrandan çıkış olayı gelmezse (site çökerse) kabuk 2 saniyelik
  zamanlayıcıyla kendini geri getiriyor — kullanıcı kilitli kalmıyor.

## Picture-in-Picture

Motorun kendi PiP'i kullanılıyor, kendi pencere yönetimimizi yazmıyoruz. PiP
aktif olan sekme korumalı.

## Muiply'a devir

Bir sekme üzerinde:

- **Sağ tık → "Muiply'da aç"** (video/ses öğesi üzerinde),
- Yerel dosya (`file://`) bir medya dosyasına gezinildiğinde otomatik öneri,
- İndirilen bir medya dosyası tamamlandığında bildirimden.

Ne gönderiliyor: dosya yolu (yerelse) ya da doğrudan medya URL'i. Ne
gönderilmiyor: çerez, oturum başlığı, kimlik bilgisi. Muiply yerel oynatıcı;
ağ akışı zaten kapsam dışı (bkz. Muiply CLAUDE.md). Yani devir esas olarak
**yerel dosyalar** için anlamlı; uzak URL devri yalnız Muiply ileride
destekleyeceğini bildirirse eklenecek. Ayrıntı: `docs/Kopruler.md`.

## Ses göstergesi ve sessize alma

Ses çalan sekmede çubukta gösterge. Göstergeye tıklayınca o sekme sessize
alınıyor. Sessize alınmış sekme **artık korumalı değil** — kullanıcı sesini
kapattıysa arka planda tutmaya gerek yok, uyku eşiğine tabi. Küçük bir detay
ama 50 sekmede fark yaratıyor.

## Reklam ve izleyici engelleme

Anime siteleri reklam bakımından web'in en ağır köşesi: pop-under, sahte
oynat düğmesi, yönlendirme. Engelleme bu projede bir **bellek özelliği**:
yüklenmeyen reklam çerçevesi = yüklenmeyen render süreci.

Faz 4 kapsamı, `docs/Roadmap.md` içinde. Tasarım notları:

- Motor seviyesinde istek filtreleme (`WebResourceRequested`), sayfaya JS
  enjekte ederek değil — enjeksiyon hem yavaş hem kırılgan.
- Filtre listeleri kullanıcı tarafından eklenen dosyalar; **uygulama kendi
  listesini internetten çekmiyor** (telemetri/bulut yok kuralı). Kullanıcı
  liste dosyasını kendi koyuyor ya da Muiget ile indiriyor.
- Pop-up ve yönlendirme engelleme (kullanıcı jesti olmadan açılan pencere)
  varsayılan açık — bu liste gerektirmiyor, davranış kuralı.
