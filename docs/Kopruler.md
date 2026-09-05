# Köprüler — Mui Ailesiyle Entegrasyon

Bu dosya Muiren'i "bir Chrome kabuğu daha" olmaktan kurtaran kısım. Teknik
olarak da en ucuz kısım, çünkü **karşı taraflar zaten yazılmış**.

## Genel kural

Köprüler **isteğe bağlı**. Kardeş uygulama kurulu değilse Muiren o özelliği
sessizce gizliyor; hata vermiyor, indirme önerisi zorlamıyor. Muiren tek başına
çalışan bir tarayıcı; köprüler onu ailenin içinde daha iyi yapıyor.

Kardeş uygulamanın kurulu olup olmadığı: kayıt defterinde kurulum anahtarı →
yoksa bilinen kurulum yolları → yoksa "kurulu değil". Sonuç önbelleğe alınıyor,
her sağ tıkta disk taranmıyor.

**Hiçbir köprü çerez, oturum başlığı, kimlik bilgisi veya sayfa içeriği
göndermiyor.** Gönderilen şey her zaman bir URL ya da bir dosya yolu.

---

## Muiget — İndirme

**Karşı taraf hazır:** Muiget'in `extension_bridge/native_host.rs` dosyası
Chrome uzantısıyla **length-prefixed JSON** (4 bayt uzunluk öneki + gövde)
konuşuyor. Mesaj biçimi `ExtensionMessage` / `HostResponse` enum'ları
(`#[serde(tag = "type", rename_all = "camelCase")]`).

**Kararımız: yeni protokol icat etmiyoruz.** Muiren de aynı native host
ikilisini çalıştırıp aynı protokolü konuşuyor. Kazanç:

- Muiget tarafında **tek satır** değişiklik gerekmiyor.
- Protokol zaten test edilmiş ve `Muiget/docs/decisions.md` #6 içinde
  gerekçelendirilmiş.
- Muiren kendi indirme motorunu yazmıyor (kapsam dışı, CLAUDE.md).

Akış:

```
Muiren'de indirilebilir bağlantı / motorun indirme olayı
  → motor indirmeyi İPTAL ediyor (WebView2 kendi indirmesini başlatmıyor)
  → bridge/muiget.rs
     → native host süreci (yoksa başlatılıyor, boştaysa kapatılıyor)
     → { "type": "download", "url": "...", "fileName": "..." }
  ← { "type": ... } yanıtı
  → arayüzde "Muiget'e gönderildi" bildirimi
```

Dikkat:

- `fileName` **sayfadan geliyor** — güvenilmez. Yol ayırıcı, `..`, kontrol
  karakteri temizlenmeden gönderilmiyor (`docs/Sekmeler.md`, aynı kural).
- Muiget kurulu değilse motorun kendi indirmesi devrede kalıyor. Kullanıcı
  indirme yapamaz duruma düşmüyor.
- Kullanıcı "her indirme Muiget'e" ya da "her seferinde sor" seçebiliyor
  (`settings`).
- Native host'un beklediği çağırma biçimi Muiget'in kurulumunda kayıtlı;
  Muiren onu **kendi kaydı olarak eklemiyor**, doğrudan ikiliyi çalıştırıyor.
  (Muiget `docs/worklog.md` içinde native messaging manifestinde argüman alanı
  olmadığı not edilmiş — bu yüzden argüman geçmeye çalışılmıyor.)

### DOĞRULANDI (Faz 3) — Muiget'te değişiklik gerekmiyor

Bırakılan soru şuydu: *"native host ikilisini uzantı dışından, doğrudan çocuk
süreç olarak çalıştırmak Muiget tarafında bir kimlik doğrulaması gerektiriyor
mu?"*

`Muiget/src-tauri/src/extension_bridge/native_host.rs` okundu. Üç bulgu:

1. **Ayrı bir host ikilisi yok.** Host, Muiget'in kendisi:
   `muiget.exe --native-host`. `is_host_invocation` bu bayrağı tek başına
   köprü işareti sayıyor (`HOST_FLAG`), dolayısıyla Muiren'in manifest
   yazmasına ya da bir uzantı kimliği taklit etmesine gerek yok.
2. **Kimlik doğrulaması host sürecinde değil, tarayıcıda.** İzin listesi
   native messaging *manifestinde* (`allowed_origins` / `allowed_extensions`)
   duruyor ve onu tarayıcı uyguluyor. İkiliyi doğrudan çalıştıran bir çağıran
   o kapıdan hiç geçmiyor. **Yani Muiget tarafında hiçbir değişiklik ve
   hiçbir karar kaydı gerekmiyor.**
3. **Host durumsuz ve kısa ömürlü.** İsteği `muiget --add <base64>` olarak ana
   pencereye devredip çıkıyor. Bu yüzden Muiren kalıcı bir host süreci
   **tutmuyor**: tutmanın kazancı yok, bedeli boşta duran bir süreç. Yukarıdaki
   "boştaysa kapatılıyor" cümlesinin bugünkü karşılığı, her devirde kısa ömürlü
   bir süreç.

Protokol ayrıntısı: uzunluk öneki **yerel bayt sırasında** (`u32::from_ne_bytes`),
üst sınır 1 MB. Muiren okuma tarafında aynı sınırı uyguluyor — bozuk bir öneke
güvenip `Vec::with_capacity` çağırmak gigabaytlarca bellek ayırtabilirdi.

### Ne gönderiliyor, ne gönderilmiyor

`DownloadRequest` beş alan taşıyabiliyor. Muiren **üçünü** dolduruyor:

| Alan | Gönderiliyor mu | Gerekçe |
|---|---|---|
| `url` | evet | devrin kendisi |
| `fileName` | evet, **temizlenmiş** | sayfadan geliyor (CLAUDE.md #8) |
| `referrer` | evet | bir **URL**; genel kural URL'ye izin veriyor. Hotlink korumalı siteler (hedef kitlenin yarısı) `Referer` olmadan indirme vermiyor ve köprü işe yaramaz hâle geliyordu |
| `cookies` | **hayır** | genel kural: hiçbir köprü çerez taşımıyor |
| `userAgent` | **hayır** | aynı kural |

Bedeli açıkça yazalım: giriş gerektiren bir indirme Muiget tarafında başarısız
olabiliyor. Alternatifi, kullanıcının oturum çerezini süreçler arasında
taşımaktı; bu takas yapılmıyor.

---

## Muiply — Yerel Oynatma

**Ne zaman:** video/ses öğesine sağ tık, `file://` üzerinden bir medya
dosyasına gezinme, indirme tamamlandı bildirimi.

**Nasıl:** Muiply'ı dosya yolu argümanıyla çalıştırmak. Zaten çalışıyorsa
tek-örnek (single instance) mekanizmasıyla var olan pencereye devrediliyor.

**Neden değerli:** Muiren'in codec eksiği (özellikle HEVC, bkz. `docs/Medya.md`)
Muiply'ın libmpv'siyle kapanıyor. Tarayıcının açamadığı yerel dosyayı kardeş
uygulama açıyor.

**Sınır:** Muiply bilinçli olarak **ağ akışı oynatmıyor** (Muiply CLAUDE.md,
kapsam dışı kararlar). Yani uzak URL devri şu an anlamsız; köprü yalnız yerel
dosyalarda çalışıyor. Muiply bir gün uzak URL desteklerse burası genişletilir.

### Faz 3'te öğrenilen tek tuzak

`Muiply/src-tauri/src/acilis.rs` → `dosya_argumanlari`: `-` ile başlayan her
argüman **bayrak sayılıp atılıyor**. Yani `-film.mkv` adlı bir dosya hiç
açılmıyordu ve belirtisi "bazen hiçbir şey olmuyor" — bulunması en pahalı hata
türü. `bridge::muiply::arguman_guvenli` yolu `.\` ile önekliyor; saf ve testli.

---

## Muiwatch — Birlikte İzleme / Co-Browsing

**Bu köprü çift yönlü değerli.** Muiwatch'ın `docs/ROADMAP.md` dosyasında
**Faz 3 — Co-Browsing** başlanmamış durumda ve gömülü bir tarayıcı gerektiriyor.
Muiren tam olarak o gömülü tarayıcı.

Muiwatch'ın protokolü hazır (`Muiwatch/docs/PROTOCOL.md`):

| Mesaj | Muiren'in rolü |
|---|---|
| `nav_event` (`url_change`, `click`, `scroll`, `input`) | Kontrolcüyken üretir, takipçiyken uygular |
| `video_event` (`play`, `pause`, `seek`) | Aynı |
| `sync_heartbeat` | Kontrolcüyken pozisyon yayar |

Kurallar Muiwatch'tan devralınıyor, yeniden tartışılmıyor:

- **Yalnız `controllerId` sahibinden gelen `nav_event` uygulanır.** Takipçi
  konumdaki Muiren kendi input dinleyicisini pasif tutuyor.
- `play` uygulanırken RTT/2 ileri telafi ekleniyor (`senkron.ts` içindeki
  `ileriTelafi`); `pause`/`seek` telafisiz.

Muiren tarafındaki tek ek kural:

> **Muiwatch oturumuna bağlı sekme korumalıdır** — uyutulmaz, atılmaz.
> Uyuyan sekme `nav_event` alamaz ve senkron sessizce kopar; kullanıcı bunu
> "Muiwatch bozuk" diye okur. `docs/Bellek.md` koruma listesi #7.

### DURUM (Faz 5) — karşı uçta bugün alacak bir kapı yok

Muiwatch tarafına bakıldı:

- `Muiwatch/docs/ROADMAP.md` **Faz 3 — Co-Browsing başlanmamış.**
- Muiwatch'ta tek-örnek (single instance) ya da derin bağlantı eklentisi
  **yok**; yani "şu odaya bağlan" diyecek bir argüman kapısı bulunmuyor.
- `nav_event` / `video_event` / `sync_heartbeat` WebRTC DataChannel üzerinde,
  yani Muiwatch **süreci içinde** yaşıyor. Süreçler arası bir uç yok.

Bu yüzden Muiren'in bu fazdaki payı üç şeyle sınırlı ve **üçü de yazıldı**:

1. **Kurulum algılama** — Muiwatch yoksa köprü menüsü hiç çizilmiyor.
2. **Uygulamayı açma** — `muiwatch.exe --oda <kimlik>`. Argüman bugün karşı
   tarafta okunmuyor; okunduğunda Muiren tarafında değişiklik gerekmiyor.
3. **Koruma kuralı #7** — **asıl iş bu.** Bağlanan sekme uyutulmuyor ve
   atılmıyor. Bu kural protokolün hazır olup olmamasından bağımsız bir Muiren
   kuralı; bugün yazılabildi ve bugün test edildi (`memory/esik.rs`).

Oda kimliği serbest metin değil: harf, rakam, `-` ve `_`, en çok 64 karakter,
`-` ile başlayamıyor (`bridge::muiwatch::oda_gecerli`, saf + testli). Kimlik
komut satırına gidiyor ve `-` ile başlayan bir kimlik karşı tarafta bayrak
sanılırdı.

---

## Muifly — Oyun Modu

Muifly bir oyun performans aracı ve `muifly://durum` olayını yayınlıyor
(`Muifly/docs/ARCHITECTURE.md`). Oyun modu değiştiğinde Muiren'in haberi
olması, "oyuncular için tarayıcı" iddiasının somut karşılığı.

**Muifly → Muiren:** mod değiştiğinde Muiren bellek politikasını sertleştiriyor
(`docs/Bellek.md`, "Oyun modu" bölümü): eşikler ÷ 4, korumalılar dışında her
şey uyuyor, uyanık üst sınır 2, kabuk penceresi gizleniyor.

**Muifly kurulu değilse:** Muiren kendi başına tam ekran bir oyun süreci
algılamayı deniyor (öncelikli, ses çalan, tam ekran, odakta olmayan pencere).
Bu kabaca bir sezgi; yanlış pozitif ihtimali var, o yüzden **varsayılan
kapalı** ve ayarlarda "Oyun algılandığında sekmeleri uyut" olarak açık ediliyor.

**Muiren → Muifly:** bir şey göndermiyoruz. Tarayıcının oyun optimizasyon
aracına söyleyecek sözü yok.

### DOĞRULANDI (Faz 4) — köprünün karşı ucu yok, kendi algılamamız devrede

Bırakılan soru şuydu: *"`muifly://durum` bir Tauri olayı — yani süreç içi.
Süreçler arası dinlemek için Muifly tarafında bir dışa açılma noktası
gerekiyor."*

Bakıldı: `Muifly/src-tauri/src/commands.rs` içinde
`OLAY_DURUM = "muifly://durum"` ve olay `app.emit` ile yayınlanıyor. **Süreç
içi.** Named pipe, tek-örnek mesajı ya da izlenebilir bir durum dosyası yok.

Sonuç, sorunun kendi yazdığı sonuç: *"Karşılığı yoksa Faz 4 yalnız kendi
algılamamızla çıkıyor."* Öyle çıktı.

Bugün bu köprü iki iş yapıyor:

1. **Kurulum algılama** — Muifly kurulu değilse arayüz "Muifly ile" ifadesini
   kullanmıyor.
2. **Kendi algılamamız** (`bridge/oyun.rs`), varsayılan kapalı
   (`oyun_algilama` ayarı).

Sezginin kendisi dört koşul birlikte: ön plandaki pencere bir monitörün
tamamını kaplıyor **ve** bizim değil **ve** kabuğun (`explorer.exe`) değil
**ve** (yüksek öncelikli **ya da** kenarlıksız). Karar saf ve testli; testler
tam ekran tarayıcıyı, masaüstünü ve maksimize edilmiş normal pencereyi ayrı
ayrı sabitliyor.

Bilinen sınır: **pencereli kipte oynayan oyun görünmüyor.** Sezgi gevşetilmedi
— yanlış pozitifin bedeli (kullanıcının baktığı sekmelerin sebepsiz uyuması)
daha yüksek.

Algılama döngüsü gözcüden ayrı ve daha sık (3 sn): oyun açıldıktan 10 saniye
sonra sekmeleri uyutmak, tam da belleğin en gergin olduğu anı kaçırmak olurdu.

`muiren://oyun-modu` olayının `kaynak` alanı bu yüzden `elle` ya da
`algilama`; **`muifly` diye bir kaynak yok.**

---

## MuiLabs — Vitrin

MuiLabs `src/config/apps.config.ts` içinde ailenin listesini tutuyor. Muiren
ilk sürümde oraya bir kayıt olarak ekleniyor (`id: "muiren"`). Kod
entegrasyonu değil, tek satır kayıt — ama unutulursa uygulama vitrinde
görünmüyor.

---

## Köprü modülünün ortak şekli

Dört köprü de aynı imzayı taşıyor ki test edilebilsinler:

```rust
// bridge/mod.rs
pub trait Kopru {
    fn kurulu(&self) -> bool;                  // önbellekli
    fn ad(&self) -> &'static str;
    fn devret(&self, yuk: Yuk) -> Result<()>;  // yuk: Url | DosyaYolu | Mesaj
}
```

`kurulu()` yanlış dönüyorsa arayüzde ilgili menü öğesi hiç çizilmiyor.
Devretme hataları kullanıcıya **sessiz kalmadan** ama küçük bir bildirimle
gösteriliyor — köprü kırıldığında kullanıcı tarayıcıyı suçlamasın.
