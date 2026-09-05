/**
 * Adres çubuğunun iki sorusu — saf ve testli (`docs/Frontend.md`).
 *
 * 1. Kullanıcının yazdığı şey URL mi arama mı?
 * 2. Gösterilecek biçim ne?
 *
 * İkinci soru bir **güvenlik özelliği**. `https://banka.com.saldirgan.net/`
 * adresinde kullanıcının okuduğu ilk şey "banka.com" oluyor; vurgulanması
 * gereken ise `saldirgan.net`. Aynı şekilde `аpple.com` (Kiril "а") ile
 * `apple.com` ekranda ayırt edilemiyor — o yüzden Latin dışı harf taşıyan
 * alan adları punycode gösteriliyor.
 *
 * Ayrım burada, arayüzde, çünkü kullanıcı yazarken canlı geri bildirim
 * gerekiyor. Backend gelen dizeyi yine de doğruluyor (`docs/IPC.md`).
 */

export type Cozum =
  | { tur: "adres"; url: string }
  | { tur: "arama"; sorgu: string };

/** Kabuğun kendi sayfaları. */
export const YENI_SEKME = "muiren://yeni";

/** Gezinilebilir şemalar. Listede olmayan her şey arama sayılıyor. */
const IZINLI_SEMALAR = new Set(["http:", "https:", "file:", "muiren:"]);

/**
 * Birden çok etiketli genel son ekler.
 *
 * Tam bir Public Suffix List değil ve olamaz: liste binlerce satır ve haftada
 * bir değişiyor, arayüz paketine gömülecek bir şey değil. Buradaki kısa liste
 * Türkçe konuşan kullanıcının günlük olarak gördüklerini kapsıyor. Listede
 * olmayan bir çok parçalı son ekte vurgu bir etiket sağa kayıyor
 * (`ornek.co.jp` yerine `co.jp`) — yanlış yönde değil: fazladan vurgulanan
 * etiket kullanıcıyı yanıltmıyor, eksik vurgulanan yanıltırdı.
 */
const COK_PARCALI_SON_EKLER = [
  "com.tr", "net.tr", "org.tr", "gov.tr", "edu.tr", "av.tr", "bel.tr", "k12.tr",
  "co.uk", "org.uk", "ac.uk", "gov.uk",
  "com.au", "com.br", "com.cn", "com.mx", "co.jp", "co.kr", "co.in", "co.nz",
  "com.ar", "com.tw", "com.hk", "com.sg", "com.ua",
];

/** IPv4 düz yazımı. */
const IPV4 = /^\d{1,3}(\.\d{1,3}){3}$/;

/** Şema öneki: `https:`, `muiren:`, `javascript:` … */
const SEMA = /^([a-zA-Z][a-zA-Z0-9+.\-]*):/;

/**
 * `konak:port` yazımı — `localhost:3000`, `192.168.1.5:8080`.
 *
 * [`SEMA`] bunları da yakalıyor (`localhost:` geçerli bir şema deseni) ve
 * ayırt edilmezse geliştiricinin en çok yazdığı adres aramaya gidiyor.
 * Ayıran şey iki nokta üst üsteden sonra **yalnız rakam** olması.
 */
const KONAK_PORT = /^[a-zA-Z0-9.\-]+:\d+(?:[/?#]|$)/;

/** Girdi gerçekten bir şemayla mı başlıyor. */
function semaVarMi(metin: string): boolean {
  return SEMA.test(metin) && !KONAK_PORT.test(metin);
}

/**
 * Yazılan şey adres mi.
 *
 * `?` ile başlayan girdi **zorla arama**: kullanıcı "bu bir adres değil" demiş
 * oluyor ve `localhost` gibi belirsiz durumlarda tek çıkış yolu bu.
 */
export function adresMi(girdi: string): boolean {
  const metin = girdi.trim();
  if (metin === "" || metin.startsWith("?")) return false;

  const sema = SEMA.exec(metin);
  if (sema && !KONAK_PORT.test(metin)) {
    // `javascript:` ve `data:` bilinçli olarak dışarıda: ikisi de kullanıcıya
    // "şunu adres çubuğuna yapıştır" dedirten saldırıların taşıyıcısı.
    return IZINLI_SEMALAR.has(`${sema[1].toLowerCase()}:`);
  }

  // Boşluk varsa arama. `ornek com` bir adres değil.
  if (/\s/.test(metin)) return false;

  const [otorite] = metin.split(/[/?#]/, 1);
  const konak = otorite.replace(/^[^@]*@/, "").replace(/:\d+$/, "");

  if (konak === "localhost") return true;
  if (IPV4.test(konak)) return true;

  const etiketler = konak.split(".");
  if (etiketler.length < 2) return false;
  // Sondaki nokta (`ornek.com.`) boş etiket bırakıyor.
  const son = etiketler[etiketler.length - 1];
  if (son.length < 2) return false;
  // Üst düzey alan adları harf; `1.2` ya da `sürüm.3` adres değil.
  return /^[a-zA-Z¡-￿]{2,}$/.test(son);
}

/** Ham girdiyi ya adrese ya aramaya çevirir. */
export function coz(girdi: string): Cozum {
  const metin = girdi.trim();
  if (!adresMi(metin)) {
    return { tur: "arama", sorgu: metin.startsWith("?") ? metin.slice(1).trim() : metin };
  }
  return { tur: "adres", url: semaVarMi(metin) ? metin : `https://${metin}` };
}

/** Arama şablonunu doldurur (`ayarlar.aramaUrl`). */
export function aramaAdresi(sorgu: string, sablon: string): string {
  return sablon.replace("%s", encodeURIComponent(sorgu));
}

/**
 * Vurgulanacak alan adı: kayıtlanabilir alan (eTLD+1).
 *
 * `banka.com.saldirgan.net` → `saldirgan.net`
 */
export function alanAdi(konak: string): string {
  const kucuk = konak.toLowerCase().replace(/\.$/, "");
  if (kucuk === "localhost" || IPV4.test(kucuk)) return kucuk;

  const etiketler = kucuk.split(".");
  if (etiketler.length <= 2) return kucuk;

  const sonIki = etiketler.slice(-2).join(".");
  if (COK_PARCALI_SON_EKLER.includes(sonIki)) {
    return etiketler.slice(-3).join(".");
  }
  return sonIki;
}

/* ------------------------------------------------------------------ punycode */

const PUNY_TABAN = 36;
const PUNY_ESIK_ALT = 1;
const PUNY_ESIK_UST = 26;
const PUNY_SONUMLEME = 700;
const PUNY_CARPIKLIK = 38;
const PUNY_SAPMA = 72;
const PUNY_BASLANGIC = 128;

/** RFC 3492 `adapt`. */
function uyarla(fark: number, sayi: number, ilk: boolean): number {
  let d = ilk ? Math.floor(fark / PUNY_SONUMLEME) : Math.floor(fark / 2);
  d += Math.floor(d / sayi);
  let k = 0;
  while (d > ((PUNY_TABAN - PUNY_ESIK_ALT) * PUNY_ESIK_UST) / 2) {
    d = Math.floor(d / (PUNY_TABAN - PUNY_ESIK_ALT));
    k += PUNY_TABAN;
  }
  return k + Math.floor(((PUNY_TABAN - PUNY_ESIK_ALT + 1) * d) / (d + PUNY_CARPIKLIK));
}

/**
 * Tek bir `xn--` etiketini çözer. RFC 3492.
 *
 * Kendi yazılıyor çünkü tarayıcı bize her zaman punycode veriyor
 * (`new URL(...).hostname` IDNA ToASCII uyguluyor) ve ters yönde bir API yok.
 * Çözülemeyen etikette `null` dönüyor — o zaman punycode gösteriliyor.
 */
export function punycodeCoz(etiket: string): string | null {
  if (!etiket.toLowerCase().startsWith("xn--")) return etiket;
  const kodlu = etiket.slice(4);

  const ayirici = kodlu.lastIndexOf("-");
  const cikti: number[] = [];
  let baslangic = 0;
  if (ayirici > 0) {
    for (let i = 0; i < ayirici; i++) {
      const k = kodlu.charCodeAt(i);
      if (k >= 0x80) return null;
      cikti.push(k);
    }
    baslangic = ayirici + 1;
  }

  let n = PUNY_BASLANGIC;
  let i = 0;
  let sapma = PUNY_SAPMA;

  for (let konum = baslangic; konum < kodlu.length; ) {
    const eskiI = i;
    let agirlik = 1;
    for (let k = PUNY_TABAN; ; k += PUNY_TABAN) {
      if (konum >= kodlu.length) return null;
      const c = kodlu.charCodeAt(konum++);
      let rakam: number;
      if (c >= 0x30 && c <= 0x39) rakam = c - 0x30 + 26;
      else if (c >= 0x61 && c <= 0x7a) rakam = c - 0x61;
      else if (c >= 0x41 && c <= 0x5a) rakam = c - 0x41;
      else return null;

      i += rakam * agirlik;
      const t = k <= sapma ? PUNY_ESIK_ALT : k >= sapma + PUNY_ESIK_UST ? PUNY_ESIK_UST : k - sapma;
      if (rakam < t) break;
      agirlik *= PUNY_TABAN - t;
    }
    const uzunluk = cikti.length + 1;
    sapma = uyarla(i - eskiI, uzunluk, eskiI === 0);
    n += Math.floor(i / uzunluk);
    i %= uzunluk;
    cikti.splice(i, 0, n);
    i++;
  }

  try {
    return String.fromCodePoint(...cikti);
  } catch {
    return null;
  }
}

/**
 * Bu etiket Unicode olarak gösterilebilir mi.
 *
 * Kural bilinçli olarak **dar**: yalnız Latin harfleri (Türkçe ğ ş ı İ ö ç ü
 * dahil), rakam ve tire. Kiril `а`, Yunan `ο`, ve karışık yazımlar punycode
 * kalıyor — bunlar ekranda Latin harflerinden ayırt edilemiyor ve sahte alan
 * adının bir numaralı yöntemi.
 *
 * Bedeli dürüstçe: Rusça, Yunanca, Arapça, Çince alan adları punycode
 * görünüyor. Bir tarayıcı için ideal değil; çözümü tam Unicode güvenlik
 * profili ve o Faz 3'ün işi.
 */
export function unicodeGosterilebilir(metin: string): boolean {
  return /^[a-z0-9\-À-ſ]+$/i.test(metin);
}

/** Konak adını ekranda görüneceği hâle getirir. */
export function gosterilecekKonak(konak: string): string {
  return konak
    .split(".")
    .map((etiket) => {
      if (!etiket.toLowerCase().startsWith("xn--")) return etiket;
      const cozulmus = punycodeCoz(etiket);
      if (cozulmus === null || !unicodeGosterilebilir(cozulmus)) return etiket;
      return cozulmus;
    })
    .join(".");
}

export interface Gosterim {
  /** `https:` · `http:` · `file:` … Arayüz https'i gizliyor, ötekini gösteriyor. */
  sema: string;
  guvenli: boolean;
  /** Alt alan adları — soluk. Sonunda nokta var. */
  onEk: string;
  /** Kayıtlanabilir alan adı — vurgulu. */
  alan: string;
  /** Yol, sorgu, parça — soluk. */
  sonEk: string;
  /** Ayrıştırılamayan girdide her şey burada. */
  ham: string;
}

/**
 * Adresi vurgulanabilir parçalara böler.
 *
 * Ayrıştırılamayan girdi bir hata değil: kullanıcı yazarken adres çubuğunda
 * her zaman yarım bir dize var. O durumda her şey `ham` alanında dönüyor ve
 * arayüz düz metin çiziyor.
 */
export function gosterim(girdi: string): Gosterim {
  const bos: Gosterim = {
    sema: "",
    guvenli: false,
    onEk: "",
    alan: "",
    sonEk: "",
    ham: girdi,
  };
  if (girdi === YENI_SEKME) return { ...bos, ham: "" };

  let u: URL;
  try {
    u = new URL(girdi);
  } catch {
    return bos;
  }
  if (!IZINLI_SEMALAR.has(u.protocol)) return bos;
  if (u.protocol === "muiren:") return { ...bos, ham: "" };

  const konak = gosterilecekKonak(u.hostname);
  const alan = alanAdi(konak);
  const onEk = konak.endsWith(alan) ? konak.slice(0, konak.length - alan.length) : "";

  return {
    sema: u.protocol,
    guvenli: u.protocol === "https:",
    onEk,
    alan: alan + (u.port ? `:${u.port}` : ""),
    sonEk: `${u.pathname}${u.search}${u.hash}`,
    ham: girdi,
  };
}
