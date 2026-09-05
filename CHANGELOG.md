# Değişiklikler

Sürümler [semantik sürümlemeyi](https://semver.org/lang/tr/) izliyor. Proje
0.x'te olduğu sürece küçük sürüm (`0.X.0`) kırıcı değişiklik taşıyabilir.

Bir sürümde **ne yapılmadığı** da yazılıyor: kapanmayan bir kabul kriteri,
tamamlanmış bir özellik listesinin arkasında kaybolmamalı.

---

## 0.2.0 — 2026-09-05

Arayüz yenilemesi ve yedi hata.

### Eklendi

- **Durum halkası.** Sekmenin bellek durumu artık favicon'un çevresindeki
  halkada: etkin dolu, uyuyan sönük, atılmış **kesikli**. Sebebi yer — alt
  sınıra inmiş bir sekmede favicon dışında hiçbir şey görünmüyor ve durum,
  tam da en çok sekme açıkken kayboluyordu. Aynı bileşen yatay şeritte, dikey
  listede ve sekme aramasında.
- **Bellek nabzı** — gezinme çubuğunda üç dilimli çubuk ve tek rakam.
  Projenin tek iddiası `Ctrl+Shift+M` ile açılan bir panelin arkasındaydı;
  kullanıcı politikanın çalıştığını hiç görmeden kullanabiliyordu.
- **Yeni sekme sayfası**: sık gidilenler, yer imleri ve bellek kartı. Hepsi
  yerel diskten, **ağ isteği yok**. Gizli sekmede geçmiş listeleri hiç
  çizilmiyor.
- Sekme listelerinde **dolaşan odak** (ok tuşları, `Home`/`End`, `Delete`).
  Ok tuşları sekmeyi etkinleştirmiyor: klavyeyle şeridi taramak, uyuyan
  sekmeleri uyandırmamalı.
- `prefers-reduced-motion` desteği.
- `src/dev/sahte.ts` — geliştirme koşumu. `npm run dev` sonrası
  `localhost:1420/?sahte` ile kabuk düz bir tarayıcıda gerçekçi veriyle
  açılıyor. Üretim paketine girmiyor.

### Düzeltildi

- Adres çubuğu taslağı sekme değişince kalıyordu; Enter yeni sekmeyi eski
  sekmenin adresine götürüyordu.
- `Escape` açık örtüyü kapatmıyor, backend'e gidip sayfayı durduruyordu.
  Ayarlar ekranındayken tek çıkış kapatma düğmesiydi.
- Yeni sekme sayfasında "bağlantı şifresiz" uyarısı çıkıyordu; ortada
  bağlantı yok.
- Şerit etkin sekmeye kaydırmıyordu: `Ctrl+T` ile açılan sekme ekran dışında
  kalabiliyordu.
- 50 sekmede her sekme bir `Tab` durağıydı; klavyeyle adres çubuğuna geçmek
  pratikte imkânsızdı.
- Sekme yoğunluğu pencere genişliğine bakıyordu, sekmenin kendi genişliğine
  değil: geniş pencerede 40 sekme açıldığında kapatma düğmesi çizilmeye
  devam ediyordu.
- Grubun renk şeridi, sürükleme hedef çizgisini yutuyordu.

### Değişti

- Etkin sekmenin genişlik alt sınırı ayrıldı (152 px): 14 sekmede bile
  kullanıcının şu an baktığı sayfanın adı görünmüyordu.
- Bellek gözcüsünün ilk turu öne alındı (bir periyot → 3 sn). Döngü önce
  uyuyup sonra ölçtüğü için nabzın yeri açılıştan sonra boş kalıyordu.
- Adres/arama ayrımı backend'e taşındı (`tabs::adres_mi`, saf ve testli).
  Arayüzdeki kopyanın işi daraldı: yalnız kilit simgesi ve alan adı vurgusu.
  `javascript:` ve `data:` adres sayılmıyor, aramaya düşüyor.
- Webview yaratabilen komutlar `async`. Eşzamanlı bir komut, kabuk
  webview'inin geri çağrısı içinde `Window::add_child` çağırdığında
  uygulama kilitleniyordu.
- Türetilmiş yüzey renkleri `color-mix` ile temanın kendi jetonlarından
  çıkıyor; kullanıcı teması bozulmuyor.

### Hâlâ ölçülmedi

`docs/Bellek.md` içindeki kabul tablosu. Ölçüm aracı hazır, rakam yok —
**projenin asıl iddiası bu sürümde de doğrulanmış değil.** Durum tablosu:
`docs/Roadmap.md`.

---

## 0.1.0 — 2026-09-04

İlk yayın. Sekmeler, bellek politikası, geçmiş ve yer imleri, temalar,
engelleme, kardeş uygulama köprüleri ve ölçüm modu yazıldı; otomatik
testlerle doğrulanıyor.

Kabul kriterlerinin hiçbiri kapanmadı: ne 20 sekmelik günlük kullanım
denemesi, ne `docs/Bellek.md` ölçüm tablosu.
