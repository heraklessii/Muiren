/**
 * Sekme arama — `Ctrl+Shift+A`.
 *
 * Açık sekmelerde başlık ve adres araması. **Uyuyan ve atılmış sekmeler
 * dahil** ve onları uyandırmadan (CLAUDE.md #5): kayıt zaten bellekte.
 *
 * 50 sekmede yatay şerit fizik olarak yetmiyor; dikey liste Faz 5'te geliyor,
 * o gelene kadar aradığını bulmanın yolu bu.
 */

import { useMemo, useState } from "react";

import { Ara, Uyku } from "./Ikonlar";
import { sirala, suz } from "../lib/suz";
import { sure } from "../lib/bicim";
import type { SekmeId, SekmeOzeti } from "../ipc/tipler";

interface Ozellik {
  sekmeler: SekmeOzeti[];
  onSec: (id: SekmeId) => void;
  onKapat: () => void;
}

export function SekmeArama({ sekmeler, onSec, onKapat }: Ozellik) {
  const [sorgu, ayarla] = useState("");
  const [imlec, ayarlaImlec] = useState(0);

  const sonuc = useMemo(() => sirala(suz(sekmeler, sorgu), sorgu), [sekmeler, sorgu]);
  const secili = Math.min(imlec, Math.max(sonuc.length - 1, 0));

  return (
    <div className="ortu" onMouseDown={onKapat}>
      <div className="arama" onMouseDown={(e) => e.stopPropagation()}>
        <div className="arama__satir">
          <Ara />
          <input
            className="arama__girdi"
            autoFocus
            value={sorgu}
            spellCheck={false}
            placeholder="Sekmelerde ara"
            onChange={(e) => {
              ayarla(e.target.value);
              ayarlaImlec(0);
            }}
            onKeyDown={(e) => {
              if (e.key === "ArrowDown") {
                e.preventDefault();
                ayarlaImlec((i) => Math.min(i + 1, sonuc.length - 1));
              } else if (e.key === "ArrowUp") {
                e.preventDefault();
                ayarlaImlec((i) => Math.max(i - 1, 0));
              } else if (e.key === "Enter") {
                const s = sonuc[secili];
                if (s) {
                  onSec(s.id);
                  onKapat();
                }
              } else if (e.key === "Escape") {
                onKapat();
              }
            }}
          />
        </div>

        <ul className="arama__liste">
          {sonuc.length === 0 && <li className="arama__bos">Eşleşen sekme yok</li>}
          {sonuc.map((s, i) => (
            <li key={s.id}>
              <button
                type="button"
                className={`arama__oge${i === secili ? " arama__oge--secili" : ""}`}
                onClick={() => {
                  onSec(s.id);
                  onKapat();
                }}
              >
                <span className="arama__ad">{s.gorunenAd || "Yeni sekme"}</span>
                <span className="arama__url">{s.url}</span>
                {(s.durum === "uyuyan" || s.durum === "atilmis") && (
                  <span className="rozet rozet--uyku" title={`${sure(s.bostaSn)} boşta`}>
                    <Uyku boyut={11} />
                    {s.durum === "uyuyan" ? "uykuda" : "atıldı"}
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
