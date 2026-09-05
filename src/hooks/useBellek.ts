/**
 * Bellek özetinin arayüzdeki yansıması.
 *
 * Her kanca aynı iskelet (`docs/Frontend.md`): ilk değeri komutla çek, sonra
 * olayla güncelle, `unlisten` ile temizle.
 *
 * İlk değer `null` olabiliyor — gözcü henüz ilk turunu koşmamış demektir.
 * Panel o durumda "ölçülüyor" diyor; sıfır göstermek yanlış olurdu, çünkü
 * sıfır bir ölçüm sonucu değil ölçümün yokluğu.
 */

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { bellekOzeti } from "../ipc";
import { OLAY, type BellekOzeti } from "../ipc/tipler";

export function useBellek(): BellekOzeti | null {
  const [ozet, ayarla] = useState<BellekOzeti | null>(null);

  useEffect(() => {
    let canli = true;

    void bellekOzeti()
      .then((o) => {
        if (canli && o) ayarla(o);
      })
      .catch(() => {
        /* sürücü henüz hazır değil; ilk olay düzeltiyor */
      });

    const soz = listen<BellekOzeti>(OLAY.bellekOzeti, (e) => {
      if (canli) ayarla(e.payload);
    });

    return () => {
      canli = false;
      void soz.then((kapat) => kapat());
    };
  }, []);

  return ozet;
}
