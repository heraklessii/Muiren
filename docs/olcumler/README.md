# Ölçümler

`docs/Roadmap.md` üç yerde aynı şeyi söylüyor: **ölçüm buraya yazılmadan faz
kapanmıyor.** Bu dizin o yazının yeri.

Kural tek cümle: *"hızlı olduğunu hissediyorum" kabul edilmiyor.* Bir kabul
kriterinin karşısına ya bir sayı ya "ölçülmedi" yazılıyor; üçüncü seçenek yok.

## Dosya adı

`YYYY-AA-GG-konu.md` — ör. `2026-09-05-faz2-50-sekme.md`. Tarih başta, çünkü
aynı ölçüm makine değiştikçe ve kod değiştikçe tekrarlanacak ve eskisi
silinmeyecek: **eski ölçüm yanlış değil, o günün cevabı.** Silinirse bir
gerilemenin ne zaman girdiği bir daha bulunamaz.

## Her raporun başında olması gerekenler

Bunlar olmadan sayı taşınabilir değil; başka bir makinede tekrarlanamayan
ölçüm ölçüm değil.

```markdown
| Alan | Değer |
|---|---|
| Tarih | 2026-09-05 |
| Makine | Ryzen 7 5800X · 32 GB · NVMe · Windows 11 26200 |
| WebView2 Runtime | 141.0.3537.71 |
| Muiren | commit `abc1234` · `cargo build --release` |
| Bellek profili | Dengeli (varsayılan) |
| Süreç politikası | `process-per-site` açık |
| Ağ | 100 Mbit, kablolu |
| Karşılaştırılan | Chrome 141.0.7390.55, temiz profil, uzantısız |
```

**Sürüm alanları boş bırakılmıyor.** WebView2 Runtime kendi kendine
güncelleniyor; iki ölçüm arasındaki farkın sebebi bizim kodumuz değil, o
güncelleme olabilir.

## Ölçüm modu — protokolün otomatik yarısı

Aşağıdaki adımların dördü (oturum, 1, 2, 3, 4) **koşturulabiliyor**. Muiren
`MUIREN_OLCUM` ortam değişkeni tanımlıyken açılırsa planı okuyor, sekmeleri
açıyor, protokolün istediği anlarda ölçüyor ve raporu bu dizine yazıyor.

```bash
MUIREN_OLCUM=docs/olcumler/plan-ornek.json npm run tauri dev
```

Windows kabuğunda (PowerShell):

```powershell
$env:MUIREN_OLCUM = "docs/olcumler/plan-ornek.json"; npm run tauri dev
```

Plan bir JSON dosyası; eksik anahtarlar protokolün varsayılanına düşüyor:

```json
{
  "ad": "faz2-50-sekme",
  "adresler": ["https://...", "..."],
  "acilisAraligiMs": 1500,
  "yerlesmeSn": 60,
  "bostaSn": 900,
  "cikti": ""
}
```

`cikti` boşsa rapor **planın yanına** `YYYY-AA-GG-ad.md` olarak yazılıyor.

`plan-ornek.json` bu dizinde duruyor ve **kabul ölçümü değil**: beş adres, kısa
süreler — modun çalıştığını görmek için. Faz 2'nin gerçek planı 50 adresle bir
kez yazılıp bu dizinde saklanıyor, çünkü farklı sitelerle alınan iki sayı
karşılaştırılamaz.

**Modun sınırları, dürüstçe:**

- **Chrome ölçülmüyor.** Kabul tablosunun üç satırı karşılaştırmalı; o sütun
  elle doluyor ve rapor bunu kendi içinde yazıyor. Otomatik "geçti" yalnız
  mutlak hedefi olan satırda çıkıyor (boş tarayıcı < 150 MB).
- **Uyanma gecikmeleri ölçülmüyor** (5–6. maddeler). Mod hiçbir sekmeye
  tıklamıyor, dolayısıyla bir uyanma da üretmiyor. O iki satır elle koşuluyor.
- **Sekme kapatmıyor.** Koşu başlarken açık sekme varsa "boş tarayıcı" satırı
  taban maliyet değil ve bu rapora **not olarak** düşüyor. Kullanıcının açık
  sekmelerini bir ölçüm aracının kapatması kabul edilemez.
- **Gözcü kapatılmıyor** — aşağıdaki kuralın gereği: ölçtüğümüz şey
  politikanın çalıştığı hâl.

Mod normal kullanımda **yok**: değişken tanımlı değilse tek satırı çalışmıyor,
arayüzde düğmesi yok, IPC yüzeyine komut eklemiyor.

## Faz 2 kabul tablosu — nasıl ölçülüyor

`docs/Bellek.md` "Başarı kriterleri" tablosundaki altı satırın her birinin
yöntemi. Sıra önemli: aynı oturumda yukarıdan aşağı koşuluyor.

### Oturum

50 sekme, beş kategoriden onar (10 sosyal medya, 10 haber, 10 forum, 10
anime/video, 10 dokümantasyon). **Aynı liste her ölçümde kullanılıyor** ve
raporun sonuna ekleniyor; farklı sitelerle alınan iki sayı karşılaştırılamaz.

Sekmeler açılırken bellek gözcüsü çalışmaya devam ediyor — kapatılmıyor.
Ölçtüğümüz şey politikanın çalıştığı hâl.

### 1. Boş tarayıcı (< 150 MB)

Sıfır sekme, yeni sekme sayfası açık, 60 saniye bekle. Bellek panelindeki
**Sayfalar + Kabuk**. Yeni sekme sayfasının webview'i yok (kabuk çiziyor),
yani bu gerçekten taban maliyet.

### 2. 50 sekme açıldıktan hemen sonra (Chrome'un altında)

Son sekme yüklendikten 60 saniye sonra oku. Chrome tarafında karşılığı: Görev
Yöneticisi → Chrome süreçlerinin toplamı, ya da `chrome://system`.

### 3. Render süreç sayısı (Chrome'un yarısından az)

Bellek panelindeki **"WebView2 süreci"** satırı. Chrome tarafı: Görev
Yöneticisi'nde Chrome süreçlerinin sayısı.

> Bu satır aynı zamanda **Faz 0/R4'ün cevabı**. Ayrıca doğrudan bir deneme
> var: aynı siteden 10 sekme aç, sayıya bak. ~1 render süreci görüyorsan
> `--process-per-site` geçmiş; 10 kalıyorsa geçmemiş ve `docs/Bellek.md`
> içindeki 2. kazanç kaynağı bu motorda yok demektir.

### 4. 50 sekme, 15 dk boşta (Chrome'un yarısından az)

15 dakika hiç dokunma. Uyutmanın işini yaptığı yer burası; sayı gözcünün
uyuttuğu sekme sayısıyla birlikte yazılıyor ("38 uyuyan, 4 atılmış").

### 5–6. Uyanma gecikmeleri (< 300 ms · < 2 sn)

**Bunlar elle kronometreyle ölçülmüyor**; Muiren kendi ölçüyor
(`src-tauri/src/memory/gecikme.rs`). Yöntem:

1. Bellek panelini aç, **"Ölçümü sıfırla"**ya bas. Sabahki gezinmelerin
   dağılıma karışmaması için: rapor edilen dağılım, o oturumda ölçülenlerin
   dağılımı olmalı.
2. Uyuyan ve atılmış sekmelere teker teker tıkla — en az 20 uyanma. Az örnekle
   alınan p95 bir sayı değil, bir tesadüf.
3. Panelde çıkan medyanı yaz; ipucunda duran `n`, p95 ve en kötü değeri de
   rapora geçir. **Yalnız medyan yazmak yok**: kullanıcının "bazen takılıyor"
   dediği an tam olarak kuyrukta.

İki satır **farklı şeyler ölçüyor** ve rapor bunu tekrar etmek zorunda:

| Satır | Kronometre başlıyor | Duruyor | Ne demek |
|---|---|---|---|
| Uykudan dönüş | `Resume` çağrısından önce | sayfanın JS iş parçacığı cevap verince | gerçekten "etkileşime hazır" |
| Atılmıştan dönüş | webview yaratılmadan önce | `NavigationCompleted` | ilk boya **değil**, onun üst sınırı |

İkincisi kasıtlı olarak temkinli: WebView2 bize bir ilk boya olayı vermiyor ve
olmayan bir olayı varmış gibi göstermektense dürüst bir üst sınır veriliyor. Bu
sayı eşiği geçiyorsa kriter kesin olarak sağlanmış oluyor; geçmiyorsa "ilk boya
da geçmedi" **denemiyor**, ayrıca not düşülüyor. Ağ süresi dahil
(`docs/Bellek.md` bunu şart koşuyor).

Sayılmayanlar, sessizce değil kuralla: başarısız gezinme (CLAUDE.md #7),
30 saniyeyi aşan uyanma (tek bir ağ kopması medyanı kalıcı zehirlerdi) ve
uyanmayı tamamlamadan geri uyutulan sekme.

## Faz 0 — açık riskler

| Risk | Nasıl ölçülüyor |
|---|---|
| R1 — `TrySuspend` kazandırıyor mu | 20 sekme aç, "Hepsini uyut", öncesi/sonrası **Sayfalar** satırı. `Yetenekler.askiyaAlma` false ise ölçüm anlamsız, önce o yazılır |
| R3 — 30+ webview taşınıyor mu | 30, 40, 50 sekmede sekme değiştirme gecikmesi + çökme var mı. Sızıntı: hepsini kapat, 5 dk bekle, **WebView2 süreci** sıfıra dönüyor mu |
| R4 — bayraklar geçiyor mu | yukarıda madde 3 |

R2 (DRM) bu protokolün dışında: cevabı bir sayı değil, "bölüm baştan sona
oynadı mı" (`docs/Medya.md`).

## Rapor iskeleti

```markdown
# 2026-09-05 · Faz 2 kabul ölçümü

[üstteki ortam tablosu]

## Sonuç

| Ölçüt | Hedef | Muiren | Chrome | Geçti mi |
|---|---|---|---|---|
| Boş tarayıcı | < 150 MB | | — | |
| 50 sekme, hemen sonra | Chrome'un altında | | | |
| 50 sekmede render süreci | Chrome'un yarısı | | | |
| 50 sekme, 15 dk boşta | Chrome'un yarısı | | | |
| Uykudan dönüş (medyan) | < 300 ms | | — | |
| Atılmıştan dönüş (medyan) | < 2 sn | | — | |

## Ham veri

[panelin gösterdiği sayılar, n / p95 / en kötü, uyuyan+atılmış dağılımı]

## Notlar

[beklenmeyen her şey: bir sitenin uyumaması, bir sekmenin çökmesi,
ölçüm sırasında koşan başka program]

## Sekme listesi

[50 URL]
```

**"Notlar" boş bırakılmıyor.** Sorunsuz geçen bir ölçümde bile "sorunsuz" yazan
bir satır, o günün gerçekten sorunsuz olduğunu söylüyor; boşluk ise not almayı
unutmuş olabileceğini.
