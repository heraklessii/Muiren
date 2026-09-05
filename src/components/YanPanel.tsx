/**
 * Yan panel — sekmeli (`docs/Frontend.md`).
 *
 * Üç sekme: **dikey sekme listesi**, **bellek paneli** ve **geçmiş**.
 *
 * **Kapalıyken hiç çizilmiyor** (`App.tsx` bileşeni hiç oluşturmuyor): 50
 * sekmelik bir listeyi görünmeyen bir panelde tutmanın maliyeti, tam da
 * kaçındığımız türden. Panelin açılması içerik alanını daraltıyor ve
 * `useIcerikAlani` yeni dikdörtgeni backend'e bildiriyor — sekme webview'i
 * panelin altında kalmıyor.
 */

import { BellekPaneli } from "./BellekPaneli";
import { GecmisPaneli } from "./GecmisPaneli";
import { GrupPaneli } from "./GrupPaneli";
import { Kapat, Klasor, Liste, Sabit, Saat, Ses, Sessiz, Yonga } from "./Ikonlar";
import { sure } from "../lib/bicim";
import type {
  BellekOzeti,
  Grup,
  GrupId,
  GrupRengi,
  SekmeId,
  SekmeOzeti,
} from "../ipc/tipler";

export type PanelSekmesi = "sekmeler" | "bellek" | "gecmis" | "gruplar";

interface Ozellik {
  sekme: PanelSekmesi;
  onSekme: (s: PanelSekmesi) => void;
  onKapat: () => void;
  sekmeler: SekmeOzeti[];
  ozet: BellekOzeti | null;
  onEtkinlestir: (id: SekmeId) => void;
  onKapatSekme: (id: SekmeId) => void;
  onSessizeAl: (id: SekmeId, sessiz: boolean) => void;
  onHepsiniUyut: () => void;
  /** Geçmiş panelinden bir kayda tıklanınca. */
  onGezin: (url: string) => void;

  // --- gruplar (Faz 5) ---
  gruplar: Grup[];
  etkinId: SekmeId | null;
  onGrupAc: (ad: string) => void;
  onGrupSil: (id: GrupId) => void;
  onGrupGuncelle: (
    id: GrupId,
    degisiklik: {
      ad?: string;
      renk?: GrupRengi;
      katli?: boolean;
      uykuEsigiSn?: number | null;
    },
  ) => void;
  onGrupAta: (sekme: SekmeId, grup: GrupId | null) => void;
}

/**
 * Dikey sekme listesi.
 *
 * Yatay şeritten farkı daralmıyor olması: 50 sekmede yatay şerit 52 piksele
 * inip başlıkları yutuyor, dikey liste başlığı okunur tutuyor. Hedef kitle
 * onlarca sekme açtığı için bu liste bir süs değil, asıl kullanım biçimi
 * (`docs/Sekmeler.md`).
 */
function DikeySekmeler({
  sekmeler,
  onEtkinlestir,
  onKapatSekme,
  onSessizeAl,
}: Pick<Ozellik, "sekmeler" | "onEtkinlestir" | "onKapatSekme" | "onSessizeAl">) {
  if (sekmeler.length === 0) {
    return <p className="yan__bos">Açık sekme yok.</p>;
  }

  return (
    <ul className="dikey" role="tablist" aria-label="Sekmeler (dikey)">
      {sekmeler.map((s) => (
        <li key={s.id}>
          <div
            role="tab"
            tabIndex={0}
            aria-selected={s.etkin}
            className={[
              "dikey__oge",
              `dikey__oge--${s.durum}`,
              s.etkin ? "dikey__oge--etkin" : "",
            ]
              .filter(Boolean)
              .join(" ")}
            // Girinti ağaçtaki derinlikten; hesap backend'de (CLAUDE.md #2).
            style={{ "--derinlik": s.derinlik } as React.CSSProperties}
            onClick={() => onEtkinlestir(s.id)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onEtkinlestir(s.id);
              }
            }}
            title={`${s.gorunenAd || "Yeni sekme"}\n${s.url}`}
          >
            {s.sabit && (
              <span className="dikey__sabit" aria-label="Sabitlenmiş">
                <Sabit boyut={11} />
              </span>
            )}
            <span className="dikey__isaret" aria-hidden />
            {/* Başlık sayfadan geliyor: backend temizledi, React kaçışlıyor. */}
            <span className="dikey__ad">{s.gorunenAd || "Yeni sekme"}</span>
            <span className="dikey__bosta">{sure(s.bostaSn)}</span>

            {(s.sesCaliyor || s.sessiz) && (
              <button
                type="button"
                className="dikey__dugme"
                aria-label={s.sessiz ? "Sesi aç" : "Sesi kapat"}
                onClick={(e) => {
                  e.stopPropagation();
                  onSessizeAl(s.id, !s.sessiz);
                }}
              >
                {s.sessiz ? <Sessiz boyut={12} /> : <Ses boyut={12} />}
              </button>
            )}
            {!s.sabit && (
              <button
                type="button"
                className="dikey__dugme"
                aria-label="Sekmeyi kapat"
                onClick={(e) => {
                  e.stopPropagation();
                  onKapatSekme(s.id);
                }}
              >
                <Kapat boyut={11} />
              </button>
            )}
          </div>
        </li>
      ))}
    </ul>
  );
}

export function YanPanel({
  sekme,
  onSekme,
  onKapat,
  sekmeler,
  ozet,
  onEtkinlestir,
  onKapatSekme,
  onSessizeAl,
  onHepsiniUyut,
  onGezin,
  gruplar,
  etkinId,
  onGrupAc,
  onGrupSil,
  onGrupGuncelle,
  onGrupAta,
}: Ozellik) {
  return (
    <aside className="yan" aria-label="Yan panel">
      <div className="yan__baslik">
        <div className="yan__sekmeler" role="tablist" aria-label="Panel bölümleri">
          <button
            type="button"
            role="tab"
            aria-selected={sekme === "sekmeler"}
            className={`yan__sekme ${sekme === "sekmeler" ? "yan__sekme--etkin" : ""}`}
            onClick={() => onSekme("sekmeler")}
          >
            <Liste boyut={13} /> Sekmeler
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={sekme === "bellek"}
            className={`yan__sekme ${sekme === "bellek" ? "yan__sekme--etkin" : ""}`}
            onClick={() => onSekme("bellek")}
          >
            <Yonga boyut={13} /> Bellek
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={sekme === "gruplar"}
            className={`yan__sekme ${sekme === "gruplar" ? "yan__sekme--etkin" : ""}`}
            onClick={() => onSekme("gruplar")}
          >
            <Klasor boyut={13} /> Gruplar
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={sekme === "gecmis"}
            className={`yan__sekme ${sekme === "gecmis" ? "yan__sekme--etkin" : ""}`}
            onClick={() => onSekme("gecmis")}
          >
            <Saat boyut={13} /> Geçmiş
          </button>
        </div>
        <button
          type="button"
          className="dugme dugme--ikon"
          aria-label="Yan paneli kapat"
          onClick={onKapat}
        >
          <Kapat boyut={13} />
        </button>
      </div>

      <div className="yan__govde">
        {sekme === "sekmeler" && (
          <DikeySekmeler
            sekmeler={sekmeler}
            onEtkinlestir={onEtkinlestir}
            onKapatSekme={onKapatSekme}
            onSessizeAl={onSessizeAl}
          />
        )}
        {sekme === "bellek" && (
          <BellekPaneli
            ozet={ozet}
            sekmeler={sekmeler}
            onHepsiniUyut={onHepsiniUyut}
            onEtkinlestir={onEtkinlestir}
          />
        )}
        {sekme === "gruplar" && (
          <GrupPaneli
            gruplar={gruplar}
            sekmeler={sekmeler}
            etkinId={etkinId}
            onAc={onGrupAc}
            onSil={onGrupSil}
            onGuncelle={onGrupGuncelle}
            onAta={onGrupAta}
          />
        )}
        {sekme === "gecmis" && <GecmisPaneli onGezin={onGezin} />}
      </div>
    </aside>
  );
}
