/**
 * Köprü menüsü — kardeş uygulamalara devir.
 *
 * `docs/Kopruler.md`, genel kural: **köprü isteğe bağlı.** Kurulu kardeş
 * yoksa bu düğme hiç çizilmiyor; kurulu olan varsa yalnız onun öğesi
 * görünüyor. Kurulu olmayan bir uygulamanın soluk bir öğesini göstermek,
 * kullanıcıya tıklanamayan bir düğme sunmak olurdu ve
 * `Yetenekler`/`docs/IPC.md` bunu açıkça reddediyor: "sessizce tıklanan ölü
 * düğme, kullanıcıya 'bu program bozuk' dedirtiyor."
 *
 * ## Muiwatch oda kimliği neden burada soruluyor
 *
 * Muiwatch tarafında bugün bir oda kapısı yok (`bridge/muiwatch.rs`): Faz 3
 * başlanmamış ve tek-örnek eklentisi bulunmuyor. Yine de kimlik soruluyor ve
 * geçiriliyor, çünkü Muiren tarafındaki asıl iş kimliği göndermek değil —
 * **sekmeyi korumaya almak** (koruma kuralı #7). O kayıt oda kimliğine
 * bağlı ve bugün de çalışıyor.
 */

import { useEffect, useRef, useState } from "react";

import { Klasor } from "./Ikonlar";
import type { SekmeOzeti } from "../ipc/tipler";

interface Ozellik {
  sekme: SekmeOzeti | undefined;
  muiplyVar: boolean;
  muiwatchVar: boolean;
  onMuiply: (yol: string) => void;
  onMuiwatch: (oda: string) => void;
}

/**
 * Adres yerel bir dosyayı mı gösteriyor.
 *
 * Medya uzantısı kontrolü **burada yok**: o karar `bridge::muiply::yerel_dosya`
 * içinde, saf ve testli. Arayüz yalnız "yerel dosya mı" diye bakıyor; medya
 * olmayan bir dosya gönderilirse backend anlaşılır bir hata dönüyor. İki
 * yerde iki liste tutmak, birinin sessizce eskimesi demek.
 */
function yerelDosya(url: string): boolean {
  return url.startsWith("file://");
}

export function KopruMenusu({
  sekme,
  muiplyVar,
  muiwatchVar,
  onMuiply,
  onMuiwatch,
}: Ozellik) {
  const [acik, ayarlaAcik] = useState(false);
  const [oda, ayarlaOda] = useState("");
  const kokRef = useRef<HTMLDivElement>(null);

  // Dışarı tıklayınca kapan. `mousedown` çünkü `click` menüdeki düğmeye
  // basıldığında da tetikleniyor ve menü kapanıp eylem kaybolurdu.
  useEffect(() => {
    if (!acik) return;
    const dinle = (e: MouseEvent) => {
      if (!kokRef.current?.contains(e.target as Node)) ayarlaAcik(false);
    };
    window.addEventListener("mousedown", dinle);
    return () => window.removeEventListener("mousedown", dinle);
  }, [acik]);

  // Hiçbir kardeş kurulu değilse düğme hiç yok.
  if (!muiplyVar && !muiwatchVar) return null;

  const muiplyUygun = muiplyVar && sekme !== undefined && yerelDosya(sekme.url);

  return (
    <div className="kopru" ref={kokRef}>
      <button
        type="button"
        className={`dugme dugme--ikon ${acik ? "dugme--acik" : ""}`}
        aria-expanded={acik}
        title="Mui ailesine devret"
        onClick={() => ayarlaAcik((a) => !a)}
      >
        <Klasor boyut={14} />
        <span className="gorunmez">Köprüler</span>
      </button>

      {acik && (
        <div className="kopru__menu" role="menu">
          {muiplyVar && (
            <button
              type="button"
              role="menuitem"
              className="kopru__oge"
              disabled={!muiplyUygun}
              title={
                muiplyUygun
                  ? "Dosyayı Muiply'da aç"
                  : "Yalnız yerel dosyalar devrediliyor — Muiply ağ akışı oynatmıyor"
              }
              onClick={() => {
                if (sekme) onMuiply(sekme.url);
                ayarlaAcik(false);
              }}
            >
              <b>Muiply'da aç</b>
              <span>
                Muiren'in açamadığı codec'i (HEVC gibi) Muiply'ın libmpv'si
                açıyor. Yalnız yerel dosya.
              </span>
            </button>
          )}

          {muiwatchVar && (
            <form
              className="kopru__oge kopru__oge--form"
              onSubmit={(e) => {
                e.preventDefault();
                if (oda.trim() === "" || !sekme) return;
                onMuiwatch(oda.trim());
                ayarlaOda("");
                ayarlaAcik(false);
              }}
            >
              <b>Muiwatch ile birlikte izle</b>
              <span>
                Bağlanan sekme <b>uyutulmuyor ve atılmıyor</b>: uyuyan sekme
                senkron olayı alamaz ve bağlantı sessizce kopardı.
              </span>
              <span className="kopru__girdi">
                <input
                  type="text"
                  value={oda}
                  maxLength={64}
                  placeholder="oda kimliği"
                  spellCheck={false}
                  onChange={(e) => ayarlaOda(e.target.value)}
                />
                <button type="submit" className="dugme" disabled={!sekme}>
                  Bağla
                </button>
              </span>
            </form>
          )}
        </div>
      )}
    </div>
  );
}
