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

import { useRef, useState } from "react";

import { BellekPaneli } from "./BellekPaneli";
import { GecmisPaneli } from "./GecmisPaneli";
import { GrupPaneli } from "./GrupPaneli";
import { Kapat, Klasor, Liste, Sabit, Saat, Ses, Sessiz, Yonga } from "./Ikonlar";
import { SekmeIkonu } from "./SekmeIkonu";
import { useFavicon } from "../hooks/useFavicon";
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

/** Panel bölümleri — sıra, etiket ve ikon tek yerde. */
const BOLUMLER = [
  { ad: "sekmeler", etiket: "Sekmeler", ikon: Liste },
  { ad: "bellek", etiket: "Bellek", ikon: Yonga },
  { ad: "gruplar", etiket: "Gruplar", ikon: Klasor },
  { ad: "gecmis", etiket: "Geçmiş", ikon: Saat },
] as const satisfies ReadonlyArray<{
  ad: PanelSekmesi;
  etiket: string;
  ikon: (o: { boyut?: number }) => React.ReactElement;
}>;

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
  // Dolaşan odak — yatay şeritle aynı gerekçe (`SekmeSeridi`): 50 satırlık
  // bir listede her satıra `tabIndex=0` vermek, panelden çıkmak için 50 kez
  // Tab'a basmak demek. Ok tuşları burada da **etkinleştirmiyor**, yalnız
  // odağı taşıyor: uyuyan sekmenin üstünden geçmek onu uyandırmamalı
  // (CLAUDE.md #5).
  const [odakId, ayarlaOdakId] = useState<SekmeId | null>(null);
  const ogeler = useRef(new Map<SekmeId, HTMLDivElement | null>());
  // Aynı önbellek yatay şeritle paylaşılıyor (modül düzeyinde): dikey liste
  // açıldığında ikonlar yeniden okunmuyor (hooks/useFavicon.ts).
  const favicon = useFavicon();

  const odakHedefi =
    sekmeler.find((s) => s.id === odakId)?.id ??
    sekmeler.find((s) => s.etkin)?.id ??
    sekmeler[0]?.id ??
    null;

  const odagiTasi = (adim: number | "bas" | "son") => {
    if (sekmeler.length === 0) return;
    const simdiki = sekmeler.findIndex((s) => s.id === odakHedefi);
    const yeni =
      adim === "bas"
        ? 0
        : adim === "son"
          ? sekmeler.length - 1
          : Math.min(Math.max(simdiki + adim, 0), sekmeler.length - 1);
    const id = sekmeler[yeni].id;
    ayarlaOdakId(id);
    ogeler.current.get(id)?.focus();
  };

  if (sekmeler.length === 0) {
    return <p className="yan__bos">Açık sekme yok.</p>;
  }

  return (
    <ul className="dikey" role="tablist" aria-label="Sekmeler (dikey)">
      {sekmeler.map((s) => (
        <li key={s.id}>
          <div
            role="tab"
            ref={(el) => {
              ogeler.current.set(s.id, el);
            }}
            tabIndex={s.id === odakHedefi ? 0 : -1}
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
            onFocus={() => ayarlaOdakId(s.id)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onEtkinlestir(s.id);
              } else if (e.key === "ArrowDown") {
                e.preventDefault();
                odagiTasi(1);
              } else if (e.key === "ArrowUp") {
                e.preventDefault();
                odagiTasi(-1);
              } else if (e.key === "Home") {
                e.preventDefault();
                odagiTasi("bas");
              } else if (e.key === "End") {
                e.preventDefault();
                odagiTasi("son");
              } else if (e.key === "Delete" && !s.sabit) {
                e.preventDefault();
                onKapatSekme(s.id);
              }
            }}
            title={`${s.gorunenAd || "Yeni sekme"}\n${s.url}`}
          >
            {s.sabit && (
              <span className="dikey__sabit" aria-label="Sabitlenmiş">
                <Sabit boyut={11} />
              </span>
            )}
            <SekmeIkonu sekme={s} adres={favicon(s.favicon)} boyut={16} />
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
          {BOLUMLER.map(({ ad, etiket, ikon: Ikon }) => (
            <button
              key={ad}
              type="button"
              role="tab"
              aria-selected={sekme === ad}
              // Etiket yalnız etkin bölümde yazılıyor; diğerleri ikon.
              // Dördü birden yazıldığında 300 piksellik panele sığmıyordu ve
              // sonuncusu ("Geçmiş") kenardan taşıp yarım görünüyordu —
              // yani panelin en az bulunan bölümü, bulunamayan bölümdü.
              title={etiket}
              className={`yan__sekme ${sekme === ad ? "yan__sekme--etkin" : ""}`}
              onClick={() => onSekme(ad)}
            >
              <Ikon boyut={13} />
              <span className={sekme === ad ? "" : "gorunmez"}>{etiket}</span>
            </button>
          ))}
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
