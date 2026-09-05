/**
 * Backend sözleşmesinin TypeScript karşılığı.
 *
 * `docs/IPC.md` **kanonik**; burası onun aynası. Rust tarafındaki yapılar
 * `#[serde(rename_all = "camelCase")]` taşıyor, dolayısıyla alan adları burada
 * `camelCase`.
 *
 * Yeni bir komut ya da alan önce `docs/IPC.md` içine yazılıyor (CLAUDE.md #3),
 * sonra Rust ve buraya.
 */

export type SekmeId = number;
export type GrupId = number;

export type Durum = "etkin" | "arkaplan" | "uyuyan" | "atilmis";

export type Sebep =
  | "bostaKaldi"
  | "sistemBaskisi"
  | "oyunModu"
  /** Pencere simge durumundaydı — kullanıcı tarayıcıya bakmıyordu. */
  | "pencereGizli"
  | "elle"
  | "uyanikSinir"
  | "etkilesim";

export type Asama = "basladi" | "bitti";

export interface SekmeOzeti {
  id: SekmeId;
  /** Adres çubuğunun göstereceği adres (devam eden gezinme varsa o). */
  url: string;
  /** Sayfadan geldi; backend temizledi. React'in kaçışına güveniliyor,
   *  `dangerouslySetInnerHTML` kullanılmıyor (`docs/Sekmeler.md`). */
  baslik: string;
  /** Başlık yoksa alan adı. Sekme çubuğu bunu yazıyor. */
  gorunenAd: string;
  /**
   * Favicon **kimliği** (içeriğin karması), adres değil.
   *
   * Uzak bir adresi kabuğa basmak, açılan her sitenin kabuk penceresine ağ
   * isteği yaptırabilmesi demek olurdu (`src-tauri/src/favicon/mod.rs`).
   * Baytları `faviconOku(kimlik)` veriyor ve `useFavicon` kimliğe göre
   * önbelleğe alıyor — kimlik içerikten geldiği için sınırsız
   * önbelleklenebilir.
   */
  favicon: string | null;
  durum: Durum;
  ebeveyn: SekmeId | null;
  /** Ağaçtaki derinlik; girinti buradan. Hesap backend'de (CLAUDE.md #2). */
  derinlik: number;
  sabit: boolean;
  uyutmaIstisnasi: boolean;
  sesCaliyor: boolean;
  sessiz: boolean;
  /** Doldurulmuş form var → atılmaz, uyutulabilir (koruma kuralı #4). */
  formDolu: boolean;
  /**
   * Gizli sekme (InPrivate). Sekme çubuğunda ayrı bir işaretle çiziliyor:
   * kullanıcının hangi sekmenin gizli olduğunu **görmesi** gerekiyor, yoksa
   * gizli sandığı sekmede geçmişe yazan bir arama yapıyor.
   */
  gizli: boolean;
  grup: GrupId | null;
  /** Grubu katlanmış; sekme çubuğunda çizilmiyor. */
  katli: boolean;
  /** Grubundan gelen uyku eşiği; `null` ise genel ayar geçerli. */
  grupUykuEsigiSn: number | null;
  /** Sayfa tam ekran bir öğe gösteriyor (koruma kuralı #8). */
  tamEkran: boolean;
  yukleniyor: boolean;
  geriVar: boolean;
  ileriVar: boolean;
  etkin: boolean;
  bostaSn: number;
}

/** Kurulu WebView2 Runtime'ın gerçekten desteklediği çağrılar. */
export interface Yetenekler {
  motor: boolean;
  askiyaAlma: boolean;
  bellekHedefi: boolean;
  surecBilgisi: boolean;
  /** `ICoreWebView2_4`: indirme olayı. Yoksa Muiget köprüsü indirmeyi
   *  yakalayamıyor ve motor kendi indirmesini yapıyor. */
  indirmeOlayi: boolean;
  /** `ICoreWebView2_15`: favicon. Yoksa sekmede yalnız durum noktası var. */
  favicon: boolean;
}

export interface Dikdortgen {
  x: number;
  y: number;
  genislik: number;
  yukseklik: number;
}

export type Profil = "siki" | "dengeli" | "rahat";
export type SurecPolitikasi = "birlesik" | "varsayilan";
/** İndirmeyi kim yapacak (`docs/Kopruler.md`). */
export type IndirmePolitikasi = "muiget" | "sor" | "motorda";

export interface Settings {
  profil: Profil;
  uykuEsigiSn: number;
  atmaEsigiSn: number;
  uyanikUstSinir: number;
  gozcuPeriyoduSn: number;
  istisnaAlanlari: string[];
  surecPolitikasi: SurecPolitikasi;
  rendererTavani: number;
  aramaUrl: string;
  tema: string;
  gecmisSaklamaGun: number;
  indirmePolitikasi: IndirmePolitikasi;
  /** Pop-up ve yönlendirme engelleme. Varsayılan **açık**. */
  engellemeAcik: boolean;
  /** Kullanıcının filtre listeleri. Varsayılan **kapalı** (karar #5). */
  filtreAcik: boolean;
  filtreKurallari: string[];
  /** Tam ekran oyun algılandığında sekmeler uyusun mu. Varsayılan kapalı. */
  oyunAlgilama: boolean;
  /**
   * Oyun modunda kabuk penceresi simge durumuna alınsın mı
   * (`docs/Bellek.md`, "Oyun modu" 4. adım). Varsayılan **kapalı**.
   *
   * Gizlemek değil küçültmek: gizlenen pencerenin görev çubuğu düğmesi de
   * kaybolur ve tam ekran bir oyunun altında kullanıcının geri dönecek
   * görünür bir yolu kalmaz.
   */
  oyunPencereGizle: boolean;
}

/* --------------------------------------------------------------------- hata */

/**
 * Backend Türkçe metin göndermiyor (`docs/IPC.md`); cümleyi arayüz kuruyor.
 * `ayrinti` çevrilmeyen teknik veri.
 */
export type HataTuru =
  | "sekmeYok"
  | "gecersizAdres"
  | "motorYok"
  | "yetenekYok"
  | "motor"
  | "dosya"
  | "bicim"
  | "kopruYok"
  | "kopru";

export interface MuirenHata {
  tur: HataTuru;
  ayrinti?: unknown;
}

/** Hatanın kullanıcıya gösterilecek Türkçe karşılığı. */
export function hataMetni(hata: unknown): string {
  const h = hata as Partial<MuirenHata> | undefined;
  switch (h?.tur) {
    case "sekmeYok":
      return "Bu sekme kapanmış.";
    case "gecersizAdres":
      return "Bu adres açılamıyor.";
    case "motorYok":
      return "Bu sürüm sayfa açamıyor (motorsuz derleme).";
    case "yetenekYok":
      return "Kurulu WebView2 sürümü bu işi desteklemiyor.";
    case "motor":
      return "Sayfa katmanı yanıt vermedi.";
    case "dosya":
      return "Dosya yazılamadı.";
    case "bicim":
      return "Dosya biçimi okunamadı.";
    case "kopruYok":
      // Bir hata değil, bir yokluk (`docs/Kopruler.md`). Buraya düşmesi
      // yarış demek: menü öğesi zaten çizilmemeliydi.
      return `${String(h.ayrinti ?? "Kardeş uygulama")} kurulu değil.`;
    case "kopru":
      return "Kardeş uygulamaya gönderilemedi.";
    default:
      return "Beklenmeyen bir sorun oldu.";
  }
}

/* -------------------------------------------------------------------- olaylar */

/** `docs/IPC.md` tablosuyla birebir aynı; Rust karşılığı `src/olaylar.rs`. */
export const OLAY = {
  sekmeDegisti: "muiren://sekme-degisti",
  sekmeGuncellendi: "muiren://sekme-guncellendi",
  sekmeDurumDegisti: "muiren://sekme-durum-degisti",
  bellekOzeti: "muiren://bellek-ozeti",
  gezinme: "muiren://gezinme",
  indirmeOnerisi: "muiren://indirme-onerisi",
  temaDegisti: "muiren://tema-degisti",
  kopruDurumu: "muiren://kopru-durumu",
  oyunModu: "muiren://oyun-modu",
  engellendi: "muiren://engellendi",
  kisayol: "muiren://kisayol",
} as const;

/* ------------------------------------------------------------------ köprüler */

export interface KopruDurumu {
  ad: string;
  kurulu: boolean;
  /** Teşhis için; arayüzde yalnız ayrıntı bölümünde görünüyor. */
  yol: string | null;
}

/** `muiren://oyun-modu` yükü. */
export interface OyunModuOlayi {
  acik: boolean;
  /**
   * `elle` kullanıcının düğmesi, `algilama` bizim sezgimiz.
   *
   * `muifly` diye bir kaynak **yok**: `muifly://durum` süreç içi bir Tauri
   * olayı ve süreçler arası bir uç bulunmuyor
   * (`src-tauri/src/bridge/muifly.rs`).
   */
  kaynak: "elle" | "algilama";
}

/** `muiren://indirme-onerisi` yükü. */
export interface IndirmeOlayi {
  id: SekmeId;
  url: string;
  /** Temizlenmiş ad; `null` ise motorun önerdiği addan geriye bir şey
   *  kalmamış. */
  dosyaAdi: string | null;
  /** Sunucunun bildirdiği boyut; bilinmiyorsa 0. */
  boyut: number;
  sonuc: "soruluyor" | "devredildi" | "basarisiz";
}

/* ---------------------------------------------------------------- engelleme */

export type EngelSebebi = "popUp" | "yonlendirme" | "filtre";

/** `muiren://engellendi` yükü. */
export interface EngelOlayi {
  id: SekmeId;
  url: string;
  sebep: EngelSebebi;
  /** Bu sayfada engellenen toplam istek; adres çubuğundaki rozet. */
  sayi: number;
}

export interface EngelOzeti {
  filtreAcik: boolean;
  popupAcik: boolean;
  kuralSayisi: number;
  /** Ayrıştırılamayan satırlar. Sessizce atmak, kullanıcının çalışmayan bir
   *  listeyle dolaşması demek (`src-tauri/src/engel/liste.rs`). */
  anlasilmayanlar: string[];
}

/* ----------------------------------------------------------- veri temizleme */

export interface Temizlik {
  /** Muiren'in geçmiş veritabanı. **Yer imleri dahil değil.** */
  gecmis: boolean;
  favicon: boolean;
  cerezler: boolean;
  onbellek: boolean;
  siteVerisi: boolean;
  otomatikDoldurma: boolean;
}

export interface TemizlikRaporu {
  gecmisKaydi: number;
  faviconDosyasi: number;
  /** Motorun silmesi eşzamansız: yalnız "başlatıldı". */
  motorBaslatildi: boolean;
}

/* -------------------------------------------------------------------- grup */

export type GrupRengi =
  | "teal"
  | "mor"
  | "kehribar"
  | "kirmizi"
  | "yesil"
  | "mavi"
  | "gri";

export interface Grup {
  id: GrupId;
  ad: string;
  renk: GrupRengi;
  katli: boolean;
  /** `null` ise genel uyku eşiği geçerli. */
  uykuEsigiSn: number | null;
}

export interface DurumOlayi {
  id: SekmeId;
  durum: Durum;
  sebep: Sebep;
}

export interface GezinmeOlayi {
  id: SekmeId;
  asama: Asama;
  url: string;
  basarili: boolean;
}

/* --------------------------------------------------------------------- bellek */

export type BellekBaskisi = "dusuk" | "orta" | "yuksek" | "kritik";

/**
 * Bellek panelinin beslendiği özet (`docs/IPC.md`).
 *
 * `olcumYaklasik` true ise arayüz `~` işareti ve bir ipucu ekliyor —
 * **yaklaşık değeri kesin gibi göstermek yok** (`docs/Bellek.md`).
 */
export interface BellekOzeti {
  baski: BellekBaskisi;
  /** WebView2 süreçlerinin toplamı (MB). */
  toplamMb: number;
  kabukMb: number;
  sistemToplamMb: number;
  sistemBosMb: number;
  etkin: number;
  arkaplan: number;
  uyuyan: number;
  atilmis: number;
  tahminiKazancMb: number;
  olcumYaklasik: boolean;
  /**
   * Sekme başına ölçülen bellek; yalnız değeri **olan** sekmeler.
   *
   * `SekmeOzeti` içinde değil çünkü o yapı eşik kararının girdisi ve karar
   * sekme başına rakama dayanmıyor (`docs/Bellek.md`). Arayüz iki listeyi
   * `id` üzerinden birleştiriyor.
   */
  sekmeMb: SekmeBellegi[];
  /**
   * Sekmelere düşmeyen WebView2 belleği: tarayıcı, GPU, ağ, yardımcı
   * süreçler **ve kabuk arayüzünün kendi render süreci**. Sonuncusu
   * `kabukMb` değil — o, Muiren'in Rust sürecinin kendisi.
   */
  ortakMb: number;
  /** Faz 0/R4'ün ölçüm aracı: `--process-per-site` geçti mi. */
  surecSayisi: number;
}

/** Bir sekmenin ölçülen belleği (`memory/esleme.rs`). */
export interface SekmeBellegi {
  id: SekmeId;
  mb: number;
  /**
   * Render süreci başka sekmelerle paylaşılıyor → rakam bölüştürülmüş.
   *
   * Arayüz bunu söylemek zorunda: paylaşılan bir rakamı tek sekmenin
   * maliyeti gibi göstermek, kullanıcıyı yanlış sekmeyi kapatmaya iter.
   */
  paylasimli: boolean;
  /** Bu turda ölçülmedi; son bilinen değer (uyuyan/atılmış sekme). */
  bayat: boolean;
}

/**
 * Tek bir uyanma kaynağının dağılımı (`docs/IPC.md`, `gecikme_ozeti`).
 *
 * Ortalama **yok**: tek bir 8 saniyelik uyanma otuz iyi örneğin ortalamasını
 * bozup tabloyu olduğundan kötü gösterirdi. Medyan tipik hâli, p95 ve
 * `enKotuMs` kuyruğu söylüyor.
 */
export interface Dagilim {
  n: number;
  medyanMs: number;
  p95Ms: number;
  enKotuMs: number;
}

/**
 * Uyanma gecikmesi — `docs/Bellek.md` kabul tablosunun iki satırı.
 *
 * `null` = **henüz ölçülmedi**. Sıfır göstermek yok: hiç sekme uyandırmamış
 * kullanıcıya "medyan 0 ms" demek, ölçülmemiş bir şeyi ölçülmüş gibi
 * göstermek olurdu.
 *
 * İkisi farklı şeyler ölçüyor ve panel bunu etiketliyor: `uyuyan` gerçekten
 * "etkileşime hazır" (sayfanın JS iş parçacığı cevap verdi), `atilmis` ise
 * yüklemenin bitişi — yani gerçek ilk boyanın **üst sınırı**.
 */
export interface GecikmeOzeti {
  uyuyan: Dagilim | null;
  atilmis: Dagilim | null;
  hedefUyuyanMs: number;
  hedefAtilmisMs: number;
}

/* -------------------------------------------------------------------- kısayol */

/**
 * Backend'in arayüze devrettiği kısayollar (`docs/IPC.md`, `muiren://kisayol`).
 *
 * Burada **tablo yok**, yalnız eylem adları: hangi tuşun hangi eylemi ürettiği
 * `src-tauri/src/kisayol.rs` içinde ve tek kaynak orası.
 */
export type KisayolIsi =
  | "adresOdak"
  | "sekmeArama"
  | "bellekPaneli"
  | "yanPanel"
  | "tamEkran";
// `gizliSekme` burada YOK ve olmamalı: o bir backend eylemi
// (`Kisayol::arayuz_isi() == false`), yani `Ctrl+Shift+N` sekmeyi backend
// açıyor ve arayüz olayı `sekme-degisti` ile öğreniyor.

export interface KisayolOlayi {
  is: KisayolIsi;
}

/* ------------------------------------------------------ geçmiş ve yer imleri */

export interface GecmisKaydi {
  id: number;
  url: string;
  baslik: string;
  /** Unix epoch, saniye. */
  ziyaret: number;
  /** Bu adrese kaç kez gidildi; önerilerde sıralama ölçütü. */
  sayac: number;
  alan: string;
}

export interface YerImi {
  id: number;
  url: string;
  baslik: string;
  klasor: number | null;
  sira: number;
  eklendi: number;
}

export type Aralik = "sonSaat" | "sonGun" | "sonHafta" | "hepsi";

/* ---------------------------------------------------------------------- tema */

/**
 * **Doğrulanmış** tema (`docs/Temalar.md`).
 *
 * `jetonlar` doğrudan `documentElement.style` üzerine yazılıyor. Her değer
 * backend'in `theme/jeton.rs` süzgecinden geçti; arayüzde ikinci bir süzgeç
 * **yok ve olmamalı** — iki süzgeç, biri gevşediğinde fark edilmeyen bir açık
 * demek.
 */
export interface TemaOzeti {
  ad: string;
  yazar: string;
  surum: string;
  /** Yerleşik temalar silinemiyor. */
  yerlesik: boolean;
  /** `[--jeton, deger]` çiftleri. */
  jetonlar: [string, string][];
  arkaplan: Arkaplan | null;
  /** Beyaz listede olmayan ya da doğrulamadan geçemeyen anahtarlar. */
  atilanJetonlar: string[];
  /** Kontrast uyarıları. Tema reddedilmiyor ama kullanıcı görüyor. */
  uyarilar: string[];
}

export interface Arkaplan {
  yol: string;
  yerlesim: string;
  karartma: number;
  bulanik: number;
}
