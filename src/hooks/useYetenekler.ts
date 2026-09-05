/**
 * Motorun gerçekten neyi desteklediği.
 *
 * Arayüz buna bakıp yapamayacağı işin düğmesini çizmiyor: sessizce tıklanan
 * ölü düğme, kullanıcıya "bu program bozuk" dedirtiyor. Eksik yetenek bir hata
 * değil, eski bir WebView2 Runtime (`docs/Setup.md`).
 */

import { useEffect, useState } from "react";

import { yetenekler as yeteneklerOku } from "../ipc";
import type { Yetenekler } from "../ipc/tipler";

const BILINMIYOR: Yetenekler = {
  motor: false,
  askiyaAlma: false,
  bellekHedefi: false,
  surecBilgisi: false,
  indirmeOlayi: false,
  favicon: false,
};

export function useYetenekler(): Yetenekler {
  const [y, ayarla] = useState<Yetenekler>(BILINMIYOR);

  useEffect(() => {
    let canli = true;
    yeteneklerOku()
      .then((v) => {
        if (canli) ayarla(v);
      })
      .catch(() => ayarla(BILINMIYOR));
    return () => {
      canli = false;
    };
  }, []);

  return y;
}
