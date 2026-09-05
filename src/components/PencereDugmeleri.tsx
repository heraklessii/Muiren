/**
 * Pencere düğmeleri — küçült / büyüt / kapat.
 *
 * Pencere **kenarlıksız** (`docs/Frontend.md`): başlık çubuğunun işini sekme
 * şeridi görüyor ve sistem düğmeleri burada yeniden çiziliyor. Sebep yer:
 * ayrı bir başlık çubuğu satırı, ekranın tepesinde sayfaya ait olması gereken
 * 32 pikseli alıyor.
 *
 * Kenarlıksız pencerenin bedeli, Windows davranışını **elle** doğru yapmak
 * zorunda olmak. Bu bileşen üç düğmeyi veriyor; sürükleme ve çift tıkla
 * büyütme `data-tauri-drag-region` ile şeridin kendisinde (Tauri kenara
 * yaslamayı ve Aero Snap'i o yoldan sürdürüyor).
 *
 * Komut yok, doğrudan pencere API'si: bunlar backend kararı değil, pencere
 * yönetimi. İzinler `capabilities/varsayilan.json` içinde ve **yalnız kabuk
 * webview'ine** verilmiş — açılan sayfaların pencereyi kapatma yetkisi yok.
 *
 * ## Neden her çağrı korumalı
 *
 * `getCurrentWindow()` Tauri bağlamı yokken **fırlatıyor**, ve React'te
 * render sırasında fırlayan bir bileşen bütün ağacı düşürüyor: kabuk tamamen
 * boş kalıyor. Yani üç pencere düğmesi yüzünden tarayıcının tamamı
 * kullanılamaz hâle geliyor. Bu bileşen o yüzden hiçbir koşulda fırlatmıyor;
 * tutamağı alamazsa düğmeleri **çizmiyor** ve kabuğun geri kalanı çalışmaya
 * devam ediyor.
 *
 * Bu aynı zamanda kabuğu düz bir tarayıcıda (vite dev sunucusu) açıp
 * incelemeyi mümkün kılıyor — arayüz hatalarını ayıklamanın en ucuz yolu.
 */

import { useEffect, useState } from "react";
import { getCurrentWindow, type Window } from "@tauri-apps/api/window";

import { PencereBuyut, PencereKapat, PencereKucult } from "./Ikonlar";

/**
 * Tauri penceresi; bağlam yoksa `null`. **Fırlatmıyor.**
 *
 * İçe aktarımın kendisi güvenli — modül yüklenirken Tauri iç durumuna
 * dokunulmuyor. Fırlatan `getCurrentWindow()` çağrısı ve o da burada
 * yakalanıyor.
 */
function pencereAl(): Window | null {
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

export function PencereDugmeleri() {
  const [pencere, ayarlaPencere] = useState<Window | null>(null);
  const [buyuk, ayarlaBuyuk] = useState(false);

  useEffect(() => {
    const p = pencereAl();
    ayarlaPencere(p);
    if (!p) return;

    let canli = true;
    void p.isMaximized().then((d) => {
      if (canli) ayarlaBuyuk(d);
    }).catch(() => {});

    // Simge şeklini pencerenin gerçek durumundan alıyoruz: kullanıcı kenara
    // yaslayarak da büyütebiliyor ve o yol bu bileşenden geçmiyor.
    const soz = p
      .onResized(() => {
        void p.isMaximized().then((d) => {
          if (canli) ayarlaBuyuk(d);
        }).catch(() => {});
      })
      .catch(() => () => {});

    return () => {
      canli = false;
      void soz.then((kapat) => kapat());
    };
  }, []);

  // Tutamak yoksa düğmeler hiç çizilmiyor. Tıklandığında hiçbir şey yapmayan
  // bir kapatma düğmesi, olmayan düğmeden kötü.
  if (!pencere) return null;

  return (
    <div className="pencere">
      <button
        type="button"
        className="pencere__dugme"
        onClick={() => void pencere.minimize().catch(() => {})}
        aria-label="Küçült"
        title="Küçült"
      >
        <PencereKucult boyut={16} />
      </button>
      <button
        type="button"
        className="pencere__dugme"
        onClick={() => void pencere.toggleMaximize().catch(() => {})}
        aria-label={buyuk ? "Önceki boyut" : "Büyüt"}
        title={buyuk ? "Önceki boyut" : "Büyüt"}
      >
        <PencereBuyut boyut={16} />
      </button>
      <button
        type="button"
        className="pencere__dugme pencere__dugme--kapat"
        onClick={() => void pencere.close().catch(() => {})}
        aria-label="Kapat"
        title="Kapat"
      >
        <PencereKapat boyut={16} />
      </button>
    </div>
  );
}
