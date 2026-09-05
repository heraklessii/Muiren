/**
 * Sayı ve süre biçimleme — saf ve testli.
 *
 * Tek yerde durması önemli: bellek paneli, sekme ipucu ve ayarlar ekranı aynı
 * rakamı aynı biçimde göstermeli, yoksa kullanıcı iki farklı sayı gördüğünü
 * sanıyor.
 */

/**
 * Megabaytı okunur hâle getirir.
 *
 * `yaklasik` true ise başına `~` geliyor. **Yaklaşık değeri kesin gibi
 * göstermek yok** (`docs/Bellek.md`): süreç → sekme eşlemesi güvenilmezse
 * kullanıcı bunu görecek.
 */
export function bellek(mb: number, yaklasik = false): string {
  const on = yaklasik ? "~" : "";
  if (!Number.isFinite(mb) || mb < 0) return `${on}0 MB`;
  if (mb < 1024) return `${on}${Math.round(mb)} MB`;
  const gb = mb / 1024;
  // 10 GB'ın altında bir ondalık: 3.2 GB ile 3 GB arasındaki fark anlamlı.
  return `${on}${gb < 10 ? gb.toFixed(1) : Math.round(gb)} GB`;
}

/**
 * Boşta geçen süre — sekme ipucunda "42 dk önce" olarak görünüyor.
 *
 * Türkçe ek üretmiyor (CLAUDE.md #11): "3 sa" gibi eksiz kalıplar her sayıyla
 * çalışıyor.
 */
export function sure(saniye: number): string {
  if (!Number.isFinite(saniye) || saniye < 0) return "0 sn";
  if (saniye < 60) return `${Math.floor(saniye)} sn`;
  const dakika = Math.floor(saniye / 60);
  if (dakika < 60) return `${dakika} dk`;
  const saat = Math.floor(dakika / 60);
  if (saat < 24) return `${saat} sa`;
  return `${Math.floor(saat / 24)} gün`;
}

/**
 * Sekme sayısını duruma göre özetler: "14 uyanık · 22 uyuyan · 9 atılmış".
 *
 * Sıfır olan kalem hiç yazılmıyor — "0 atılmış" bilgi değil, gürültü.
 */
export function durumOzeti(uyanik: number, uyuyan: number, atilmis: number): string {
  return [
    uyanik > 0 ? `${uyanik} uyanık` : "",
    uyuyan > 0 ? `${uyuyan} uyuyan` : "",
    atilmis > 0 ? `${atilmis} atılmış` : "",
  ]
    .filter((p) => p !== "")
    .join(" · ");
}

/**
 * Ham bayt sayısı — indirme önerisinde görünüyor.
 *
 * [`bellek`] ile ayrı: orası zaten MB alıyor ve bellek paneline ait. Burası
 * sunucunun bildirdiği `Content-Length` gibi ham bir sayı; MB'a çevirip
 * `bellek`e vermek 900 KB'lık bir dosyayı "1 MB" yapardı.
 *
 * 1000'lik değil **1024'lük** basamak: dosya boyutu bağlamında Windows da
 * böyle gösteriyor ve kullanıcı iki farklı sayı görmemeli.
 */
export function bayt(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return "0 B";
  const birimler = ["B", "KB", "MB", "GB", "TB"];
  let deger = n;
  let i = 0;
  while (deger >= 1024 && i < birimler.length - 1) {
    deger /= 1024;
    i += 1;
  }
  // Baytta ondalık anlamsız; üstünde 10'un altındaysa bir hane.
  const yazi = i === 0 ? String(Math.round(deger)) : deger < 10 ? deger.toFixed(1) : String(Math.round(deger));
  return `${yazi} ${birimler[i]}`;
}
