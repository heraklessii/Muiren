/**
 * `muiren://yeni` — kabuğun kendi sayfası.
 *
 * Ağ isteği yok, reklam yok, öneri akışı yok (`docs/Sekmeler.md`). Sayfanın
 * webview'i de **yok**: kabuk çiziyor, dolayısıyla yeni sekmenin render
 * maliyeti sıfır. "Boş tarayıcı < 150 MB" hedefi buna dayanıyor
 * (`docs/Bellek.md`).
 */

import { useState } from "react";

import { Ara } from "./Ikonlar";

interface Ozellik {
  onGezin: (girdi: string) => void;
  /** Motor derlemede yoksa arama kutusu ölü düğme olurdu; onun yerine sebebi
   *  yazılıyor. */
  motorVar: boolean;
}

export function YeniSekme({ onGezin, motorVar }: Ozellik) {
  const [metin, ayarla] = useState("");

  return (
    <div className="yeni">
      <div className="yeni__marka">
        <svg viewBox="0 0 32 32" width="52" height="52" aria-hidden>
          <circle
            cx="16"
            cy="16"
            r="10.6"
            fill="none"
            stroke="var(--accent)"
            strokeWidth="2.4"
          />
          <path
            d="M16 24.4 19.2 16 12.8 16z"
            fill="none"
            stroke="var(--accent)"
            strokeWidth="2"
            strokeLinejoin="round"
          />
          <path d="M16 7.6 19.2 16 12.8 16z" fill="var(--accent)" />
        </svg>
        <h1>Muiren</h1>
      </div>

      {motorVar ? (
        <form
          className="yeni__form"
          onSubmit={(e) => {
            e.preventDefault();
            if (metin.trim() !== "") onGezin(metin);
          }}
        >
          <span className="yeni__ikon">
            <Ara />
          </span>
          <input
            className="yeni__girdi"
            value={metin}
            autoFocus
            spellCheck={false}
            autoComplete="off"
            placeholder="Adres ya da arama"
            onChange={(e) => ayarla(e.target.value)}
          />
        </form>
      ) : (
        <p className="yeni__not">
          Bu derlemede sayfa motoru yok (<code>--no-default-features</code>).
          Sekme yönetimi ve oturum çalışıyor, sayfa açılmıyor.
        </p>
      )}

      <p className="yeni__alt">
        Boştaki sekmeler uyutuluyor ve atılıyor. Sekme çubuğundaki yerleri,
        başlıkları ve adresleri duruyor.
      </p>
    </div>
  );
}
