# Sekmeler

`docs/Bellek.md` sekmenin *durumunu* anlatıyor. Bu dosya sekmenin *kimliğini*,
*sırasını* ve *hayatta kalmasını* anlatıyor.

## Sekme nedir

Mimarinin en önemli cümlesi burada tekrar ediliyor:

> **Sekme bir veri kaydıdır. Webview ise o kaydın pahalı, isteğe bağlı, her an
> yok edilebilir bir eki.**

```rust
pub struct Sekme {
    pub id: SekmeId,              // uygulama ömrü boyunca tekil, yeniden kullanılmaz
    pub url: String,              // son başarılı gezinme
    pub baslik: String,           // sayfadan geldi → GÜVENİLMEZ, kırpılmış
    pub favicon: Option<FaviconId>,
    pub gecmis: Vec<String>,      // geri/ileri, üst sınır 50
    pub gecmis_konum: usize,
    pub kaydirma: f32,            // 0.0–1.0, atılmış sekmeyi geri getirirken
    pub ebeveyn: Option<SekmeId>, // "bu sekme hangi sekmeden açıldı"
    pub grup: Option<GrupId>,
    pub sabit: bool,
    pub uyutma_istisnasi: bool,
    pub durum: Durum,
    pub son_etkinlik: Instant,
    pub ses_calıyor: bool,
    pub form_dolu: bool,
}
```

Bu yapı sekme başına birkaç yüz bayt. 500 sekme = birkaç yüz KB. Sekme
sayısının üst sınırını RAM değil, sekme çubuğunun çizimi belirliyor.

## Sıra ve ağaç — `tabs/agac.rs`

Sekmeler düz bir liste **ama** `ebeveyn` alanı bir ağaç kuruyor. Bir sekmeden
link açtığında yeni sekme onun çocuğu oluyor ve **hemen sağına** giriyor,
listenin sonuna değil. Çok sekme açan biri için tek gerçek fark bu: 40 sekme
arasında kaybolmuyorsun, açtığın şey açtığın yerin yanında duruyor.

`agac.rs` tamamen saf ve testli. Karar verdiği şeyler:

- `ekle(ebeveyn, konum_politikasi) -> usize` — yeni sekme hangi indekse girer.
  Kardeşleri varsa en son kardeşin sağına; yoksa ebeveynin hemen sağına.
- `kapat(id) -> Option<SekmeId>` — kapandıktan sonra hangi sekme etkin olur.
  Sırayla: sağdaki kardeş → soldaki kardeş → ebeveyn → sağdaki sekme.
  (Chrome'un davranışı "sağdaki sekme"; ebeveyne dönmek ağaç modelinde çok
  daha az şaşırtıyor.)
- `tasi(id, hedef)` — sürükle-bırak; bir sekme taşınırken çocukları da taşınır.
- `grup_kapat(grup)` / `alt_agac_kapat(id)` — toplu kapama.

> **Kural:** `agac.rs` içinde `motor` çağrısı, zaman okuma veya IO yok. Girdi
> bir sekme listesi, çıktı yeni sıra ya da bir id. Testler burada yazılıyor.

## Sekme kimliği asla geri dönüştürülmez

`SekmeId` monoton artan bir sayaç. Kapanan sekmenin id'si bir daha kullanılmaz.
Sebebi: gözcü iş parçacığı ile arayüz arasında her zaman birkaç yüz milisaniye
gecikme var. Id yeniden kullanılırsa, gözcünün "sekme 7'yi uyut" kararı
kullanıcının yeni açtığı 7 numaralı sekmeye çarpıyor. Bu tür hatalar
tekrarlanamaz ve haftalarca bulunamaz.

## Oturum — hayatta kalma

Oturum (`oturum.json`, uygulama veri klasöründe) sekme listesinin diske yazılmış
hâli. İçinde webview yok, sadece `Sekme` kayıtları.

**Ne zaman yazılıyor:**
- Sekme açılınca/kapanınca/gezinince (gecikmeli, 2 sn debounce).
- Uygulama kapanırken.
- Her 60 saniyede bir güvenlik yazımı.

**Nasıl yazılıyor:** geçici dosyaya yaz → `fsync` → atomik rename. Yarıda
kesilen yazım eski oturumu bozmuyor. Ayrıca bir önceki oturum `oturum.bak`
olarak duruyor: çökme sonrası bozuk JSON gelirse yedeğe düşülüyor.

**Açılışta:** bütün sekmeler `Atilmis` durumunda doğuyor. Yani 200 sekmelik bir
oturum saniyeler içinde açılıyor ve sıfır render belleği tüketiyor; sadece
tıklananlar yükleniyor. Bu davranış ayarlanabilir değil — "açılışta hepsini
yükle" seçeneği bilinçli olarak yok, çünkü projenin tezine aykırı.

**Sabitlenmiş sekmeler istisna:** açılışta yükleniyorlar (kullanıcı onları
kalıcı olsun diye sabitledi).

## Kapatılan sekmeyi geri alma

Son 25 kapatılmış sekme bellekte tutuluyor (`Ctrl+Shift+T`). Kayıt zaten küçük
olduğu için maliyeti yok. Uygulama kapanınca liste siliniyor — çöpü diske
yazmıyoruz.

## Gruplar

Grup = renk + ad + sekme kümesi. Katlanabilir. Katlanmış grup, içindeki
sekmelerin **uyku eşiğini yarıya indiriyor**: kullanıcı grubu katlayarak "şu an
bunlarla işim yok" demiş oluyor. Bu, bellek politikasının kullanıcı niyetinden
sinyal aldığı tek yer ve bilinçli.

**Faz 5'te eklenen ikinci sinyal:** gruba **kendi uyku eşiği** verilebiliyor.
Katlama onun *üstüne* biniyor, yerine geçmiyor — ikisi de kullanıcıdan gelen
ayrı birer sinyal ve katlama grubun eşiğinin yerine geçseydi kullanıcının
yazdığı sayı katlanınca anlamını yitirirdi.

Üç kural (`memory/esik.rs`, hepsi testli):

- Eşik **koruma kurallarının üstünde değil**: bir hızlandırıcı. Ses çalan ya
  da sabitlenmiş sekme grubu ne derse desin uyumuyor.
- Baskı bölenleri grup eşiğine de uygulanıyor.
- **Atma eşiği birlikte ölçekleniyor**, ayarlardaki uyku/atma oranı
  korunarak. Ölçeklenmeseydi "1 dakikada uyusun" diyen grubun sekmeleri
  saatlerce uyur durumda beklerdi.

Renk **serbest bir dize değil, bir enum** (`tabs::GrupRengi`): grup rengi
arayüzde bir CSS değerine dönüşüyor ve kullanıcının yazdığı bir dizeyi oraya
bırakmak, `theme/jeton.rs` içinde uzun uzun kapatılan kapıyı yan taraftan
açmak olurdu.

Grup **silmek sekmeleri kapatmıyor**, yalnız gruptan çıkarıyor. Sekmeleri de
kapatmak ayrı bir eylem; ikisini tek düğmede birleştirmek basılmaya korkulan
bir düğme üretiyor.

## Sekme çubuğu — çok sekmede ne olur

Hedef kitle 50+ sekme açıyor. Klasik sekme çubuğu 20'de kullanılamaz hâle
geliyor (her sekme 12 piksel, favicon bile sığmıyor). Muiren'in cevabı:

- **Yatay çubuk**, sekme genişliği bir alt sınıra kadar daralıyor (favicon +
  ses göstergesi sığacak kadar), sonra **kaydırma** başlıyor — daha fazla
  daralma yok.
- **Sekme arama** (`Ctrl+Shift+A`): açık sekmelerde başlık ve URL araması,
  uyuyan/atılmış sekmeler dahil ve **onları uyandırmadan** (kayıt zaten
  bellekte). Türkçe arama `toLocaleLowerCase("tr")` ile.
- **Dikey sekme çubuğu** yan panelde (Faz 3). 50 sekmede yatay çubuk fizik
  olarak yetmiyor; dikey liste doğru cevap.
- Uyuyan sekme soluk, atılmış sekme daha soluk + kesikli kenarlık. Kullanıcı
  neyin bellekte olduğunu bir bakışta görüyor — bu bir tanıtım öğesi değil,
  güven öğesi.

Görsel ayrıntı ve jetonlar: `docs/Frontend.md`.

## Sekme başlığı güvenilmez

`document.title` sayfanın yazdığı bir dize. Yapılması gerekenler, istisnasız:

- 200 karakterde kırp.
- Satır sonu, sıfır genişlikli karakter ve iki yönlü metin kontrol
  karakterlerini (`U+202E` vb.) temizle — yoksa sekme çubuğunda metin ters
  akıyor ve sahte alan adı gösterilebiliyor.
- Arayüze basarken React'in kaçışına güven, `dangerouslySetInnerHTML` kullanma.
- Geçmişe ve oturum dosyasına temizlenmiş hâlini yaz, hamını değil.

Aynısı favicon URL'i, indirme dosya adı ve `window.open` ile gelen ad için de
geçerli.

## Yeni sekme sayfası

`muiren://yeni` — kabuğun kendi sayfası, ağ isteği yok, reklam yok, öneri akışı
yok. İçinde: arama kutusu, sabitlenmiş kısayollar, tema arka planı ve
(kullanıcı isterse) bellek özeti. Yükleme maliyeti neredeyse sıfır olmalı;
`docs/Bellek.md` içindeki "boş tarayıcı < 150 MB" hedefi buna dahil.
