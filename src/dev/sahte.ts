/**
 * Sahte backend — **yalnız geliştirme**, yalnız `?sahte` sorgusuyla.
 *
 * Kabuk düz bir tarayıcıda açılabiliyor (`PencereDugmeleri` bunu bilerek
 * mümkün kılıyor) ama Tauri bağlamı olmadığı için her komut fırlatıyor ve
 * ekranda boş bir şerit kalıyor. Arayüz hatasını ayıklamanın en ucuz yolu
 * böylece en işe yaramaz yol oluyordu: görülecek bir şey yok.
 *
 * Bu dosya o boşluğu dolduruyor. `window.__TAURI_INTERNALS__` yerine geçip
 * komutlara bellek içi bir defterden cevap veriyor: sekmeler açılıyor,
 * kapanıyor, uyuyor, gezinme oluyor, bellek özeti akıyor.
 *
 * **Bu bir test değil ve bir sözleşme değil.** Kararların doğruluğu Rust
 * tarafında test ediliyor (CLAUDE.md #2); buradaki mantık yalnız ekranda
 * gerçekçi bir tablo çizmek için var. `docs/IPC.md` ile ayrışırsa hata
 * buradadır, orada değil.
 *
 * Üretim paketine **girmiyor**: çağıran yer `import.meta.env.DEV` ile
 * korunuyor ve dinamik içe aktarım derlemede eleniyor.
 *
 * Kullanım: `npm run dev` → `http://localhost:1420/?sahte`
 */

import type {
  BellekOzeti,
  Dikdortgen,
  GecikmeOzeti,
  GecmisKaydi,
  Grup,
  SekmeId,
  SekmeOzeti,
  Settings,
  TemaOzeti,
  Yetenekler,
} from "../ipc/tipler";
import { OLAY } from "../ipc/tipler";

/* ------------------------------------------------------------------ olay ucu */

type GeriCagri = (deger: unknown) => void;

const geriCagrilar = new Map<number, GeriCagri>();
/** olay adı → geri çağrı kimlikleri */
const dinleyiciler = new Map<string, Set<number>>();
let sonrakiKimlik = 1;

function yayinla(olay: string, yuk: unknown) {
  const kume = dinleyiciler.get(olay);
  if (!kume) return;
  for (const kimlik of kume) {
    geriCagrilar.get(kimlik)?.({ event: olay, id: kimlik, payload: yuk });
  }
}

/* --------------------------------------------------------------------- veri */

/**
 * Açılış tablosu.
 *
 * Gerçekçi olmak zorunda: 4 sekmelik bir ekranda daralma, kaydırma, grup
 * başlığı ve uyuyan sekme solması görünmüyor — yani tam da bu ekranda
 * ayarlanması gereken şeyler görünmüyor.
 */
const TOHUM: Array<Partial<SekmeOzeti> & { url: string; baslik: string }> = [
  { url: "https://muilabs.dev/muiren", baslik: "Muiren — hafif tarayıcı", sabit: true },
  { url: "https://github.com/mui/muiren", baslik: "mui/muiren: kaynak", sabit: true },
  { url: "https://crunchyroll.com/tr/frieren", baslik: "Frieren 12. bölüm", sesCaliyor: true, grup: 1 },
  { url: "https://anilist.co/anime/154587", baslik: "AniList — Sousou no Frieren", grup: 1, ebeveyn: 3, derinlik: 1 },
  { url: "https://myanimelist.net/anime/52991", baslik: "MyAnimeList — Frieren", grup: 1, ebeveyn: 3, derinlik: 1 },
  { url: "https://store.steampowered.com/app/1245620", baslik: "ELDEN RING on Steam", grup: 2 },
  { url: "https://eldenring.wiki.fextralife.com", baslik: "Elden Ring Wiki", grup: 2, durum: "uyuyan", bostaSn: 940 },
  { url: "https://reddit.com/r/Eldenring", baslik: "r/Eldenring", grup: 2, durum: "uyuyan", bostaSn: 1320 },
  { url: "https://doc.rust-lang.org/book/", baslik: "The Rust Programming Language", durum: "uyuyan", bostaSn: 2100 },
  { url: "https://tauri.app/v2/guides/", baslik: "Tauri 2.0 Rehber", durum: "uyuyan", bostaSn: 2640 },
  { url: "https://news.ycombinator.com", baslik: "Hacker News", durum: "atilmis", bostaSn: 5400 },
  { url: "https://developer.mozilla.org/tr/docs/Web/CSS", baslik: "CSS | MDN", durum: "atilmis", bostaSn: 7200 },
  { url: "https://youtube.com/watch?v=xxxxxxxx", baslik: "Lo-fi çalışma yayını", sessiz: true, durum: "arkaplan", bostaSn: 480 },
  { url: "https://duckduckgo.com/?q=webview2+trysuspend", baslik: "webview2 trysuspend — DuckDuckGo", gizli: true, durum: "arkaplan", bostaSn: 60 },
];

const GRUPLAR: Grup[] = [
  { id: 1, ad: "Anime", renk: "mor", katli: false, uykuEsigiSn: 1800 },
  { id: 2, ad: "Elden Ring", renk: "kehribar", katli: false, uykuEsigiSn: null },
];

/** Alan adı → tek renkli sahte favicon. Ağ isteği yok, `data:` üretiliyor. */
const FAVICON_RENK = ["#2dd4bf", "#a78bfa", "#f59e0b", "#ef4444", "#22c55e", "#3b82f6"];

function faviconUret(kimlik: string): string {
  const n = [...kimlik].reduce((a, c) => a + c.charCodeAt(0), 0);
  const renk = FAVICON_RENK[n % FAVICON_RENK.length];
  const harf = kimlik.slice(0, 1).toUpperCase();
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">` +
    `<rect width="16" height="16" rx="4" fill="${renk}"/>` +
    `<text x="8" y="12" font-family="sans-serif" font-size="10" font-weight="700" ` +
    `text-anchor="middle" fill="#0f1115">${harf}</text></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

function alanAdi(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

let sonrakiSekme = 1;

function sekmeYap(t: (typeof TOHUM)[number]): SekmeOzeti {
  const id = sonrakiSekme++;
  return {
    id,
    gorunenAd: t.baslik || alanAdi(t.url),
    favicon: alanAdi(t.url),
    durum: "arkaplan",
    ebeveyn: null,
    derinlik: 0,
    sabit: false,
    uyutmaIstisnasi: false,
    sesCaliyor: false,
    sessiz: false,
    formDolu: false,
    gizli: false,
    grup: null,
    katli: false,
    grupUykuEsigiSn: null,
    tamEkran: false,
    yukleniyor: false,
    geriVar: true,
    ileriVar: false,
    etkin: false,
    bostaSn: 30,
    ...t,
  };
}

const sekmeler: SekmeOzeti[] = TOHUM.map(sekmeYap);
sekmeler[2].etkin = true;
sekmeler[2].durum = "etkin";
sekmeler[2].bostaSn = 0;

const gruplar: Grup[] = [...GRUPLAR];

const GECMIS: GecmisKaydi[] = [
  ["https://github.com/mui/muiren", "mui/muiren: kaynak", 84],
  ["https://crunchyroll.com/tr", "Crunchyroll", 61],
  ["https://doc.rust-lang.org/book/", "The Rust Programming Language", 47],
  ["https://tauri.app/v2/guides/", "Tauri 2.0 Rehber", 39],
  ["https://news.ycombinator.com", "Hacker News", 33],
  ["https://developer.mozilla.org/tr/docs/Web/CSS", "CSS | MDN", 28],
  ["https://anilist.co", "AniList", 24],
  ["https://store.steampowered.com", "Steam", 19],
  ["https://excalidraw.com", "Excalidraw", 12],
  ["https://muilabs.dev/muiren", "Muiren — hafif tarayıcı", 9],
].map(([url, baslik, sayac], i) => ({
  id: i + 1,
  url: url as string,
  baslik: baslik as string,
  ziyaret: Math.floor(Date.now() / 1000) - i * 3600,
  sayac: sayac as number,
  alan: alanAdi(url as string),
}));

const YER_IMLERI = [
  { id: 1, url: "https://crunchyroll.com/tr", baslik: "Crunchyroll", klasor: null, sira: 0, eklendi: 0 },
  { id: 2, url: "https://doc.rust-lang.org/book/", baslik: "Rust Kitabı", klasor: null, sira: 1, eklendi: 0 },
  { id: 3, url: "https://excalidraw.com", baslik: "Excalidraw", klasor: null, sira: 2, eklendi: 0 },
];

const ayarlar: Settings = {
  profil: "dengeli",
  uykuEsigiSn: 900,
  atmaEsigiSn: 3600,
  uyanikUstSinir: 12,
  gozcuPeriyoduSn: 20,
  istisnaAlanlari: ["mail.google.com", "figma.com"],
  surecPolitikasi: "birlesik",
  rendererTavani: 8,
  aramaUrl: "https://duckduckgo.com/?q={sorgu}",
  tema: "gece",
  gecmisSaklamaGun: 90,
  indirmePolitikasi: "sor",
  engellemeAcik: true,
  filtreAcik: false,
  filtreKurallari: [],
  oyunAlgilama: true,
  oyunPencereGizle: false,
};

const YETENEKLER: Yetenekler = {
  motor: true,
  askiyaAlma: true,
  bellekHedefi: true,
  surecBilgisi: true,
  indirmeOlayi: true,
  favicon: true,
};

/* ------------------------------------------------------------------- özetler */

function bellekOzeti(): BellekOzeti {
  const say = (d: SekmeOzeti["durum"]) => sekmeler.filter((s) => s.durum === d).length;
  const uyanik = sekmeler.filter((s) => s.durum === "etkin" || s.durum === "arkaplan");
  const sekmeMb = uyanik.map((s) => ({
    id: s.id,
    mb: 90 + ((s.id * 37) % 160),
    paylasimli: s.id % 3 === 0,
    bayat: false,
  }));
  const toplam = sekmeMb.reduce((a, v) => a + v.mb, 0) + 210;
  return {
    baski: "orta",
    toplamMb: toplam,
    kabukMb: 46,
    sistemToplamMb: 16384,
    sistemBosMb: 5120,
    etkin: say("etkin"),
    arkaplan: say("arkaplan"),
    uyuyan: say("uyuyan"),
    atilmis: say("atilmis"),
    tahminiKazancMb: say("uyuyan") * 118 + say("atilmis") * 186,
    olcumYaklasik: false,
    sekmeMb,
    ortakMb: 210,
    surecSayisi: 9,
  };
}

const GECIKME: GecikmeOzeti = {
  uyuyan: { n: 24, medyanMs: 174, p95Ms: 288, enKotuMs: 412 },
  atilmis: { n: 11, medyanMs: 1180, p95Ms: 1840, enKotuMs: 2260 },
  hedefUyuyanMs: 300,
  hedefAtilmisMs: 2000,
};

const TEMA: TemaOzeti = {
  ad: "gece",
  yazar: "Mui",
  surum: "1.0",
  yerlesik: true,
  jetonlar: [],
  arkaplan: null,
  atilanJetonlar: [],
  uyarilar: [],
};

/* ------------------------------------------------------------------ yardımcı */

function listeYayinla() {
  yayinla(OLAY.sekmeDegisti, sekmeler);
  yayinla(OLAY.bellekOzeti, bellekOzeti());
}

function bul(id: SekmeId) {
  return sekmeler.find((s) => s.id === id);
}

function etkinlestir(id: SekmeId) {
  for (const s of sekmeler) {
    if (s.etkin && s.id !== id) s.durum = "arkaplan";
    s.etkin = s.id === id;
  }
  const s = bul(id);
  if (s) {
    s.durum = "etkin";
    s.bostaSn = 0;
  }
}

/** Adres mi arama mı — backend'deki `tabs::adres_mi` kararının kaba taklidi. */
function adrese_cevir(girdi: string): string {
  const g = girdi.trim();
  if (/^[a-z][a-z0-9+.-]*:/i.test(g)) return g;
  if (/^[^\s/]+\.[^\s/]{2,}/.test(g)) return `https://${g}`;
  return ayarlar.aramaUrl.replace("{sorgu}", encodeURIComponent(g));
}

/* -------------------------------------------------------------------- komut */

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Arg = any;

function cagir(komut: string, arg: Arg): unknown {
  switch (komut) {
    /* --- olay eklentisi --- */
    case "plugin:event|listen": {
      const olay = arg.event as string;
      const kimlik = arg.handler as number;
      if (!dinleyiciler.has(olay)) dinleyiciler.set(olay, new Set());
      dinleyiciler.get(olay)!.add(kimlik);
      return kimlik;
    }
    case "plugin:event|unlisten": {
      dinleyiciler.get(arg.event as string)?.delete(arg.eventId as number);
      return null;
    }

    /* --- pencere --- */
    case "plugin:window|is_fullscreen":
    case "plugin:window|is_maximized":
      return false;
    case "plugin:window|minimize":
    case "plugin:window|maximize":
    case "plugin:window|unmaximize":
    case "plugin:window|toggle_maximize":
    case "plugin:window|set_fullscreen":
    case "plugin:window|close":
    case "plugin:window|destroy":
      return null;

    /* --- sekmeler --- */
    case "sekme_listesi":
      return sekmeler;
    case "sekme_ac": {
      const s = sekmeYap({
        url: (arg?.url as string) ?? "muiren://yeni",
        baslik: "",
        gizli: Boolean(arg?.gizli),
      });
      s.gorunenAd = s.url === "muiren://yeni" ? "Yeni sekme" : alanAdi(s.url);
      s.favicon = s.url === "muiren://yeni" ? null : alanAdi(s.url);
      sekmeler.push(s);
      etkinlestir(s.id);
      listeYayinla();
      return s.id;
    }
    case "sekme_kapat": {
      const i = sekmeler.findIndex((s) => s.id === arg.id);
      if (i !== -1) {
        const kapanan = sekmeler.splice(i, 1)[0];
        if (kapanan.etkin && sekmeler.length > 0) {
          etkinlestir(sekmeler[Math.min(i, sekmeler.length - 1)].id);
        }
      }
      listeYayinla();
      return null;
    }
    case "sekme_etkinlestir":
      etkinlestir(arg.id);
      listeYayinla();
      return null;
    case "sekme_tasi": {
      const i = sekmeler.findIndex((s) => s.id === arg.id);
      if (i !== -1) sekmeler.splice(arg.hedef, 0, sekmeler.splice(i, 1)[0]);
      listeYayinla();
      return null;
    }
    case "sekme_sabitle": {
      const s = bul(arg.id);
      if (s) s.sabit = arg.sabit;
      listeYayinla();
      return null;
    }
    case "sekme_sessize_al": {
      const s = bul(arg.id);
      if (s) s.sessiz = arg.sessiz;
      listeYayinla();
      return null;
    }
    case "sekme_uyutma_istisnasi": {
      const s = bul(arg.id);
      if (s) s.uyutmaIstisnasi = arg.istisna;
      listeYayinla();
      return null;
    }
    case "sekme_uyut": {
      const s = bul(arg.id);
      if (s) s.durum = "uyuyan";
      listeYayinla();
      return null;
    }
    case "sekme_at": {
      const s = bul(arg.id);
      if (s) s.durum = "atilmis";
      listeYayinla();
      return null;
    }
    case "sekme_geri_al":
      return null;
    case "hepsini_uyut": {
      let n = 0;
      for (const s of sekmeler) {
        if (!s.etkin && !s.sabit && s.durum === "arkaplan") {
          s.durum = "uyuyan";
          n++;
        }
      }
      listeYayinla();
      return n;
    }

    /* --- gezinme --- */
    case "gezin": {
      const s = bul(arg.id);
      if (s) {
        s.url = adrese_cevir(arg.girdi as string);
        s.baslik = "";
        s.gorunenAd = alanAdi(s.url);
        s.favicon = alanAdi(s.url);
        s.durum = s.etkin ? "etkin" : "arkaplan";
        s.yukleniyor = true;
        yayinla(OLAY.sekmeGuncellendi, s);
        window.setTimeout(() => {
          s.yukleniyor = false;
          s.baslik = `${alanAdi(s.url)} — sahte sayfa`;
          s.gorunenAd = s.baslik;
          yayinla(OLAY.sekmeGuncellendi, s);
        }, 400);
      }
      return null;
    }
    case "geri":
    case "ileri":
    case "yenile":
    case "durdur":
      return null;

    /* --- kabuk --- */
    case "icerik_alani":
      sonAlan = arg.alan as Dikdortgen;
      return null;
    case "ortu_gorunur":
      return null;
    case "yetenekler":
      return YETENEKLER;
    case "kisayol_bas":
      return false;

    /* --- ayarlar / bellek --- */
    case "ayarlar_oku":
      return ayarlar;
    case "ayarlar_yaz":
      Object.assign(ayarlar, arg.ayarlar);
      return ayarlar;
    case "bellek_ozeti":
      return bellekOzeti();
    case "gecikme_ozeti":
      return GECIKME;
    case "gecikme_sifirla":
      return null;
    case "oyun_modu":
      yayinla(OLAY.oyunModu, { acik: arg.acik, kaynak: "elle" });
      return null;

    /* --- geçmiş / yer imleri --- */
    case "gecmis_ara": {
      const q = (arg.sorgu as string).toLocaleLowerCase("tr").trim();
      const liste = q === ""
        ? GECMIS
        : GECMIS.filter(
            (k) =>
              k.baslik.toLocaleLowerCase("tr").includes(q) ||
              k.url.toLocaleLowerCase("tr").includes(q),
          );
      return liste.slice(0, (arg.limit as number) ?? 8);
    }
    case "gecmis_sil":
    case "gecmis_temizle":
      return 0;
    case "yer_imi_listesi":
      return YER_IMLERI;
    case "yer_imi_mi": {
      const y = YER_IMLERI.find((k) => k.url === arg.url);
      return y ? y.id : null;
    }
    case "yer_imi_ekle":
      return 99;
    case "yer_imi_sil":
      return null;
    case "yer_imi_klasorleri":
      return [];

    /* --- tema --- */
    case "tema_listesi":
      return [TEMA];
    case "tema_etkin":
      return TEMA;
    case "tema_arkaplan":
      return null;
    case "tema_uygula":
      return TEMA;

    /* --- gruplar --- */
    case "grup_listesi":
      return gruplar;
    case "grup_guncelle": {
      const g = gruplar.find((x) => x.id === arg.id);
      if (g) {
        if (arg.katli !== undefined && arg.katli !== null) g.katli = arg.katli;
        if (arg.ad) g.ad = arg.ad;
        if (arg.renk) g.renk = arg.renk;
        for (const s of sekmeler) if (s.grup === g.id) s.katli = g.katli;
      }
      yayinla("muiren://grup-degisti", gruplar);
      listeYayinla();
      return null;
    }

    /* --- köprüler / engelleme / favicon --- */
    case "kopru_durumu":
    case "kopru_tazele":
      return [
        { ad: "Muiget", kurulu: true, yol: "C:\\Mui\\Muiget.exe" },
        { ad: "Muiply", kurulu: true, yol: "C:\\Mui\\Muiply.exe" },
        { ad: "Muiwatch", kurulu: false, yol: null },
        { ad: "Muifly", kurulu: false, yol: null },
      ];
    case "engel_ozeti":
      return { filtreAcik: false, popupAcik: true, kuralSayisi: 0, anlasilmayanlar: [] };
    case "engel_sayaci":
      return 3;
    case "favicon_oku":
      return faviconUret(arg.kimlik as string);

    default:
      console.warn(`sahte: bilinmeyen komut "${komut}"`, arg);
      return null;
  }
}

let sonAlan: Dikdortgen = { x: 0, y: 0, genislik: 0, yukseklik: 0 };

/* --------------------------------------------------------------------- kur */

/** Sahte backend'i kurar. Yalnız `main.tsx` çağırıyor, yalnız DEV'de. */
export function kur(): void {
  const ic = {
    transformCallback(geri: GeriCagri): number {
      const kimlik = sonrakiKimlik++;
      geriCagrilar.set(kimlik, geri);
      return kimlik;
    },
    invoke(komut: string, arg: Arg): Promise<unknown> {
      try {
        return Promise.resolve(cagir(komut, arg ?? {}));
      } catch (e) {
        return Promise.reject(e);
      }
    },
    metadata: { currentWindow: { label: "ana" }, currentWebview: { label: "ana" } },
    plugins: {},
    convertFileSrc: (yol: string) => yol,
  };
  // @ts-expect-error — Tauri'nin iç yüzeyi tip olarak dışa verilmiyor.
  window.__TAURI_INTERNALS__ = ic;

  // `unlisten()` bu ayrı yüzeyden geçiyor (`@tauri-apps/api/event`); yoksa
  // her bileşen sökülüşünde yakalanmamış bir söz reddi düşüyor ve gerçek
  // hatalar konsolda kayboluyor.
  (window as unknown as Record<string, unknown>).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener(olay: string, kimlik: number) {
      dinleyiciler.get(olay)?.delete(kimlik);
      geriCagrilar.delete(kimlik);
      return Promise.resolve();
    },
  };

  // Boşta kalma sayacı ilerlesin: "3 dk boşta" yazan bir panel donuk
  // durursa ekranda ölçüm olduğu anlaşılmıyor.
  window.setInterval(() => {
    for (const s of sekmeler) if (!s.etkin) s.bostaSn += 5;
    listeYayinla();
  }, 5000);

  console.info(
    "muiren: sahte backend kuruldu (yalnız geliştirme). İçerik alanı:",
    sonAlan,
  );
}
