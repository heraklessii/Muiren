/**
 * Backend sekme durumunun yansıması.
 *
 * Her kanca aynı iskelet (`docs/Frontend.md`): ilk değeri komutla çek, sonra
 * olayla güncelle, `unlisten` ile temizle.
 *
 * **Burada karar yok.** Sıra, durum ve girinti backend'den geldiği gibi
 * çiziliyor; arayüz kapalıyken de doğru olmaları gerekiyor (CLAUDE.md #2).
 */

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { sekmeListesi } from "../ipc";
import { OLAY, type SekmeOzeti } from "../ipc/tipler";

export function useSekmeler(): SekmeOzeti[] {
  const [sekmeler, ayarla] = useState<SekmeOzeti[]>([]);

  useEffect(() => {
    let canli = true;
    sekmeListesi()
      .then((liste) => {
        if (canli) ayarla(liste);
      })
      .catch(() => {
        /* açılışta sürücü henüz hazır değilse ilk olay zaten getirecek */
      });

    // Tam liste: sıra değişimlerinde kısmi güncelleme arayüzle backend'i
    // sessizce ayrıştırıyor (`docs/IPC.md`).
    const tam = listen<SekmeOzeti[]>(OLAY.sekmeDegisti, (o) => ayarla(o.payload));

    // Tek sekme: başlık her değiştiğinde geliyor, sık ve küçük.
    const tek = listen<SekmeOzeti>(OLAY.sekmeGuncellendi, (o) =>
      ayarla((onceki) => onceki.map((s) => (s.id === o.payload.id ? o.payload : s))),
    );

    return () => {
      canli = false;
      void tam.then((k) => k());
      void tek.then((k) => k());
    };
  }, []);

  return sekmeler;
}

export function etkinSekme(sekmeler: SekmeOzeti[]): SekmeOzeti | undefined {
  return sekmeler.find((s) => s.etkin);
}
