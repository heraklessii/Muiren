/**
 * İndirme önerisi ve sonuç bildirimi.
 *
 * `docs/Kopruler.md`, karar #4: indirme Muiget'in işi. `IndirmePolitikasi::Sor`
 * altında motor indirmeyi iptal ediyor ve kullanıcıya soruluyor.
 *
 * **Modal değil.** İndirme sayfanın işini durdurmuyor; kullanıcıyı da
 * durdurmamalı. Şerit adres çubuğunun altında, sağda duruyor ve sayfanın
 * üstünü kaplamıyor.
 *
 * **Sessiz de değil.** Devretme hatası küçük bir bildirimle görünüyor:
 * köprü kırıldığında kullanıcı tarayıcıyı suçlamasın (`docs/Kopruler.md`).
 */

import { Indir, Kapat } from "./Ikonlar";
import { bayt } from "../lib/bicim";
import type { Indirme as IndirmeDurumu } from "../hooks/useIndirme";

interface Ozellik {
  durum: IndirmeDurumu;
  /** Kurulu değilse "Muiget'e gönder" düğmesi hiç çizilmiyor. */
  muigetVar: boolean;
}

export function Indirme({ durum, muigetVar }: Ozellik) {
  const { oneri, bildirim, muigeteGonder, muirenIndirsin, kapat } = durum;

  if (oneri) {
    return (
      <div className="indirme-kat">
        <div className="indirme" role="dialog" aria-label="İndirme">
          <Indir boyut={15} />
          {/* Ad sayfadan geliyor; backend temizledi
              (`bridge::dosya_adi_temizle`) ve React kaçışlıyor. */}
          <span className="indirme__ad">{oneri.dosyaAdi ?? oneri.url}</span>
          {oneri.boyut > 0 && (
            <span className="indirme__boyut">{bayt(oneri.boyut)}</span>
          )}
          <div className="indirme__eylem">
            {muigetVar && (
              <button
                type="button"
                className="dugme dugme--vurgu"
                onClick={muigeteGonder}
              >
                Muiget'e gönder
              </button>
            )}
            <button type="button" className="dugme" onClick={muirenIndirsin}>
              Muiren indirsin
            </button>
            <button
              type="button"
              className="dugme dugme--ikon"
              aria-label="Vazgeç"
              onClick={kapat}
            >
              <Kapat boyut={12} />
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (!bildirim) return null;

  return (
    <div className="indirme-kat">
      <div className="indirme" role="status">
        <Indir boyut={15} />
        <span className="indirme__ad">
          {bildirim.sonuc === "devredildi"
            ? `Muiget'e gönderildi: ${bildirim.dosyaAdi ?? bildirim.url}`
            : `Muiget'e gönderilemedi: ${bildirim.dosyaAdi ?? bildirim.url}`}
        </span>
      </div>
    </div>
  );
}
