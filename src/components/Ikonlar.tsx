/**
 * Satır içi ikonlar.
 *
 * Hepsi `currentColor` çiziyor ve 16×16 ızgarada duruyor; renk sabiti yok
 * (CLAUDE.md #9). Ayrı bir ikon paketi eklenmedi: on beş yol için bir bağımlılık
 * ve onun getirdiği paket boyutu, bellek iddiası olan bir programda pazarlık
 * konusu (`docs/Bellek.md`, "şişkinlik eklememek").
 */

type Ozellik = { boyut?: number };

const ortak = (boyut: number) => ({
  width: boyut,
  height: boyut,
  viewBox: "0 0 16 16",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.6,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  "aria-hidden": true,
});

export function Geri({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M10 3 5 8l5 5" />
    </svg>
  );
}

export function Ileri({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M6 3l5 5-5 5" />
    </svg>
  );
}

export function Yenile({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M13 8a5 5 0 1 1-1.6-3.7" />
      <path d="M13 2.5V5h-2.5" />
    </svg>
  );
}

export function Durdur({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
    </svg>
  );
}

export function Kapat({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
    </svg>
  );
}

export function Arti({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M8 3.5v9M3.5 8h9" />
    </svg>
  );
}

export function Ara({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <circle cx="7.2" cy="7.2" r="3.9" />
      <path d="M10.2 10.2 13.5 13.5" />
    </svg>
  );
}

export function Kilit({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <rect x="3.5" y="7" width="9" height="6" rx="1.4" />
      <path d="M5.8 7V5.4a2.2 2.2 0 0 1 4.4 0V7" />
    </svg>
  );
}

/** `http` uyarısı. Kilidin yokluğu yeterli değil; açıkça uyarılıyor. */
export function Uyari({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M8 2.8 14 13H2z" />
      <path d="M8 6.6v3M8 11.2v.1" />
    </svg>
  );
}

export function Ses({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M4 6.2h2.2L9 3.8v8.4L6.2 9.8H4z" />
      <path d="M11 6.4a2.6 2.6 0 0 1 0 3.2" />
    </svg>
  );
}

export function Sessiz({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M4 6.2h2.2L9 3.8v8.4L6.2 9.8H4z" />
      <path d="M11.2 6.4l2.4 3.2M13.6 6.4l-2.4 3.2" />
    </svg>
  );
}

/** Sabitlenmiş sekme — ailenin "tek dolu öğe" kuralına uyarak dolu. */
export function Sabit({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)} fill="currentColor" strokeWidth={0}>
      <path d="M9.6 1.8 14.2 6.4l-1.5 1.5-1-.3-2.4 2.4.3 2.3-1.3 1.3-2.6-2.6-3 3-.7-.7 3-3L2.4 7.7l1.3-1.3 2.3.3 2.4-2.4-.3-1z" />
    </svg>
  );
}

/** Uyuyan sekme rozeti. */
export function Uyku({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M12.6 9.6A5 5 0 0 1 6.4 3.4a5 5 0 1 0 6.2 6.2z" />
    </svg>
  );
}

/** Bellek paneli — yonga. */
export function Yonga({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <rect x="4.5" y="4.5" width="7" height="7" rx="1.4" />
      <path d="M6.5 2v2.5M9.5 2v2.5M6.5 11.5V14M9.5 11.5V14M2 6.5h2.5M2 9.5h2.5M11.5 6.5H14M11.5 9.5H14" />
    </svg>
  );
}

/** Yan panel — kenar çubuğu. */
export function Panel({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <rect x="2" y="3" width="12" height="10" rx="1.6" />
      <path d="M10 3v10" />
    </svg>
  );
}

/** Dikey sekme listesi. */
export function Liste({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M3 4.5h10M3 8h10M3 11.5h10" />
    </svg>
  );
}

/* --- pencere düğmeleri ---
 *
 * Bunlar 16×16 ızgarada ama Windows'un başlık çubuğu ölçüsünde çiziliyor:
 * çizgi kalınlığı 1, çünkü sistem düğmeleri ince ve kalın olan yamalı duruyor.
 */

export function PencereKucult({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)} strokeWidth={1}>
      <path d="M4 8h8" />
    </svg>
  );
}

export function PencereBuyut({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)} strokeWidth={1}>
      <rect x="4.5" y="4.5" width="7" height="7" rx="0.5" />
    </svg>
  );
}

export function PencereKapat({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)} strokeWidth={1}>
      <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
    </svg>
  );
}

/** Yer imi — yıldız. `dolu` true ise içi teal. */
export function Yildiz({ boyut = 16, dolu = false }: Ozellik & { dolu?: boolean }) {
  return (
    <svg {...ortak(boyut)} fill={dolu ? "currentColor" : "none"}>
      <path d="M8 2.5l1.7 3.5 3.8.5-2.8 2.7.7 3.8L8 11.2l-3.4 1.8.7-3.8L2.5 6.5l3.8-.5z" />
    </svg>
  );
}

/** Geçmiş — saat. */
export function Saat({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <circle cx="8" cy="8" r="5.8" />
      <path d="M8 4.6V8l2.4 1.6" />
    </svg>
  );
}

/** Ayarlar — dişli. */
export function Disli({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <circle cx="8" cy="8" r="2.2" />
      <path d="M8 1.6v1.7M8 12.7v1.7M14.4 8h-1.7M3.3 8H1.6M12.5 3.5l-1.2 1.2M4.7 11.3l-1.2 1.2M12.5 12.5l-1.2-1.2M4.7 4.7L3.5 3.5" />
    </svg>
  );
}

/** Gizli sekme — maske. Chrome'un şapkası değil; aile diline uygun düz form. */
export function Gizli({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M2 9h12" />
      <path d="M5.2 3.4h5.6l1.4 5.6H3.8z" />
      <circle cx="5.2" cy="11.4" r="2" />
      <circle cx="10.8" cy="11.4" r="2" />
    </svg>
  );
}

/** Engellenen istek/pop-up rozeti. */
export function Kalkan({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M8 1.8 13.2 3.6v4.2c0 3-2.2 5.4-5.2 6.4-3-1-5.2-3.4-5.2-6.4V3.6z" />
    </svg>
  );
}

/** Katlanmış grup — sağ ok. Açık grup için `dondur` ile 90° çevriliyor. */
export function Ok({ boyut = 16, dondur = false }: Ozellik & { dondur?: boolean }) {
  return (
    <svg
      {...ortak(boyut)}
      style={dondur ? { transform: "rotate(90deg)" } : undefined}
    >
      <path d="M6 3.5 10.5 8 6 12.5" />
    </svg>
  );
}

/** İndirme önerisi. */
export function Indir({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M8 2v8" />
      <path d="M4.8 7 8 10.2 11.2 7" />
      <path d="M3 13h10" />
    </svg>
  );
}

/** Klasör — grup işareti. */
export function Klasor({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M2 4.2h4l1.2 1.6H14v6.4a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1z" />
    </svg>
  );
}

/** Çöp — veri temizleme. */
export function Cop({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)}>
      <path d="M2.8 4.2h10.4" />
      <path d="M6.2 4.2V2.8h3.6v1.4" />
      <path d="M4.2 4.2 4.8 13a.9.9 0 0 0 .9.8h4.6a.9.9 0 0 0 .9-.8l.6-8.8" />
    </svg>
  );
}

/**
 * Pusula — Mui ailesinin Muiren glyph'i.
 *
 * Uygulama ikonuyla **aynı çizim** (`public/icons/muiren.svg`,
 * `src-tauri/icons/kaynak.svg`, `index.html` favicon'u): çember + iğne,
 * iğnenin yalnız kuzey yarısı dolu. Biri değişirse hepsi değişir
 * (CLAUDE.md, "Diğer Mui Projeleriyle Tutarlılık").
 *
 * Buradaki kopya 16'lık ızgaraya oturtulmuş hâli; kabuğun kendi sayfaları
 * sekme şeridinde bunu taşıyor.
 */
export function Pusula({ boyut = 16 }: Ozellik) {
  return (
    <svg {...ortak(boyut)} strokeWidth={1.3}>
      <circle cx="8" cy="8" r="5.3" />
      <path d="M8 12.2 9.6 8 6.4 8z" />
      <path d="M8 3.8 9.6 8 6.4 8z" fill="currentColor" stroke="none" />
    </svg>
  );
}
