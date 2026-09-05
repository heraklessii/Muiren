# Mimari

## Ne inşa ediyoruz

Muiren bir **tarayıcı kabuğu**. Sayfayı çizen kod bizim değil (WebView2 =
Edge Chromium). Bizim yazdığımız kısım şu dört şey:

1. **Sekme yönetimi** — kaç webview var, hangisi görünür, hangisi bellekte.
2. **Bellek politikası** — projenin var olma sebebi. `docs/Bellek.md`.
3. **Kabuk** — adres çubuğu, geçmiş, yer imleri, indirme devri, tema.
4. **Köprüler** — Muiget / Muiply / Muiwatch / Muifly. `docs/Kopruler.md`.

Bunların hiçbiri "web standardı" işi değil. Bu bilinçli: HTML/CSS/JS uyumluluğu
20 yıllık bir borç ve onu ödemeye kalkmak projeyi ilk ayda bitirir.

## Katmanlar

```
┌──────────────────────────────────────────────┐
│ Arayüz (React)  —  sekme çubuğu, adres, panel │  saf görüntü
│   src/hooks   ←  olaylar                      │
│   src/ipc     →  komutlar                     │
└──────────────────────┬───────────────────────┘
                       │ Tauri IPC (docs/IPC.md)
┌──────────────────────┴───────────────────────┐
│ commands/  —  ince sarmalayıcılar             │  mantık YOK
├──────────────────────────────────────────────┤
│ tabs/     memory/    history/   theme/        │  karar veren kod
│ bridge/   settings/                           │
├──────────────────────────────────────────────┤
│ motor/  —  gercek.rs (WebView2) | yok.rs      │  dış dünya
└──────────────────────┬───────────────────────┘
                       │
              WebView2 (Edge Chromium)
```

Şemada olmayan tek modül `olcum/`: kabuk katmanlarının **yanında** değil,
kenarında duruyor. `MUIREN_OLCUM` tanımlıysa açılışta bir iş parçacığı
başlatıp sürücüyü dışarıdan sürüyor (sekme aç, bekle, gözcünün turunu
koştur, raporu diske yaz) ve komut yüzeyine hiç dokunmuyor. Değişken yoksa
tek satırı çalışmıyor (`docs/olcumler/README.md`, karar #8).

Ok yönü tek taraflı değil: `memory/gozcu.rs` kendi iş parçacığında koşuyor ve
arayüzden hiçbir tetik almadan `motor`a "bu sekmeyi uyut" diyor. Sonuç arayüze
bir **olay** olarak geri dönüyor. Bu yüzden karar veren kod arayüzde olamaz.

## Motor soyutlaması — neden `motor/`, neden `webview2/` değil

Muiply'da dizin `mpv/` çünkü orada motorun değişmesi gerçekçi bir ihtimal
değil. Burada tam tersi:

- WebView2 **Windows'a bağlı**. Linux/macOS bir gün gelecekse WebKitGTK ya da
  CEF gelecek.
- WebView2'nin süreç modeli üzerinde kontrolümüz sınırlı. Bellek iddiası
  büyürse CEF'e (tam Chromium gömme) geçmek gerekebilir; CEF süreç sayısını,
  render süreci ömrünü ve uzantı desteğini bize açar.

Bu yüzden `motor/` bir **arayüz**, WebView2 onun bir uygulaması:

```rust
// motor/mod.rs — her iki uygulamanın da tutmak zorunda olduğu yüzey
pub trait Motor<R: Runtime>: Send + Sync + 'static {
    fn yetenekler(&self) -> Yetenekler;
    fn dinleyici_ata(&self, dinleyici: Weak<dyn Dinleyici>);

    fn sekme_ac(&self, id: SekmeId, url: &str) -> Sonuc<()>;
    fn sekme_kapat(&self, id: SekmeId) -> Sonuc<()>;
    fn gezin(&self, id: SekmeId, url: &str) -> Sonuc<()>;
    fn geri(&self, id: SekmeId) -> Sonuc<()>;
    fn ileri(&self, id: SekmeId) -> Sonuc<()>;
    fn yenile(&self, id: SekmeId) -> Sonuc<()>;
    fn durdur(&self, id: SekmeId) -> Sonuc<()>;

    fn alan_ayarla(&self, dikdortgen: Dikdortgen) -> Sonuc<()>;
    fn goster(&self, id: SekmeId, dikdortgen: Dikdortgen) -> Sonuc<()>;
    fn gizle(&self, id: SekmeId) -> Sonuc<()>;
    fn ses_kes(&self, id: SekmeId, sessiz: bool) -> Sonuc<()>;

    // Bellek politikasının motordan istediği tek şey bunlar:
    fn uyut(&self, id: SekmeId) -> Sonuc<()>;      // TrySuspend
    fn uyandir(&self, id: SekmeId) -> Sonuc<()>;   // Resume
    fn bellek_hedefi(&self, id: SekmeId, seviye: BellekSeviyesi) -> Sonuc<()>;
    fn at(&self, id: SekmeId) -> Sonuc<()>;        // webview'i tamamen yık

    fn var_mi(&self, id: SekmeId) -> bool;
}
```

Yüzeyin bir `trait` olması bilinçli: CLAUDE.md #4 ("`gercek.rs` içine eklenen
her metot `yok.rs` içinde de olmalı") kuralını **derleyici** zorluyor. Muiply'da
aynı kural yorumla tutuluyor ve bir kez unutulduğunda `--no-default-features`
sessizce kırıldı.

İki ek var, ikisi de aynı sebepten:

- **`dinleyici_ata`** — motor `tabs`ı tanımıyor. Başlık değiştiğinde "sekme
  kaydını güncelle" demiyor, "başlık değişti" diyor; kaydı güncelleyen ve
  olayı yayınlayan `tabs/surucu.rs`. Bağ **zayıf** referansla kuruluyor;
  güçlü olsaydı sürücü → motor → sürücü döngüsü kapanışta ikisini de ayakta
  tutardı.
- **`alan_ayarla`** — sekme webview'i ayrı bir native pencere ve konumunu
  arayüz bildiriyor (`icerik_alani` komutu, `docs/IPC.md`).

`motor/yok.rs` bu yüzeyi hiçbir şey yapmadan uygular. İki işe yarıyor:

- `cargo test --no-default-features` Windows'suz makinede ve CI'da koşuyor.
- `tabs/`, `memory/`, `history/` modüllerinin motora bağımlı olmadığını
  derleyici zorluyor. Bir gün biri `memory/esik.rs` içine `unsafe` bir Win32
  çağrısı yazmaya kalkarsa motorsuz derleme kırılıyor.

> **Kural:** `gercek.rs` içine eklenen her metodun `yok.rs` içinde de karşılığı
> olacak. Muiply'da aynı kural var ve orada bir kez unutulduğunda
> `--no-default-features` sessizce kırıldı.

## Süreç modeli (dürüst kısım)

WebView2 kullanıldığında süreç tablosu şöyle:

| Süreç | Kim yönetiyor | Kaç tane |
|---|---|---|
| Muiren.exe | biz | 1 |
| `msedgewebview2.exe` (browser) | WebView2 | kullanıcı veri klasörü başına 1 |
| `msedgewebview2.exe` (renderer) | WebView2 | site örneği başına ~1 |
| GPU / audio / network yardımcıları | WebView2 | birkaç tane |

Aynı `CoreWebView2Environment`i (aynı kullanıcı veri klasörü) paylaşan bütün
webview'ler **tek browser sürecini** paylaşıyor. Yani "her sekme için ayrı
Chromium" gibi bir felaket yok; mimarı Edge'inkiyle aynı.

Buradan çıkan üç sonuç doğrudan projeyi tanımlıyor:

1. **Tek bir ağır sekmede Chrome'u yenemeyiz.** Motor aynı motor. YouTube'un
   JS yığını Chrome'da ne kadarsa Muiren'de de o kadar. Bunu vaat etmiyoruz.
2. **Çok sayıda boşta sekmede yenebiliriz.** Kazanç orada, çünkü kazanç
   *politikadan* geliyor, motordan değil. Ayrıntı: `docs/Bellek.md`.
3. **Renderer satırı bir politika sorusu, sabit değil.** Tablodaki "site
   örneği başına ~1" Chromium'un varsayılanı; bayrakla değiştirilebiliyor.
   Bu, projenin ikinci kazanç kaynağı.

### Renderer sayısı — değiştirilebilir olan tek satır

Render süreci başına taban maliyet 30–60 MB, sayfa içeriğinden bağımsız. Aynı
siteden 10 sekme açan bir kullanıcı varsayılan modelde 10 süreç, dolayısıyla
yarım GB'a yakın taban maliyet üretiyor.

`motor/gercek.rs` bunu `EK_BAYRAKLAR` ile daraltıyor (`docs/Setup.md`):

| Bayrak | Etkisi |
|---|---|
| `--process-per-site` | Aynı sitenin sekmeleri tek süreçte birleşir |
| `--renderer-process-limit=N` | Toplam renderer sayısına tavan |

Site izolasyonu **kapatılmıyor** — gerekçe `docs/Bellek.md` içinde.

Firefox'un çok sekmede az yemesinin sebebi de tam olarak bu satır; motor
kalitesi değil, süreç politikası. Motor alternatiflerinin tamamı
`docs/Roadmap.md` karar #6'da değerlendirildi ve elendi.

> Bu bayrakların WebView2'den geçtiği **henüz doğrulanmadı** — Faz 0/R4.
> Geçmezlerse CEF gerekçesi güçleniyor (R2 ile aynı karar).

## Sekme = webview, ama her sekme webview değil

Kritik ayrım ve mimarinin en önemli cümlesi:

> **Sekme bir veri kaydıdır. Webview ise o kaydın pahalı, isteğe bağlı,
> her an yok edilebilir bir eki.**

`tabs/` modülü sekmeleri tutar: id, URL, başlık, favicon, gezinme geçmişi,
kaydırma konumu, son etkinlik zamanı. Bunların hepsi birkaç yüz bayt.

`motor/` ise webview'leri tutar. Bir sekmenin webview'i olabilir de olmayabilir
de. Atılmış (`Atilmis`) bir sekmenin sekme çubuğunda yeri, başlığı ve favicon'u
vardır ama tek bayt bile render belleği tüketmez; tıklandığında kaydedilmiş
URL ve kaydırma konumuyla yeniden doğar.

Bu ayrım olmadan "500 sekme" mümkün değil. Bu ayrım varken sekme sayısının
üst sınırı RAM değil, sekme çubuğunun çizimi oluyor.

## Veri akışı — bir örnek

Kullanıcı bir sekmeye tıklıyor (sekme `Atilmis` durumunda):

```
Arayüz: sekme_etkinlestir(id)
  → commands/tabs.rs          (sarmalayıcı, karar yok)
    → tabs::durum::gecis(Atilmis, Etkinlestirildi)  → Etkin
      → motor.sekme_ac(id, kayitli_url)
      → motor.goster(id, dikdortgen)
    → tabs::agac::etkin_degistir(id)                (eski etkin → Arkaplan)
      → memory::gozcu bir sonraki turda eski etkini değerlendirir
  ← olay: muiren://sekme-degisti
Arayüz: sekme çubuğunu yeniden çiziyor
```

Dikkat: eski sekmeyi uyutma kararı bu akışta **yok**. Onu gözcü veriyor, çünkü
"kullanıcı 3 saniyeliğine başka sekmeye baktı" ile "kullanıcı o sekmeyi 40
dakikadır açmadı" farklı şeyler ve bu farkı bilen tek yer gözcünün zamanlayıcısı.

## Veri akışı — ters yön (gözcü)

```
memory/gozcu.rs (kendi iş parçacığı, varsayılan 10 sn'de bir)
  → memory::olcum::sistem()            (Win32)
  → memory::olcum::surecler()          (Win32: PID → MB)
  → motor.surec_haritasi()             (WebView2: PID → sekmeler)
  → memory::esleme::Defter::tur(...)   (ikisini birleştirir)          ← SAF
  → memory::esik::karar(durumlar, baski, ayarlar) -> Vec<Eylem>   ← SAF
      Eylem::Uyut(id) | Eylem::At(id) | Eylem::HedefDusur(id)
  → motor.uyut(id) / motor.at(id)
  → tabs::durum::gecis(...)
  ← olay: muiren://sekme-durum-degisti
```

`memory::esik::karar` fonksiyonu saf: girdisi bir durum listesi, çıktısı bir
eylem listesi. Sistem çağrısı yok, zaman okuma yok (zaman parametre olarak
geliyor). Bu yüzden testlerle "40 dakikadır boşta, sabitlenmiş, ses çalıyor"
gibi onlarca kombinasyonu doğrulamak mümkün.

## Neden Tauri, neden Electron değil

Electron ile hobi tarayıcı yazmak daha kolay (`WebContentsView` tam olarak
sekme primitifi). Ama Electron'da kabuk da bir Chromium örneği. Yani daha
tarayıcı hiçbir sayfa açmadan ikinci bir motor ayakta.

Tauri'de kabuk da WebView2 üstünde koşuyor ama **aynı** WebView2 ortamını
paylaşıyor ve boş bir Tauri penceresi Electron'unkinin çok altında. Bellek
iddiası olan bir projede kabuğun kendi maliyeti pazarlık konusu değil.

Bedeli dürüstçe: Tauri v2'de bir pencerede çoklu webview `unstable` özelliği
arkasında. Sürüm yükseltirken bu API'nin değişebileceğini kabul ediyoruz ve
bu yüzden `motor/` soyutlaması var — API değişirse tek dosya değişiyor.

## Doğrulanacaklar (varsayım değil, iş kalemi)

Bu belge şu üç noktayı **doğrulanmamış** kabul ediyor; Faz 0'ın çıktısı bunları
kanıtlamak (bkz. `docs/Roadmap.md`):

1. `webview2-com` ile ham `ICoreWebView2` alınıp `TrySuspend` / `Resume`
   çağrılabiliyor mu, Tauri'nin sardığı webview üzerinden erişim var mı.
2. Bellek hedefi seviyesi (`SetMemoryUsageTargetLevel` benzeri) hangi WebView2
   sürümünden itibaren mevcut ve ölçülebilir bir fark yaratıyor mu.
3. Widevine/PlayReady gerektiren siteler (Crunchyroll, Netflix) WebView2'de
   açılıyor mu — açılmıyorsa hedef kitlenin yarısı etkilenir. `docs/Medya.md`.
4. Süreç bayrakları (`--process-per-site`, `--renderer-process-limit`)
   WebView2'nin `AdditionalBrowserArguments` kanalından geçiyor mu.

Dördü de "muhtemelen çalışır"la geçilemez. Faz 0 bittiğinde bu bölüm ya silinir
ya da mimari değişir.
