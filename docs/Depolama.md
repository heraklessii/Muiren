# Depolama — Geçmiş, Yer İmleri, Ayarlar

## Neler nerede

| Veri | Yer | Biçim |
|---|---|---|
| Geçmiş, yer imleri | `muiren.db` | SQLite (WAL) |
| Oturum (açık sekmeler) | `oturum.json` + `oturum.bak` | JSON, atomik yazım |
| Ayarlar | `ayarlar.json` | JSON |
| Temalar | `temalar/<ad>/` | açılmış `.muitema` içeriği |
| Faviconlar | `favicon/` | dosya, içerik hash'iyle adlandırılmış |
| Web verisi (çerez, LocalStorage, önbellek) | `webview2/` | **WebView2'nin kendi klasörü** |

Kök: `%APPDATA%\Muiren\` (Tauri'nin uygulama veri dizini).

Son satır önemli: çerezleri, oturumları ve site verisini biz yönetmiyoruz.
WebView2'nin kullanıcı veri klasörü onun sözleşmesi. Bizim işimiz o klasörün
yolunu belirlemek ve gerektiğinde temizlemek.

### Favicon deposu (Faz 3'te yazıldı)

Dosya adı **içeriğin 128 bitlik FNV-1a karması**, onaltılık. Üç kazanç:

- Aynı ikonu kullanan yüzlerce sekme tek dosya, tek okuma.
- Kimlik **sayfadan gelen hiçbir dizeyi taşımıyor**; dolayısıyla dosya adı
  üzerinden dizin taşması mümkün değil. Okuma yolu yine de doğruluyor
  (`favicon::kimlik_gecerli`) — kimlik arayüzden geri geliyor ve bir dosya
  yoluna dönüşüyor.
- Değişmezlik: aynı kimlik hep aynı baytlar, yani arayüz kimliğe göre
  sınırsız önbellekleyebiliyor.

Karma **kriptografik değil** ve olması gerekmiyor: çakışmanın bedeli yanlış
bir ikon göstermek.

Depo **kendiliğinden küçülmüyor** — kimlik içerikten geliyor ve hiçbir kayıt
"artık bu ikon lazım değil" demiyor. Budama veri temizleme ekranından
(`favicon::buda`, kullanılmayanları siliyor).

### Veri temizleme (Faz 3'te yazıldı)

Altı kalem: geçmiş · favicon · çerez · önbellek · site verisi · otomatik
doldurma. İlk ikisi **bizim** depolarımız, kalan dördü motorun profili
(`ICoreWebView2Profile2::ClearBrowsingData`). Ayrım kullanıcıya
**görünmüyor**: "çerezleri sil" dediğinde çerezlerin yarısının kalması diye
bir şey yok.

İki şey bilerek dışarıda:

- **Yer imleri** hiçbir kalemde silinmiyor — kasten saklanan şeyler.
- **Şifreler** (`PASSWORD_AUTOSAVE`) silinmiyor: Muiren şifre saklamıyor
  (CLAUDE.md kapsam dışı) ve kullanıcının başka bir yöneticiye ait verisini
  silmek bizim işimiz değil.

Sıra önemli: favicon budaması geçmişten **sonra**. Önce yapılsaydı "hâlâ
kullanılan" listesi silinmek üzere olan geçmiş kayıtlarını da sayar ve ikonlar
depoda kalırdı.

Rapor sayı döndürüyor ("3 214 geçmiş kaydı silindi"): "temizlendi" demek,
hiçbir şey silinmediğinde de doğru görünürdü. Motorun silmesi eşzamansız
olduğu için orası yalnız "başlatıldı" diyor — olmayan bir kesinlik iddia
edilmiyor.

## Neden oturum SQLite'ta değil

Oturum saniyede birkaç kez değişebiliyor ve tamamı her seferinde yeniden
yazılıyor. Küçük ve bütün hâlinde okunan bir veri için SQLite işlem yükü
gereksiz. JSON + atomik rename yeterli ve çökme sonrası davranışı öngörülebilir
(`docs/Sekmeler.md`, "Oturum" bölümü).

Geçmiş ise büyüyen, sorgulanan, silinen bir veri — orası SQLite.

## Şema

```sql
CREATE TABLE IF NOT EXISTS gecmis (
    id          INTEGER PRIMARY KEY,
    url         TEXT NOT NULL,
    baslik      TEXT NOT NULL DEFAULT '',
    ziyaret     INTEGER NOT NULL,          -- unix epoch, saniye
    sayac       INTEGER NOT NULL DEFAULT 1,
    alan        TEXT NOT NULL,             -- host, arama ve gruplama için
    baslik_kucuk TEXT NOT NULL DEFAULT ''  -- toLocaleLowerCase("tr") karşılığı
);
CREATE INDEX IF NOT EXISTS gecmis_ziyaret ON gecmis(ziyaret DESC);
CREATE INDEX IF NOT EXISTS gecmis_alan    ON gecmis(alan);

CREATE TABLE IF NOT EXISTS yer_imi (
    id       INTEGER PRIMARY KEY,
    url      TEXT NOT NULL,
    baslik   TEXT NOT NULL,
    klasor   INTEGER REFERENCES yer_imi_klasor(id) ON DELETE CASCADE,
    sira     INTEGER NOT NULL DEFAULT 0,
    eklendi  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS yer_imi_klasor (
    id     INTEGER PRIMARY KEY,
    ad     TEXT NOT NULL,
    ebeveyn INTEGER REFERENCES yer_imi_klasor(id) ON DELETE CASCADE,
    sira   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS surum (
    anahtar TEXT PRIMARY KEY,
    deger   INTEGER NOT NULL
);
```

### `baslik_kucuk` neden ayrı sütun

SQLite'ın `LOWER()` fonksiyonu ASCII dışına dokunmuyor: `"İSTANBUL"` olduğu
gibi kalıyor ve kullanıcı "istanbul" yazınca eşleşmiyor. Türkçe küçültme
Rust tarafında yapılıp ayrı sütuna yazılıyor, arama o sütunda koşuyor.

Aynı tuzak arayüzde de var (CLAUDE.md #10): `toLocaleLowerCase("tr")`.

## Geçiş (`history/gecisler.rs`)

> **Kural (CLAUDE.md #12):** `CREATE TABLE IF NOT EXISTS` var olan tabloyu
> DEĞİŞTİRMİYOR. Kullanıcının diskindeki veritabanı eski şemada kalıyor ve ilk
> sorguda patlıyor. Yeni sütun eklerken şemayı düzenlemek **yetmez**, geçiş de
> yazılır.

```rust
const HEDEF_SURUM: i64 = 1;

pub fn uygula(db: &Connection) -> Result<()> {
    let mevcut = surum_oku(db)?;                 // yoksa 0
    if mevcut < 1 { adim_1(db)?; }               // ilk şema
    // if mevcut < 2 { adim_2(db)?; }            // örn: gecmis.baslik_kucuk eklendi
    surum_yaz(db, HEDEF_SURUM)
}
```

Her adım kendi `ALTER TABLE` ifadelerini taşıyor ve **geri alınabilir olmak
zorunda değil** ama **tekrar çalıştırılabilir** olmalı (yarıda kesilen geçiş).

## Geçmiş budama

Sınırsız geçmiş, yıllar içinde yüzlerce MB'lık bir dosya demek ve arama
yavaşlıyor. Varsayılan saklama süresi 90 gün; uygulama açılışında bir kez,
arka planda budama koşuyor. Kullanıcı süreyi ayarlardan değiştirebiliyor
(`0` = sınırsız, `-1` = geçmiş tutma).

Budamadan sonra `VACUUM` **her seferinde** koşmuyor — pahalı. Ayda bir ya da
dosya boyutu eşiği aşınca.

## Gizli sekme

> **Faz 3'te plandan sapıldı ve sebebi burada.** Bu bölüm "gizli **pencere**,
> ayrı kullanıcı veri klasörü, klasör kapanınca siliniyor" diyordu.
> Uygulanan çözüm farklı: gizli **sekme**, `SetIsInPrivateModeEnabled`.

**Neden pencere değil sekme:** Muiren tek pencerede çalışıyor (`lib.rs`,
pencere düzeni) ve ikinci bir pencere açmak mimarinin dışında. `Ctrl+Shift+N`
kas hafızası aynı tuşta kalıyor, sonuç mimariye uyuyor.

**Neden ayrı kullanıcı veri klasörü değil:** ayrı klasör = ayrı WebView2
ortamı, ve **Faz 1'i bloke eden hata tam olarak buydu** (`docs/Roadmap.md`).
WebView2 aynı kullanıcı veri klasörünü paylaşan ikinci ortamı yalnız
seçenekleri birebir aynıysa yaratıyor; ayrı bir klasör açmak o kuralın
etrafından dolaşıyor ama karşılığında ikinci bir tuzak alanı üretiyor ve
`add_child`in yuttuğu sessiz hata sınıfını geri getiriyor.

`SetIsInPrivateModeEnabled` bir **denetleyici** seçeneği, ortam seçeneği
değil (`wry/src/webview2/mod.rs`, `create_controller`). Yani ortam
seçenekleri — bayrak dizesi ve kullanıcı veri klasörü — aynı kalıyor ve
CLAUDE.md #16 kırılmıyor.

**Bedeli dürüstçe:** InPrivate kipinde çerezler ve önbellek bellekte kalıyor
ve sekme kapanınca gidiyor, ama profil klasörü paylaşıldığı için bu ayrım
motorun kendi güvencesine dayanıyor — ayrı bir klasörün verdiği fiziksel
ayrıma değil. Buna karşılık **ayrı bir browser süreci maliyeti de yok**:
plandaki çözüm gizli pencere başına bir süreç demekti ve bu, bellek iddiası
olan bir programda pahalı bir takas olurdu.

Motorun gizliliği tek başına yetmiyor, çünkü **üç depo bizim** ve motor
onları bilmiyor. Üçü ayrı ayrı kapatıldı (CLAUDE.md #17):

| Depo | Gizli sekmede |
|---|---|
| Geçmiş (`muiren.db`) | yazılmıyor — `ziyaret_ekle` ve `baslik_guncelle` ikisi de |
| Oturum (`oturum.json`) | yazılmıyor; elle eklenen bir kayıt da gizli doğmuyor |
| Favicon (`favicon/`) | yazılmıyor — depoda kalan ikon, gidilen sitenin diskteki kaydı |

Kapatılan sekme geri alınırken (`Ctrl+Shift+T`) **gizli olarak** geri
geliyor: normal bir sekmede açsaydık o adres tam o anda geçmişe yazılırdı.

## Ayarlar (`settings/mod.rs`)

Üç fonksiyon, üçü birlikte değişiyor (CLAUDE.md #13):

```rust
pub struct Settings { /* ... */ }
pub fn duzelt(ham: Settings) -> Settings;   // aralık dışı → sınıra çek
pub fn oku(yol: &Path) -> Settings;         // dosya yok/bozuk → varsayılan
pub fn yaz(yol: &Path, s: &Settings) -> Result<()>;
```

`oku` **asla hata döndürmüyor.** Bozuk ayar dosyası yüzünden tarayıcı
açılmaması kabul edilemez; bozuk dosya yedeklenip varsayılana dönülüyor ve
kullanıcıya bir bildirim çıkıyor.

`duzelt` tarafındaki kritik alanlar:

| Alan | Aralık | Aralık dışında |
|---|---|---|
| `uyku_esigi_sn` | 30 – 86400 | sınıra çekilir |
| `atma_esigi_sn` | `uyku_esigi_sn` – 604800 | uyku eşiğinin altına inemez |
| `uyanik_ust_sinir` | 1 – 200 | sınıra çekilir |
| `gozcu_periyodu_sn` | 2 – 120 | sınıra çekilir |
| `gecmis_saklama_gun` | -1, 0, 1 – 3650 | varsayılana döner |
| `arama_url` | `%s` içeren `https://` şablonu | varsayılana döner |
| `tema` | kurulu tema adı | yoksa `"Mui"` |
| `surec_politikasi` | `Birlesik` \| `Varsayilan` | varsayılana döner |
| `renderer_tavani` | 0 (sınırsız) veya 4 – 64 | sınıra çekilir |
| `indirme_politikasi` | `Muiget` \| `Sor` \| `Motorda` | — (enum) |
| `engelleme_acik` | bool, varsayılan **açık** | — |
| `filtre_acik` | bool, varsayılan **kapalı** | — |
| `filtre_kurallari` | satır listesi | boş satırlar atılır, **kural metni ayrıştırılmaz** |
| `oyun_algilama` | bool, varsayılan **kapalı** | — |

`filtre_kurallari` neden `duzelt` içinde ayrıştırılmıyor: ayrıştırma
`engel/liste.rs` içinde ve orası anlaşılmayan satırları kullanıcıya **geri
gösteriyor**. Burada sessizce elemek, kullanıcının yazdığı kuralın kaybolması
demek olurdu. `duzelt` yalnız boş satırları atıyor.

Son üç bool'un varsayılanları bilinçli ve gerekçeleri ayrı: engelleme bir
*politika* (kullanıcının bir şey yazmasını gerektirmiyor, o yüzden açık),
filtre bir *liste* (karar #5: listeler kullanıcı tarafından konuyor, kutudan
liste çıkmıyor), oyun algılaması bir *sezgi* (yanlış pozitifin bedeli
kullanıcının baktığı sekmelerin sebepsiz uyuması).

`arama_url` iki koşulu birden istiyor: şablonda `%s` **ve** `https://` şeması.
`%s` yoksa arama kutusu sessizce hep aynı sayfayı açardı; `http://` ise arama
sorgusunu düz metin olarak ağa verirdi. İkisi de kullanıcının fark etmeyeceği
türden hatalar, o yüzden `duzelt` sessizce varsayılana dönüyor.

### `surec_politikasi` — yeniden başlatma gerektiriyor

Diğer ayarların hepsi anında uygulanıyor; bu ikisi **Chromium bayraklarına**
dönüştüğü için (`docs/Setup.md`, `EK_BAYRAKLAR`) ancak WebView2 ortamı yeniden
kurulduğunda etkili oluyor. Yani tarayıcının yeniden başlatılması gerekiyor.

Ayarlar ekranı bunu açıkça söylüyor ve değişiklik kaydedildiğinde "yeniden
başlatınca etkili olacak" rozeti çıkıyor. Sessizce etkisiz kalan bir ayar,
kullanıcının özelliğin çalışmadığını sanmasına yol açıyor.

| Değer | Anlamı | Bedeli |
|---|---|---|
| `Birlesik` (varsayılan) | `--process-per-site` açık | Bir sitenin sekmesi çökerse o sitenin tüm sekmeleri gider — URL'den geri geliyorlar |
| `Varsayilan` | Chromium'un kendi modeli | Daha çok RAM, daha iyi çökme yalıtımı |

`renderer_tavani = 0` ise `--renderer-process-limit` hiç geçirilmiyor.
Düşük tavan (4–6) belleği daha çok düşürüyor ama sekmeler CPU'da sıraya
giriyor; 8 GB profilinde makul, 32 GB'da gereksiz.

`atma_esigi_sn < uyku_esigi_sn` durumu özellikle önemli: sekmeler uyumadan
atılırdı, yani kullanıcı her sekme değişiminde sayfa yeniden yükleniyor
sanırdı. Sessiz ve çok sinir bozucu bir hata; `duzelt` bunu kapatıyor.

## Veri temizleme

Ayarlar → "Verileri temizle" ekranı. Kalemler ayrı ayrı seçilebiliyor:

| Kalem | Ne siliniyor |
|---|---|
| Geçmiş | `gecmis` tablosu |
| Çerezler ve site verisi | WebView2 profil temizliği |
| Önbellek | WebView2 önbelleği |
| Faviconlar | `favicon/` |
| Oturum | `oturum.json` |

Yer imleri ve temalar bu ekranda **yok** — onlar kullanıcının kendi ürettiği
içerik, "temizle" düğmesiyle kaybolmamalı. Silinmeleri ayrı ve tek tek.
