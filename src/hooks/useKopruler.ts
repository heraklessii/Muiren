/**
 * Kardeş uygulamaların durumu.
 *
 * `docs/Kopruler.md`, genel kural: **köprü isteğe bağlı.** Kardeş kurulu
 * değilse arayüz o menü öğesini **hiç çizmiyor** — hata vermiyor, indirme
 * önerisi zorlamıyor. Bu hook o kararın verisini taşıyor.
 *
 * Kurulum araması backend'de önbellekli (`bridge::Bulunan`); buradan gelen
 * her çağrı diske inmiyor.
 */

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { kopruDurumu, kopruTazele } from "../ipc";
import { OLAY, type KopruDurumu } from "../ipc/tipler";

export interface Kopruler {
  hepsi: KopruDurumu[];
  /** Adına göre kurulu mu. Menü öğeleri bunu soruyor. */
  kurulu: (ad: string) => boolean;
  /** Kullanıcı Muiren açıkken bir kardeş kurmuş olabiliyor. */
  tazele: () => void;
}

export function useKopruler(): Kopruler {
  const [hepsi, ayarla] = useState<KopruDurumu[]>([]);

  const tazele = useCallback(() => {
    void kopruTazele().then(ayarla).catch(() => {
      /* sürücü hazır değil; liste boş kalıyor ve menüler çizilmiyor */
    });
  }, []);

  useEffect(() => {
    void kopruDurumu().then(ayarla).catch(() => {});
    const soz = listen<KopruDurumu[]>(OLAY.kopruDurumu, (e) => ayarla(e.payload));
    return () => {
      void soz.then((kapat) => kapat());
    };
  }, []);

  return {
    hepsi,
    kurulu: (ad) => hepsi.some((k) => k.ad === ad && k.kurulu),
    tazele,
  };
}
