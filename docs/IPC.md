# IPC — Komut ve Olay Sözleşmesi

Bu dosya **sözleşme**. Yeni bir komut ya da olay önce buraya yazılıyor, sonra
Rust ve TypeScript tarafı ona uyduruluyor (CLAUDE.md kural #3).

## Biçim kuralları

- Komut adları Türkçe, `snake_case`: `sekme_ac`, `bellek_ozeti`.
- Olay adları `muiren://` şemasıyla: `muiren://sekme-degisti`.
- Argüman ve dönüş yapıları `#[serde(rename_all = "camelCase")]` — TypeScript
  tarafında `camelCase` görünüyorlar.
- Her komutun TypeScript karşılığı `src/ipc/` içinde bir sarmalayıcı.
  **Bileşenler `invoke` çağırmıyor.**
- Hata: `Result<T, MuirenHata>`. `MuirenHata` serileştirilebilir bir enum;
  arayüz mesajı kendi çeviriyor, backend Türkçe metin göndermiyor.

---

## Komutlar — Sekme

| Komut | Argüman | Dönüş | Not |
|---|---|---|---|
| `sekme_ac` | `url: Option<String>`, `ebeveyn: Option<SekmeId>`, `arkaplanda: bool`, `gizli: bool` | `SekmeId` | `url` yoksa `muiren://yeni`; `gizli` InPrivate |
| `sekme_kapat` | `id: SekmeId` | `Option<SekmeId>` | dönüş: yeni etkin sekme |
| `sekme_etkinlestir` | `id: SekmeId` | `()` | uyuyansa uyandırır, atılmışsa yükler |
| `sekme_tasi` | `id: SekmeId`, `hedef: usize` | `()` | çocuklar birlikte taşınır |
| `sekme_sabitle` | `id: SekmeId`, `sabit: bool` | `()` | |
| `sekme_sessize_al` | `id: SekmeId`, `sessiz: bool` | `()` | sessize alınan koruma kaybeder |
| `sekme_uyutma_istisnasi` | `id: SekmeId`, `istisna: bool` | `()` | |
| `sekme_listesi` | — | `Vec<SekmeOzeti>` | uyuyan sekmeyi **uyandırmaz** |
| `sekme_geri_al` | — | `Option<SekmeId>` | son kapatılan; gizliyse gizli döner |

### `gizli` — gizli sekme

Chrome gizli **pencere** açıyor; Muiren tek pencerede çalıştığı için karşılığı
bir **sekme** (`Ctrl+Shift+N`). Webview
`ICoreWebView2ControllerOptions::SetIsInPrivateModeEnabled` ile doğuyor.

Bu bir **denetleyici** seçeneği, ortam seçeneği değil — yani CLAUDE.md #16'yı
bozmuyor: bayrak dizesi ve kullanıcı veri klasörü aynı kalıyor. Gizli sekme
için ayrı bir veri klasörü **açılmıyor**; açsaydık ikinci bir WebView2 ortamı
kurulur ve Faz 1'i bloke eden hataya davetiye çıkardı.

Motorun gizliliği tek başına yetmiyor, çünkü üç depo **bizim**:

| Depo | Gizli sekmede |
|---|---|
| Geçmiş (SQLite) | yazılmıyor |
| Oturum dosyası | yazılmıyor |
| Favicon deposu | yazılmıyor |

Sekme çubuğunda ayrı bir işaretle çiziliyor: kullanıcının hangi sekmenin gizli
olduğunu **görmesi** gerekiyor, yoksa gizli sandığı sekmede geçmişe yazan bir
arama yapıyor.

## Komutlar — Gruplar

| Komut | Argüman | Dönüş | Not |
|---|---|---|---|
| `grup_listesi` | — | `Vec<Grup>` | |
| `grup_ac` | `ad: String` | `GrupId` | boş ad kabul; arayüz "Adsız grup" yazıyor |
| `grup_sil` | `id: GrupId` | `()` | **sekmeler kalıyor**, gruptan çıkıyorlar |
| `grup_ata` | `sekme: SekmeId`, `grup: Option<GrupId>` | `()` | `None` gruptan çıkarıyor |
| `grup_guncelle` | `id`, `ad`, `renk`, `katli`, `uyku_esigi_sn` — hepsi `Option` | `()` | verilmeyen alan değişmiyor |

```rust
pub struct Grup {
    pub id: GrupId,
    pub ad: String,
    pub renk: GrupRengi,        // enum: teal|mor|kehribar|kirmizi|yesil|mavi|gri
    pub katli: bool,
    pub uyku_esigi_sn: Option<u64>,  // None = genel ayar
}
```

`renk` neden bir **enum**: grup rengi arayüzde bir CSS değerine dönüşüyor.
Kullanıcının yazdığı bir dizeyi oraya bırakmak, `theme/jeton.rs` içinde uzun
uzun kapatılan kapıyı yan taraftan açmak olurdu.

`uyku_esigi_sn` neden **iki katmanlı `Option`** (`Option<Option<u64>>`
argümanda): "dokunma" ile "genel ayara dön" farklı şeyler. Grubun eşiğini
ayarların eşiğine eşitleyerek kopyalasaydık, kullanıcı genel eşiği
değiştirdiğinde grupları eski değerde kalır ve sebebini bulamazdı.

Grup eşiği **atma eşiğini de ölçekliyor**: ayarlardaki uyku/atma oranı
korunuyor. Ölçeklenmeseydi "1 dakikada uyusun" diyen grubun sekmeleri
59 dakika boyunca uyanık-ama-uyuyor durumunda beklerdi (`memory/esik.rs`).

## Komutlar — Gezinme

| Komut | Argüman | Dönüş |
|---|---|---|
| `gezin` | `id: SekmeId`, `girdi: String` | `()` |
| `geri` / `ileri` / `yenile` / `durdur` | `id: SekmeId` | `()` |

`gezin` komutunun `girdi` alanı ham kullanıcı metni. URL mi arama mı ayrımı
**arayüzde** yapılıyor (`src/lib/url.ts`, saf ve testli) çünkü kullanıcı yazarken
canlı geri bildirim gerekiyor; backend gelen dizeyi yine de doğruluyor.

## Komutlar — Bellek

| Komut | Argüman | Dönüş | Not |
|---|---|---|---|
| `bellek_ozeti` | — | `BellekOzeti` | sistem baskısı + durum sayıları + toplam |
| `sekme_uyut` | `id: SekmeId` | `()` | elle uyutma |
| `sekme_at` | `id: SekmeId` | `()` | elle atma |
| `hepsini_uyut` | — | `u32` | korumalılar hariç; dönüş: uyutulan sayısı |
| `oyun_modu` | `acik: bool` | `()` | elle tetik (Muifly de tetikliyor) |

```rust
pub struct BellekOzeti {
    pub baski: BellekBaskisi,
    pub toplam_mb: u64,          // WebView2 süreçlerinin toplamı
    pub kabuk_mb: u64,
    pub sistem_toplam_mb: u64,   // baskı çubuğunun kaynağı
    pub sistem_bos_mb: u64,
    pub etkin: u32,
    pub arkaplan: u32,
    pub uyuyan: u32,
    pub atilmis: u32,
    pub tahmini_kazanc_mb: u64,  // ölçülen kazanç + ölçülemeyenlerin tahmini
    pub olcum_yaklasik: bool,    // süreç→sekme eşlemesi güvenilmezse true
    pub sekme_mb: Vec<SekmeBellegi>, // sekme başına ölçüm (memory/esleme.rs)
    pub ortak_mb: u64,           // sekmeye düşmeyen WebView2 süreçleri
                                 // (tarayıcı, GPU, ağ + kabuğun kendi payı)
    pub surec_sayisi: u32,       // Faz 0/R4 ölçüm aracı (docs/Setup.md)
}

pub struct SekmeBellegi {
    pub id: SekmeId,
    pub mb: u64,
    pub paylasimli: bool,        // render süreci bölüşülüyor → rakam bölündü
    pub bayat: bool,             // bu turda ölçülmedi; son bilinen değer
}
```

`tahmini_kazanc_mb` arayüzde "Muiren şu an ~3.2 GB tasarruf ediyor" olarak
gösteriliyor. `olcum_yaklasik` true ise "~" işareti ve bir ipucu ekleniyor —
**yaklaşık değeri kesin gibi göstermek yok** (`docs/Bellek.md`).

### `sekme_mb` neden `SekmeOzeti` içinde değil

Sekme başına bellek, sekmeyi anlatan yapıya (`SekmeOzeti`) konabilirdi ve
arayüzde iki listeyi birleştirmek gerekmezdi. Konmadı, çünkü `SekmeOzeti`
aynı zamanda **eşik kararının girdisi** (`memory/esik.rs`) ve o karar sekme
başına rakama dayanmıyor — boşta kalma süresine ve sistem baskısına dayanıyor
(`docs/Bellek.md`, "Ölçüm"). Alanı oraya koymak, saf karara bir gün sızacak
bir veriyi elinin altına bırakmak olurdu; panelin ihtiyacı olan yer ise zaten
`BellekOzeti`.

`bayat` ve `paylasimli` **iki ayrı belirsizlik** ve arayüz ikisini
karıştırmıyor: paylaşımlı rakam bugün ölçüldü ama bölüştürüldü, bayat rakam
bölünmedi ama bugüne ait değil. Tek bir "~" ikisini aynı kefeye koyardı.

Değeri **olmayan** sekme listede yok. Sıfırla listelemek, ölçümün yokluğunu
bir ölçüm sonucu gibi göstermek olurdu.

### `gecikme_ozeti` — kabul tablosunun ölçüm ucu

`docs/Bellek.md` başarı tablosunun iki satırı ("uyuyan sekmeye tıklayıp
etkileşime hazır olma < 300 ms", "atılmış sekmeye tıklayıp ilk boya < 2 sn")
uzun süre **ölçülemiyordu**: gecikmeyi sayan bir yer yoktu ve "hızlı olduğunu
hissediyorum" kabul edilmiyor.

| Komut | Argüman | Dönüş | Not |
|---|---|---|---|
| `gecikme_ozeti` | — | `GecikmeOzeti` | son 64 uyanmanın dağılımı |
| `gecikme_sifirla` | — | `()` | ölçüm oturumu başlatmak için defteri boşaltır |

```rust
pub struct GecikmeOzeti {
    pub uyuyan: Option<Dagilim>,   // Uyuyan → etkileşime hazır
    pub atilmis: Option<Dagilim>,  // Atilmis → yükleme bitti
    pub hedef_uyuyan_ms: u32,      // 300  — docs/Bellek.md kabul tablosu
    pub hedef_atilmis_ms: u32,     // 2000
}

pub struct Dagilim {
    pub n: u32,
    pub medyan_ms: u32,
    pub p95_ms: u32,
    pub en_kotu_ms: u32,
}
```

`Option` çünkü **örnek yokken sıfır göstermek yok**: hiç sekme uyandırmamış bir
kullanıcıya "medyan 0 ms" demek, ölçülmemiş bir şeyi ölçülmüş gibi göstermek
olurdu. Arayüz `None` gördüğünde "henüz ölçülmedi" yazıyor.

**İki ölçüm iki farklı şeyi sayıyor ve bu etiketleniyor:**

- `uyuyan` — kronometre `Resume` çağrısından **önce** başlıyor, sayfanın JS ana
  iş parçacığı bizim betiğimizi çalıştırdığında duruyor. Yani ölçülen şey
  gerçekten "etkileşime hazır": render süreci uyandı ve kullanıcının tuşunu
  işleyecek iş parçacığı boşta.
- `atilmis` — kronometre webview yaratılmadan önce başlıyor, `NavigationCompleted`
  ile duruyor. Bu **ilk boya değil, ondan sonrası**: yüklemenin bitişi. Yani
  ölçülen sayı gerçek ilk boyanın **üst sınırı** ve eşiği geçiyorsa kriter
  kesin olarak sağlanıyor. Alt sınırı olduğunu iddia etmiyoruz; WebView2 bize
  bir ilk boya olayı vermiyor ve olmayan bir olayı varmış gibi göstermektense
  dürüst bir üst sınır veriliyor. Ağ süresi **dahil** (`docs/Bellek.md` bunu
  şart koşuyor: "ağ hariç tutulmaz, not düşülür").

Başarısız gezinme örnek üretmiyor (CLAUDE.md #7) ve 30 saniyeyi aşan bir
kronometre kaydedilmeden atılıyor: bir kez ağı kopmuş kullanıcının 5 dakikalık
örneği medyanı kalıcı olarak zehirlerdi.


## Komutlar — Geçmiş, Yer İmi

| Komut | Argüman | Dönüş |
|---|---|---|
| `gecmis_ara` | `sorgu: String`, `limit: u32` | `Vec<GecmisKaydi>` |
| `gecmis_sil` | `id: i64` | `()` |
| `gecmis_temizle` | `aralik: Aralik` | `u64` |
| `yer_imi_ekle` | `url: String`, `baslik: String`, `klasor: Option<i64>` | `i64` |
| `yer_imi_sil` | `id: i64` | `()` |
| `yer_imi_listesi` | `klasor: Option<i64>` | `Vec<YerImi>` |
| `yer_imi_mi` | `url: String` | `Option<i64>` |
| `yer_imi_klasor_ekle` | `ad: String`, `ebeveyn: Option<i64>` | `i64` |
| `yer_imi_klasorleri` | — | `Vec<YerImiKlasor>` |

`yer_imi_mi` **kimliği** döndürüyor, `bool` değil: adres çubuğundaki yıldız
hem dolu görünmek hem tıklandığında silmek zorunda ve ikinci bir arama turu
atmanın karşılığı yok. Geçmiş deposu hiç açılamadıysa (disk hatası) üçü de
boş cevap veriyor — yer imi tutamayan bir tarayıcı hâlâ gezinebilmeli.

## Komutlar — Kabuk

| Komut | Argüman | Dönüş | Not |
|---|---|---|---|
| `icerik_alani` | `alan: Dikdortgen` | `()` | kabuğun altında kalan bölge |
| `yetenekler` | — | `Yetenekler` | motorun gerçekten desteklediği çağrılar |
| `kisayol_bas` | `tus: String`, `ctrl: bool`, `shift: bool`, `alt: bool` | `bool` | dönüş: kısayol tanındı mı |
| `ortu_gorunur` | `acik: bool` | `()` | tam ekran örtü açıldı/kapandı |

### `ortu_gorunur` — örtü açıkken sekme webview'i gizleniyor

Sekme webview'i ayrı bir native pencere ve kabuğun **üstünde** duruyor; kabuğun
çizdiği bir örtü (ayarlar ekranı, sekme arama, teşhis) sayfanın altında kalıyor
ve `z-index` iki ayrı pencerenin sırasını değiştiremiyor. Bu yüzden örtü
açılınca webview gizleniyor, kapanınca aynı alana geri geliyor.

**Gizlemek uyutmak değil:** sekme durumu ve sayfanın kendisi olduğu gibi
duruyor, bir `TrySuspend` çağrılmıyor, gecikme defterine bir uyanma
yazılmıyor.

> Yeni bir tam ekran örtü eklendiğinde `App.tsx` içindeki `ortuAcik`
> ifadesine de eklenmesi gerekiyor (CLAUDE.md #21) — eklenmeyen örtü sessizce
> görünmez oluyor.

### `kisayol_bas` — kısayol tablosunun tek kaynağı

`docs/Frontend.md` şunu söylüyor: webview odaktayken tuşlar kabuğa ulaşmıyor,
dolayısıyla her kısayolun **iki karşılığı** olmak zorunda. Bunu iki ayrı
kısayol tablosu yazarak yapmıyoruz — iki tablo sessizce ayrışır ve sonuç
"bazen çalışmıyor" diye rapor edilir.

Bunun yerine **tablo tek yerde**, `src-tauri/src/kisayol.rs` içinde ve saf.
Ona iki kapıdan giriliyor:

1. **Kabuk odaktayken** — React `keydown` dinleyicisi `kisayol_bas` komutunu
   çağırıyor.
2. **Sayfa odaktayken** — `motor/gercek.rs` içindeki
   `ICoreWebView2Controller::add_AcceleratorKeyPressed` kaydı aynı çözümleyiciyi
   çağırıyor.

Kısayolun **kimin işi** olduğu da tabloda: bir kısmı backend eylemi (sekme aç,
uyut), bir kısmı yalnız arayüzü ilgilendiriyor (adres çubuğuna odaklan, paneli
aç). İkincisi için backend `muiren://kisayol` olayı yayınlıyor ve arayüz onu
uyguluyor — böylece sayfa odaktayken de `Ctrl+L` çalışıyor.

```rust
pub struct Dikdortgen { pub x: f64, pub y: f64, pub genislik: f64, pub yukseklik: f64 }

pub struct Yetenekler {
    pub motor: bool,           // motorsuz derleme mi
    pub askiya_alma: bool,     // ICoreWebView2_3: TrySuspend / Resume
    pub bellek_hedefi: bool,   // ICoreWebView2_19
    pub surec_bilgisi: bool,   // Environment13 + ICoreWebView2_20: süreç → sekme
    pub indirme_olayi: bool,   // ICoreWebView2_4: DownloadStarting
    pub favicon: bool,         // ICoreWebView2_15: FaviconChanged + GetFavicon
}
```

`indirme_olayi` yoksa Muiget köprüsü indirmeyi yakalayamıyor ve motor kendi
indirmesini yapıyor — karar #4'ün "kullanıcı indirme yapamaz duruma düşmüyor"
yarısı. `favicon` yoksa sekme çubuğunda yalnız durum noktası kalıyor.

`surec_bilgisi` **iki** arayüz istiyor ve tek bayrakla temsil edilmesi
bilinçli: `ICoreWebView2Environment13::GetProcessExtendedInfos` bir süreçte
hangi çerçevelerin olduğunu söylüyor, `ICoreWebView2_20::FrameId` hangi
çerçevenin hangi sekme olduğunu. Biri olmadan diğerinin karşılığı yok, yani
"yarı çalışan bir eşleme" diye bir durum yok. Yoksa `sekme_mb` boş geliyor ve
`olcum_yaklasik` true kalıyor.

`icerik_alani` neden var: sekme webview'i ayrı bir native pencere, CSS onu
yerleştirmiyor — koordinat gerekiyor. Ölçüyü arayüz biliyor (kabuk yüksekliği,
yan panelin açık olup olmadığı, tam ekran) ve backend'e **veri** olarak
gönderiyor. Bu bir karar değil bir ölçüm; CLAUDE.md #2'ye aykırı değil.
Backend sabit bir yükseklik tutsaydı yan panel açıldığında sayfa panelin
altında kalırdı.

`yetenekler` neden var: eksik yetenek bir hata değil, eski bir WebView2
Runtime (`docs/Setup.md`). Arayüz buna bakıp yapamayacağı işin düğmesini
çizmiyor — sessizce tıklanan ölü düğme, kullanıcıya "bu program bozuk"
dedirtiyor.

## Komutlar — Tema ve Ayarlar

| Komut | Argüman | Dönüş |
|---|---|---|
| `tema_listesi` | — | `Vec<TemaOzeti>` |
| `tema_yukle` | `dosya_yolu: String` | `TemaOzeti` |
| `tema_uygula` | `ad: String` | `()` |
| `tema_sil` | `ad: String` | `()` |
| `tema_disa_aktar` | `ad: String`, `hedef: String` | `()` |
| `tema_etkin` | — | `Option<TemaOzeti>` |
| `ayarlar_oku` | — | `Settings` |
| `ayarlar_yaz` | `ayarlar: Settings` | `Settings` |

`ayarlar_yaz` **düzeltilmiş** ayarları geri döndürüyor. Kullanıcı uyku eşiğine
0 yazarsa `duzelt` onu alt sınıra çekiyor ve arayüz düzeltilmiş değeri
gösteriyor — sessizce farklı bir değerle çalışmak yok (CLAUDE.md #13).

`Settings` **eksik anahtara dayanıklı** (`#[serde(default)]`): sürüm
yükseltmede eklenen yeni bir alan, kullanıcının bütün ayar dosyasını `.bozuk`
yapıp sıfırlamıyor, yalnız o alan varsayılana düşüyor. Yeni alan eklerken üç
yer birden değişiyor — `Settings`, `varsayilan`, `duzelt` (CLAUDE.md #13) — ve
karşılığı `src/ipc/tipler.ts` içine yazılıyor.

İki alan pencereye dokunuyor ve ikisi de **varsayılan kapalı**:
`oyun_algilama` (tam ekran oyun sezgisi) ve `oyun_pencere_gizle` (oyun modunda
kabuk penceresini **simge durumuna alma** — gizleme değil; gerekçesi
`docs/Bellek.md`, "Oyun modu" 4. adım).

## Komutlar — Köprüler

| Komut | Argüman | Dönüş | Not |
|---|---|---|---|
| `kopru_durumu` | — | `Vec<KopruDurumu>` | hangi kardeş kurulu |
| `kopru_tazele` | — | `Vec<KopruDurumu>` | önbelleği düşürüp yeniden arar |
| `muiget_gonder` | `url: String`, `dosya_adi: Option<String>`, `kaynak_sayfa: Option<String>` | `()` | ad **temizlenerek** gidiyor |
| `indirme_izin_ver` | `id: SekmeId`, `url: String` | `()` | motorun kendi indirmesine tek atımlık izin |
| `muiply_ac` | `yol: String` | `()` | `file://` adresi de kabul |
| `muiwatch_bagla` | `id: SekmeId`, `oda: String` | `()` | sekmeyi korumaya alıyor (kural #7) |
| `muiwatch_birak` | `id: SekmeId` | `()` | korumayı kaldırıyor |
| `favicon_oku` | `kimlik: String` | `Option<String>` | `data:image/png;base64,…` |

```rust
pub struct KopruDurumu {
    pub ad: &'static str,
    pub kurulu: bool,
    /// Teşhis için; arayüzde yalnız ipucunda.
    pub yol: Option<String>,
}
```

`kurulu` false ise arayüz o menü öğesini **hiç çizmiyor** (`docs/Kopruler.md`,
genel kural). Soluk bir öğe göstermek, tıklanamayan bir düğme sunmak olurdu.

`favicon_oku` neden bir **kimlik** alıyor, adres değil: sekme özeti favicon
adresini taşısaydı, açılan her site kabuk penceresine bir ağ isteği
yaptırabilirdi. Kimlik içeriğin karması — aynı kimlik hep aynı baytlar, yani
arayüz sınırsız önbellekleyebiliyor (`src-tauri/src/favicon/mod.rs`).

`indirme_izin_ver` neden var: `IndirmePolitikasi::Sor` altında motor indirmeyi
**iptal** ediyor ve iptal edilmiş bir WebView2 indirmesi sürdürülemiyor.
Kullanıcı "Muiren indirsin" derse adres tek atımlık bir bilete yazılıyor ve
sekme aynı adrese yeniden gönderiliyor. Bilet tek atımlık: kalıcı olsaydı bir
kez verilen izin, aynı adresin sonraki bütün indirmelerinde sessizce geçerli
olurdu.

## Komutlar — Engelleme ve veri temizleme

| Komut | Argüman | Dönüş | Not |
|---|---|---|---|
| `engel_ozeti` | — | `EngelOzeti` | kaç kural derlendi, hangileri anlaşılmadı |
| `engel_sayaci` | `id: SekmeId` | `u32` | bu **sayfada** engellenen istek |
| `veri_temizle` | `istek: Temizlik` | `TemizlikRaporu` | motor profili + bizim depolar |
| `tema_arkaplan` | — | `Option<String>` | etkin temanın arka planı, `data:` |

```rust
pub struct EngelOzeti {
    pub filtre_acik: bool,
    pub popup_acik: bool,
    pub kural_sayisi: usize,
    /// Ayrıştırılamayan satırlar — sessizce atılmıyorlar.
    pub anlasilmayanlar: Vec<String>,
}

pub struct Temizlik {
    pub gecmis: bool,       // bizim SQLite; YER İMLERİ DAHİL DEĞİL
    pub favicon: bool,      // bizim depo
    pub cerezler: bool,     // motor profili
    pub onbellek: bool,
    pub site_verisi: bool,  // DOM storage + IndexedDB + Service Worker
    pub otomatik_doldurma: bool,  // şifreler DAHİL DEĞİL
}

pub struct TemizlikRaporu {
    pub gecmis_kaydi: u64,
    pub favicon_dosyasi: u64,
    /// Motorun silmesi eşzamansız: yalnız "başlatıldı".
    pub motor_baslatildi: bool,
}
```

`anlasilmayanlar` neden döndürülüyor: kullanıcının yazdığı bir kuralı sessizce
atmak, çalışmayan bir listeyle dolaşması demek. Ayarlar ekranı "12 satır
anlaşılmadı" diyor.

`TemizlikRaporu` neden sayı taşıyor: "temizlendi" demek, hiçbir şey
silinmediğinde de doğru görünürdü.

`tema_arkaplan` neden `TemaOzeti` içinde **değil**: görsel megabaytlarca
olabiliyor ve özet her tema listesinde, her `tema-degisti` olayında gidiyor.
Oraya koymak, tema listesini açan kullanıcıya üç duvar kâğıdını birden
göndermek olurdu. Adres bir `data:` URL'i, dosya yolu değil — kabuğa dosya
sistemi kanalı (Tauri `asset` protokolü) açmamak için, favicon deposuyla aynı
gerekçe.

---

## Olaylar (backend → arayüz)

| Olay | Ne zaman | Yük |
|---|---|---|
| `muiren://sekme-degisti` | sekme eklendi/kapandı/taşındı/etkin değişti | `Vec<SekmeOzeti>` (tam liste) |
| `muiren://sekme-guncellendi` | başlık, favicon, yükleme durumu, ses | `SekmeOzeti` (tek) |
| `muiren://sekme-durum-degisti` | gözcü uyuttu/attı/uyandırdı | `{ id, durum, sebep }` |
| `muiren://bellek-ozeti` | gözcü her turda | `BellekOzeti` |
| `muiren://gezinme` | gezinme başladı/bitti/hata | `{ id, asama, url, basarili }` |
| `muiren://indirme-onerisi` | motorun indirme olayı yakalandı | `{ id, url, dosyaAdi, boyut, sonuc }` |
| `muiren://tema-degisti` | tema uygulandı | `TemaOzeti` (doğrulanmış jetonlar) |
| `muiren://kopru-durumu` | kardeş uygulama bulundu/kayboldu | `Vec<KopruDurumu>` |
| `muiren://oyun-modu` | oyun modu açıldı/kapandı | `{ acik, kaynak }` |
| `muiren://engellendi` | pencere/yönlendirme/istek engellendi | `{ id, url, sebep, sayi }` |
| `muiren://kisayol` | sayfa odaktayken arayüz işi bir kısayola basıldı | `{ is }` |

### `indirme-onerisi` — üç an, tek olay

`sonuc` alanı: `soruluyor` · `devredildi` · `basarisiz`. Üçü tek olayda çünkü
arayüzün üç ayrı dinleyici yazması, birinin unutulmasıyla sessizce eksik
kalırdı.

### `oyun-modu` — `kaynak` alanı

`elle` (kullanıcının düğmesi) ya da `algilama` (kendi sezgimiz).

`muifly` diye bir kaynak **yok ve olmayacak**: Faz 4'te bakıldı, `muifly://durum`
süreç içi bir Tauri olayı ve Muifly'da süreçler arası bir uç bulunmuyor
(`docs/Kopruler.md`, "Muifly").

### `engellendi` — sessiz engelleme yok

`sebep`: `popUp` · `yonlendirme` · `filtre`. `sayi` bu **sayfada** engellenen
toplam; gezinme başlayınca sıfırlanıyor.

Adres çubuğunda bir rozet olarak görünüyor. Sessiz engelleme kabul edilmiyor:
bazı siteler ödeme ya da giriş akışını pop-up'ta açıyor ve sessizce engellenen
bir pencere, kullanıcıya "tarayıcı bozuk" dedirtiyor.

**Alt kaynak engellemesinde olay yayınlanmıyor**, yalnız sayaç artıyor: bir
sayfada yüzlerce olabiliyor ve arayüzü olay yağmuruna tutmanın karşılığı yok.

### Neden `sekme-degisti` tam liste gönderiyor

Kısmi güncelleme (delta) daha ucuz ama sıra değişimlerinde arayüzle backend'in
sekme sırası ayrışabiliyor ve bu ayrışma sessiz. Sekme kayıtları küçük
(`docs/Sekmeler.md`) — 200 sekmelik tam liste birkaç yüz KB JSON, saniyede bir
gönderilmiyor zaten. Doğruluk burada ucuz.

`sekme-guncellendi` ise sık (her başlık değişiminde) ve tek sekme taşıyor.

### `sebep` alanı

`sekme-durum-degisti` olayının `sebep` alanı arayüzde ipucu olarak
gösteriliyor: `BostaKaldi` · `SistemBaskisi` · `OyunModu` · `PencereGizli` ·
`Elle` · `UyanikSinir`. Kullanıcı sekmesinin neden uyuduğunu bilmezse özelliği
kapatıyor; sebebi görünce ayarı değiştiriyor.

`PencereGizli` ile `SistemBaskisi` **ayrı** tutuluyor, ikisi de eşikleri ÷ 4
yapsa bile: biri "makinende bellek daralıyor", diğeri "sen bakmıyordun, ben de
bekletmedim" demek. Tek sebebe indirilseydi pencereyi küçülten kullanıcı,
makinesinde olmayan bir bellek sorunu arardı. Aynı sebeple sıra da belli:
**gerçek** baskı yüksekse sebep `SistemBaskisi`, pencere gizli olsa bile —
kullanıcının bilmesi gereken önce o.

---

## Sözleşme dışı: sayfadan gelen mesajlar

Web sayfalarının `window.chrome.webview.postMessage` benzeri kanallara
erişimi **kapalı**. Muiren sayfaya hiçbir ayrıcalıklı köprü açmıyor.

Tek istisna kabuğun kendi sayfaları (`muiren://yeni`, ayarlar, teşhis) ve
onlar da ayrı bir kaynak/kanal üzerinden konuşuyor. Web içeriğine açılan her
ayrıcalıklı kanal, açılan her sitenin o kanala erişmesi demek.
