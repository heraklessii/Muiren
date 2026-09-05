/**
 * Teşhis yorumlaması — **saf ve testli** (`docs/Medya.md`).
 *
 * `docs/Medya.md` üç şey istiyordu: donanım kod çözmeyi devre dışı
 * bırakmamak, **düştüğünü fark etmek** ve **ölçüm sunmak**. İkincisi ile
 * üçüncüsü bir ekran gerektiriyor; bu dosya o ekranın karar veren yarısı.
 *
 * Ölçümün kendisi burada **yok**: `navigator`, `WebGLRenderingContext` ve
 * `MediaCapabilities` çağrıları `hooks/useTeshis.ts` içinde. Ayrım
 * `memory/esik.rs` ile `memory/olcum.rs` arasındaki ayrımın aynısı — dizeyi
 * yorumlayan kod tarayıcı olmadan da test edilebilmeli, çünkü yanlış
 * yorumlanan bir sürücü dizesi kullanıcıya "GPU kapalı" diye yalan söyler.
 */

/** Bir codec'in bu makinedeki hâli. */
export type CodecDurumu = "donanim" | "yazilim" | "yok";

/** [`codecDurumu`] girdisi — `MediaCapabilities.decodingInfo` cevabının özü. */
export interface CozmeCevabi {
  supported: boolean;
  powerEfficient: boolean;
}

/**
 * WebView2 Runtime sürümü, kullanıcı aracısı dizesinden.
 *
 * **Neden bu kadar önemli:** `docs/olcumler/README.md` her raporun başında bu
 * sürümü şart koşuyor. WebView2 Runtime kendi kendine güncelleniyor ve iki
 * ölçüm arasındaki farkın sebebi bizim kodumuz değil, o güncelleme olabilir.
 * Kullanıcıdan bunu Ayarlar → Uygulamalar'dan bulmasını istemek yerine
 * tarayıcı kendi söylüyor.
 *
 * `Edg/` etiketi Chromium sürümünden (`Chrome/`) ayrı ve doğru olan bu:
 * WebView2'nin sürüm numarası Edge'in numarası.
 */
export function webview2Surumu(ua: string): string | null {
  const eslesme = /Edg(?:e|A|iOS)?\/(\d+(?:\.\d+)*)/.exec(ua);
  return eslesme ? eslesme[1] : null;
}

/**
 * WebGL oluşturucu dizesi **yazılımsal** bir arka uca mı işaret ediyor.
 *
 * Bu satır `docs/Medya.md` tablosunun ilk satırının teşhisi: "kare atlıyor,
 * CPU %100" belirtisinin gerçek sebebi genelde GPU hızlandırmanın düşmesi.
 * Kullanıcı sürücü sorununu bilmiyorsa tarayıcıyı suçluyor.
 *
 * Liste **beyaz değil kara** ve bu bilinçli bir istisna: donanım adları sonsuz
 * (her GPU modeli ayrı bir dize), yazılım oluşturucular ise sayılı ve adları
 * yıllardır aynı. Tanımadığı dizeye "donanım" demek, bilmediği için yanlış
 * alarm vermekten iyi.
 */
export function yazilimOlusturucu(renderer: string): boolean {
  // **CLAUDE.md #10'un istisnası ve sebebi tam tersi yönde.** Türkçe küçültme
  // kullanıcının yazdığı metin için doğru; burada dize sürücünün ürettiği bir
  // ASCII tanımlayıcı ve Türkçe kural onu bozuyor: "SWIFTSHADER" Türkçe
  // küçültmede "swıftshader" oluyor (noktasız ı) ve aradığımız ada hiç
  // uymuyor. Aşağıdaki test bunu sabitliyor.
  const d = renderer.toLowerCase();
  return (
    d.includes("swiftshader") ||
    d.includes("llvmpipe") ||
    d.includes("softwarerasterizer") ||
    d.includes("software rasterizer") ||
    d.includes("basic render") ||
    d.includes("microsoft basic display")
  );
}

/**
 * `decodingInfo` cevabını üç durumdan birine indirger.
 *
 * `powerEfficient` tarayıcının "bunu donanım çözüyor" dediği yer ve
 * `docs/Medya.md` içindeki codec tablosunun "doğrulanacak" sütununun cevabı.
 * Desteklenen ama verimsiz bir codec **yazılımla** çözülüyor: açılıyor ama
 * pilden ve CPU'dan yiyor, yani kullanıcının "kasıyor" dediği hâl.
 *
 * Cevap alınamadıysa (`null`) "yok" **denmiyor** — bilinmeyeni olumsuz
 * saymak, olmayan bir arızayı rapor etmek olurdu; çağıran taraf bunu ayrı
 * gösteriyor.
 */
export function codecDurumu(cevap: CozmeCevabi | null): CodecDurumu | null {
  if (!cevap) return null;
  if (!cevap.supported) return "yok";
  return cevap.powerEfficient ? "donanim" : "yazilim";
}

/** Raporun başlık tablosuna giren, tarayıcının bilebildiği alanlar. */
export interface RaporGirdisi {
  tarih: string;
  cekirdek: number | null;
  /** Sistem RAM'i, MB — backend ölçüyor (`memory/olcum.rs`). */
  sistemToplamMb: number | null;
  platform: string;
  gpu: string | null;
  webview2: string | null;
  muiren: string | null;
  profil: string;
  surecPolitikasi: string;
}

/** Tarayıcının bilemediği alanlar için kullanılan işaret. */
const ELLE = "? — elle doldurun";

/**
 * `docs/olcumler/README.md` başlık tablosunu üretir (Markdown).
 *
 * Protokolün tek katı kuralı şu: **sürüm alanları boş bırakılmıyor.** Bu
 * fonksiyon bilinebilecek her alanı dolduruyor, bilinemeyeni (işlemci modeli,
 * ağ, karşılaştırılan tarayıcı) `?` ile ve "elle doldurun" notuyla bırakıyor.
 * Boş bırakmakla `?` yazmak arasındaki fark, raporu okuyanın eksiği görmesi.
 *
 * Türkçe ek üretilmiyor (CLAUDE.md #11): satırlar ada ek gerektirmeyen
 * kalıplarda.
 */
export function raporBasligi(g: RaporGirdisi): string {
  const makine = [
    g.cekirdek ? `${g.cekirdek} mantıksal çekirdek` : null,
    g.sistemToplamMb ? `${Math.round(g.sistemToplamMb / 1024)} GB RAM` : null,
    g.platform || null,
    "işlemci modeli: ?",
  ]
    .filter(Boolean)
    .join(" · ");

  const satirlar: [string, string][] = [
    ["Tarih", g.tarih],
    ["Makine", makine],
    ["GPU", g.gpu ?? ELLE],
    ["WebView2 Runtime", g.webview2 ?? ELLE],
    ["Muiren", g.muiren ? `${g.muiren} · derleme: ?` : ELLE],
    ["Bellek profili", g.profil],
    ["Süreç politikası", g.surecPolitikasi],
    ["Ağ", ELLE],
    ["Karşılaştırılan", ELLE],
  ];

  return [
    "| Alan | Değer |",
    "|---|---|",
    ...satirlar.map(([ad, deger]) => `| ${ad} | ${deger} |`),
  ].join("\n");
}
