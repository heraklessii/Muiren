# Temalar ve Arka Plan

Hedef kitle arayüzünü kişiselleştiren bir kitle. Ama tema sistemi aynı zamanda
projenin en büyük **güvenlik yüzeyi**: kullanıcı internetten indirdiği bir
dosyayı uygulamanın içine alıyor. Bu dosyanın yarısı bu yüzden güvenlik.

## Temel karar: tema veri, kod değil

> **Tema paketi JavaScript çalıştırmaz. Rastgele CSS de çalıştırmaz.**
> Tema = doğrulanmış jeton kümesi + doğrulanmış varlık dosyaları.

Sebep somut: bir tarayıcının arayüzünde çalışan kod, o tarayıcının açtığı her
sayfayı görebilir. "Şu forumdan indirdiğim güzel tema" bankacılık sekmesini
okuyabilir hâle gelirdi. Rastgele CSS bile yetiyor — `background-image` ile
uzak sunucuya sinyal göndermek, `content` ile sahte arayüz çizmek mümkün.

Bedeli: tema yazarları istedikleri her şeyi yapamıyor. Kabul ediyoruz.

## `.muitema` biçimi

ZIP arşivi, uzantısı `.muitema`:

```
tema.json          — üstveri + jetonlar (zorunlu)
arkaplan.jpg       — isteğe bağlı, tek dosya
arkaplan.mp4       — isteğe bağlı (Faz 5, aşağıdaki uyarıya bak)
onizleme.png       — mağaza/liste görseli (zorunlu, 640x400)
```

`tema.json`:

```json
{
  "bicim": 1,
  "ad": "Gece Sakura",
  "yazar": "kullanıcı adı",
  "surum": "1.0.0",
  "jetonlar": {
    "bg": "#12101a",
    "bg-panel": "#1b1826",
    "bg-elevated": "#241f33",
    "bg-sunken": "#0d0b13",
    "border": "#2e2840",
    "border-strong": "#3d3555",
    "text": "#ece8f2",
    "text-muted": "#9a92ab",
    "accent": "#f0a8c8",
    "accent-strong": "#f7c4dc",
    "on-accent": "#2a0f1c",
    "radius": "12px",
    "radius-lg": "16px"
  },
  "arkaplan": {
    "dosya": "arkaplan.jpg",
    "yerlesim": "kapla",
    "karartma": 0.55,
    "bulanik": 0
  }
}
```

## İzin verilen jetonlar — beyaz liste

Yalnız aşağıdaki anahtarlar kabul ediliyor. Listede olmayan anahtar **hata
değil, sessizce yok sayılıyor** (ileri sürüm temaları eski Muiren'de de açılsın
diye) ama uygulanmıyor.

| Jeton | Tip | Doğrulama |
|---|---|---|
| `bg`, `bg-panel`, `bg-elevated`, `bg-sunken` | renk | `#rgb`/`#rrggbb`/`#rrggbbaa` — **sadece bu** |
| `border`, `border-strong` | renk | aynı |
| `text`, `text-muted` | renk | aynı |
| `accent`, `accent-strong`, `on-accent` | renk | aynı |
| `radius`, `radius-lg` | ölçü | `0–32px` aralığında tam sayı + `px` |

Renk ayrıştırıcısı **düzenli ifadeyle** doğruluyor ve `url(`, `var(`,
`expression(`, `image-set(`, `\` ve boşluk içeren hiçbir değeri kabul etmiyor.
CSS enjeksiyonunun tamamı bu tek fonksiyonda kapanıyor:

```rust
// theme/jeton.rs
pub fn renk_dogrula(ham: &str) -> Option<Renk>   // #rrggbb(aa)? — başka hiçbir şey
pub fn olcu_dogrula(ham: &str) -> Option<Olcu>   // 0..=32 + "px"
```

Değiştirilemeyen şeyler (bilinçli): Mui logosu ve ikonu, hakkında ekranı,
izin/uyarı diyaloglarının rengi, adres çubuğundaki güvenlik göstergesi.
Uyarı diyaloğunu yeşile boyayabilen tema, uyarıyı yok edebilir demektir.

## Kontrast zorunluluğu

Tema yüklenirken `text` ile `bg`, `text-muted` ile `bg-panel` ve `on-accent`
ile `accent` arasındaki kontrast oranı hesaplanıyor. 4.5:1 altındaysa tema
**reddedilmiyor** ama kullanıcıya "bu tema metni okunmaz yapıyor" uyarısı
çıkıyor ve tek tıkla geri alınabiliyor.

`on-accent` özel bir tuzak: Mui teal'i (`#2dd4bf`) **açık** bir renk, üstündeki
yazı koyu olmak zorunda. Tema yazarı `accent`i koyulaştırıp `on-accent`i
güncellemezse bütün birincil düğmeler okunmaz hâle geliyor. Bu tam olarak
`..\MuiLabs\docs\ui-conventions.md` içinde not düşülmüş tuzak.

## Arka plan görseli

- **Yalnız yerel dosya.** `tema.json` içindeki `dosya` alanı arşivin içindeki
  bir ada işaret ediyor. `http(s)://`, `file://`, mutlak yol ve `..` reddediliyor
  (dizin taşması). Uzak URL yok — hem gizlilik (her yeni sekmede sunucuya sinyal)
  hem de "bulut yok" kuralı.
- **Boyut sınırı:** görsel ≤ 8 MB, çözünürlük ≤ 4096×4096. Aşan tema reddediliyor.
- **Biçim:** JPEG, PNG, WebP, AVIF. Ayrıştırma motorun kendi kod çözücüsüyle,
  kendi çözücümüzü yazmıyoruz.
- **Karartma** (`karartma`, 0–0.9) arka planla metin arasına düz bir katman
  koyuyor. Varsayılan 0.5 — çoğu fotoğrafın üstünde metin okunmuyor ve bunu
  tema yazarına bırakmak, kullanıcıyı okunmaz bir arayüzle baş başa bırakıyor.
- **Nerede görünüyor:** yeni sekme sayfası ve kabuğun boş alanları. Web
  sayfasının üstünde **hiçbir zaman** görünmüyor.

### Görsel kabuğa nasıl ulaşıyor (Faz 4'te yazıldı)

`convertFileSrc` ve Tauri'nin `asset` protokolü **kullanılmıyor.** O protokolü
açmak, kabukta çalışan her satırın kullanıcının diskinden okuyabilmesi demek
ve bunun karşılığı bir duvar kâğıdı değil.

Bunun yerine görsel backend'de okunup `data:` adresi olarak gidiyor
(`theme::arkaplan_veri`) — favicon deposuyla birebir aynı gerekçe
(`src-tauri/src/favicon/mod.rs`). Adres `TemaOzeti` içinde **değil**, ayrı bir
komutta (`tema_arkaplan`): özet her tema listesinde ve her `tema-degisti`
olayında gidiyor, görseli oraya koymak tema listesini açan kullanıcıya üç
duvar kâğıdını birden göndermek olurdu.

Çizim üç katman ve sırası önemli:

1. Görselin kendisi (`z-index: -2`), `filter: blur()` uygulanmış.
2. Karartma perdesi (`z-index: -1`) — **ayrı katman**, çünkü perde de
   bulanıklaşsaydı kenarlarda halka çıkardı.
3. Arayüz.

Bulanık görsel ayrıca taşırılıyor (`inset: -(bulanık + 8px)`): `blur` kenarda
saydamlık üretiyor ve taşırma olmadan çerçevede açık bir şerit kalıyor.

### Video arka plan (Faz 5, uyarıyla)

Kardeş proje Muivly (live wallpaper) bu işi zaten yapıyor. Muiren'de video arka
plan **bellek iddiasıyla doğrudan çelişiyor**: sürekli çalışan bir video kod
çözücü, tam da geri kazanmaya çalıştığımız kaynağı yiyor.

Karar: Faz 5'te değerlendirilecek ve eklenirse:
- Varsayılan **kapalı**,
- Pencere odakta değilken duruyor,
- Oyun modunda duruyor,
- Ayarlar ekranında ölçülmüş maliyeti yazıyor ("~%3 CPU, ~120 MB").

Rakam yazılmadan bu özellik eklenmiyor.

## Uygulama akışı

```
kullanıcı .muitema seçiyor
  → theme::paket::ac()        ZIP aç, boyut sınırlarını uygula
  → theme::paket::dogrula()   şema + her jeton + arka plan yolu + kontrast
     ├─ hata → tema uygulanmıyor, sebep kullanıcıya gösteriliyor
     └─ tamam ↓
  → tema uygulama veri klasörüne kopyalanıyor (orijinal dosyaya bağlı kalmıyor)
  → settings.tema = ad
  → olay: muiren://tema-degisti
Arayüz: :root üzerindeki CSS değişkenlerini güncelliyor
```

Arayüz tarafı sadece doğrulanmış jetonları `document.documentElement.style`
üzerine yazıyor. **Stil etiketi enjekte edilmiyor, `innerHTML` kullanılmıyor.**

> **Kural (CLAUDE.md #14):** `tema.json` içine yeni bir alan eklendiğinde
> `paket::dogrula` içine karşılığı ve testi eklenmeden birleştirilmiyor.

## Yerleşik temalar

Uygulamayla üç tema geliyor:

- **Mui** (varsayılan) — kardeş projelerin birebir jetonları, teal + `#0f1115`.
- **Mürekkep** — daha koyu, düşük kontrastlı, gece için.
- **Kağıt** — açık tema. Mui ailesinde açık tema yok; burada var çünkü tarayıcı
  gündüz de kullanılıyor ve sayfaların çoğu beyaz — koyu kabuk + beyaz sayfa
  göz yoruyor.

Yerleşik temalar da aynı jeton beyaz listesinden geçiyor. İstisna yok; istisna
olan yol, bir gün doğrulamayı atlayan yol oluyor.

## Tema paylaşımı

Kullanıcı kendi temasını `.muitema` olarak dışa aktarabiliyor. **Uygulama içi
tema mağazası yok** — sunucu yok, hesap yok, moderasyon yükü yok. Dosya
paylaşımı kullanıcının kendi işi. MuiLabs bir gün bir liste yayınlarsa oraya
bakılır.
