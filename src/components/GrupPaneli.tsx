/**
 * Grup paneli — yan panelin dördüncü sekmesi (`docs/Roadmap.md` Faz 5).
 *
 * Üç iş bir arada, çünkü üçü aynı veri yapısının üç yüzü:
 *
 * - **Gruplama** — sekmeyi bir gruba almak.
 * - **Katlama** — grubu şeritte tek satıra indirmek.
 * - **Grup bazlı uyku eşiği** — projenin tezine bağlanan kısım. Katlanmış
 *   bir grup, kullanıcının "buraya şimdilik bakmıyorum" demesi; o sekmelerin
 *   genel eşikte beklemesi için bir sebep yok (`tabs/grup.rs`).
 *
 * Burada **karar yok**: her düğme bir komut çağırıyor, sıralamayı ve etkin
 * eşiği backend hesaplıyor. Eşik kutusu bile düzeltilmiş değeri gösteriyor —
 * `grup_guncelle` sonrası liste `sekme-degisti` ile tazeleniyor
 * (CLAUDE.md #13 ile aynı kalıp).
 */

import { useState } from "react";

import { Kapat, Klasor } from "./Ikonlar";
import { sure } from "../lib/bicim";
import type { Grup, GrupId, GrupRengi, SekmeId, SekmeOzeti } from "../ipc/tipler";

/** Seçilebilir renkler. Liste `tabs::GrupRengi` ile birebir. */
const RENKLER: GrupRengi[] = [
  "teal",
  "mor",
  "kehribar",
  "kirmizi",
  "yesil",
  "mavi",
  "gri",
];

interface Ozellik {
  gruplar: Grup[];
  sekmeler: SekmeOzeti[];
  /** Şu an bakılan sekme; "bu sekmeyi ekle" onu hedefliyor. */
  etkinId: SekmeId | null;
  onAc: (ad: string) => void;
  onSil: (id: GrupId) => void;
  onGuncelle: (
    id: GrupId,
    degisiklik: { ad?: string; renk?: GrupRengi; katli?: boolean; uykuEsigiSn?: number | null },
  ) => void;
  onAta: (sekme: SekmeId, grup: GrupId | null) => void;
}

export function GrupPaneli({
  gruplar,
  sekmeler,
  etkinId,
  onAc,
  onSil,
  onGuncelle,
  onAta,
}: Ozellik) {
  const [yeniAd, ayarlaYeniAd] = useState("");
  const etkin = sekmeler.find((s) => s.id === etkinId) ?? null;

  return (
    <div className="grupsel">
      <form
        className="grupsel__yeni"
        onSubmit={(e) => {
          e.preventDefault();
          // Boş ad kabul ediliyor: backend onu boş bırakıyor ve arayüz
          // "Adsız grup" yazıyor. Kullanıcıyı ad bulmaya zorlamak, grubu hiç
          // açmamasına yol açıyor.
          onAc(yeniAd);
          ayarlaYeniAd("");
        }}
      >
        <input
          type="text"
          value={yeniAd}
          maxLength={40}
          placeholder="Yeni grup adı"
          onChange={(e) => ayarlaYeniAd(e.target.value)}
        />
        <button type="submit" className="dugme">
          <Klasor boyut={13} /> Ekle
        </button>
      </form>

      {gruplar.length === 0 && (
        <p className="yan__bos">
          Grup yok. Bir grup açıp sekmeleri içine alabilirsiniz; katlanan
          grubun sekmeleri şeritte gizleniyor ve isterseniz daha erken
          uyuyorlar.
        </p>
      )}

      <ul className="grupsel__liste">
        {gruplar.map((g) => {
          const uyeler = sekmeler.filter((s) => s.grup === g.id);
          return (
            <li key={g.id} className={`grupsel__oge renk--${g.renk}`}>
              <div className="grupsel__ust">
                <span className="grupsel__renkler">
                  {RENKLER.map((r) => (
                    <button
                      key={r}
                      type="button"
                      className={`grupsel__renk renk--${r}${g.renk === r ? " grupsel__renk--etkin" : ""}`}
                      aria-label={`Renk: ${r}`}
                      onClick={() => onGuncelle(g.id, { renk: r })}
                    />
                  ))}
                </span>
                <button
                  type="button"
                  className="dugme dugme--ikon"
                  aria-label="Grubu sil"
                  title="Grubu sil — sekmeler kalıyor, yalnız gruptan çıkıyorlar"
                  onClick={() => onSil(g.id)}
                >
                  <Kapat boyut={12} />
                </button>
              </div>

              <input
                className="grupsel__ad"
                type="text"
                value={g.ad}
                maxLength={40}
                placeholder="Adsız grup"
                onChange={(e) => onGuncelle(g.id, { ad: e.target.value })}
              />

              <label className="grupsel__satir">
                <input
                  type="checkbox"
                  checked={g.katli}
                  onChange={(e) => onGuncelle(g.id, { katli: e.target.checked })}
                />
                Katlı ({uyeler.length})
              </label>

              <label className="grupsel__satir">
                <input
                  type="checkbox"
                  checked={g.uykuEsigiSn !== null}
                  onChange={(e) =>
                    // Kapatınca `null`: "genel ayarı kullan". Bir sayıya
                    // eşitlemek, kullanıcı genel eşiği değiştirdiğinde
                    // grubun eski değerde kalması demek olurdu.
                    onGuncelle(g.id, { uykuEsigiSn: e.target.checked ? 300 : null })
                  }
                />
                Kendi uyku eşiği
              </label>

              {g.uykuEsigiSn !== null && (
                <label className="grupsel__satir">
                  <input
                    type="number"
                    min={1}
                    value={Math.round(g.uykuEsigiSn / 60)}
                    onChange={(e) =>
                      onGuncelle(g.id, {
                        uykuEsigiSn: Math.max(1, Number(e.target.value)) * 60,
                      })
                    }
                  />
                  dakika · şu an {sure(g.uykuEsigiSn)}
                </label>
              )}

              {/* Etkin sekme bu grupta değilse eklenebiliyor. Sürükle-bırak
                  ile gruplama YOK: şeritteki sürükleme zaten sırayı
                  taşıyor ve ikisini aynı jeste yüklemek, kullanıcının
                  yanlışlıkla grup değiştirmesi demek. */}
              {etkin && etkin.grup !== g.id && (
                <button
                  type="button"
                  className="dugme grupsel__ekle"
                  onClick={() => onAta(etkin.id, g.id)}
                >
                  Bu sekmeyi ekle
                </button>
              )}
              {etkin && etkin.grup === g.id && (
                <button
                  type="button"
                  className="dugme grupsel__ekle"
                  onClick={() => onAta(etkin.id, null)}
                >
                  Bu sekmeyi çıkar
                </button>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
