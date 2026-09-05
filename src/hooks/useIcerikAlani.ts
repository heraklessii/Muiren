/**
 * Kabuğun altında kalan alanı backend'e bildiriyor.
 *
 * Sekme webview'i ayrı bir native pencere: CSS onu yerleştirmiyor, koordinat
 * gerekiyor. Ölçüyü arayüz biliyor (kabuk yüksekliği, yan panelin açık olup
 * olmadığı, tam ekran) ve backend'e **veri** olarak gönderiyor — bu bir karar
 * değil, bir ölçüm, dolayısıyla CLAUDE.md #2'ye aykırı değil.
 *
 * Backend sabit bir yükseklik tutsaydı yan panel açıldığında sayfa panelin
 * altında kalırdı.
 */

import { useEffect, type RefObject } from "react";

import { icerikAlani } from "../ipc";

export function useIcerikAlani(hedef: RefObject<HTMLElement | null>): void {
  useEffect(() => {
    const el = hedef.current;
    if (!el) return;

    let sonuncu = "";
    const bildir = () => {
      const k = el.getBoundingClientRect();
      const alan = {
        x: Math.round(k.left),
        y: Math.round(k.top),
        genislik: Math.round(k.width),
        yukseklik: Math.round(k.height),
      };
      // Aynı dikdörtgeni tekrar göndermek her karede bir IPC turu demek;
      // pencere sürüklenirken bu fark ediliyor.
      const imza = `${alan.x},${alan.y},${alan.genislik},${alan.yukseklik}`;
      if (imza === sonuncu) return;
      sonuncu = imza;
      void icerikAlani(alan).catch(() => {
        /* sürücü henüz hazır değil; bir sonraki ölçüm düzeltiyor */
      });
    };

    bildir();
    const gozlemci = new ResizeObserver(bildir);
    gozlemci.observe(el);
    window.addEventListener("resize", bildir);

    return () => {
      gozlemci.disconnect();
      window.removeEventListener("resize", bildir);
    };
  }, [hedef]);
}
