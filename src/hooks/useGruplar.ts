/**
 * Sekme grupları.
 *
 * Grup listesi **sekme listesiyle birlikte** tazeleniyor: sekmenin `katli` ve
 * `grupUykuEsigiSn` alanları grubundan türetiliyor (`tabs::Depo::ozetler`) ve
 * grup değişince backend zaten `sekme-degisti` yayınlıyor. Ayrı bir olay
 * eklemek, ikisinin sessizce ayrışabildiği bir yol açardı.
 */

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { grupListesi } from "../ipc";
import { OLAY, type Grup, type GrupId } from "../ipc/tipler";

export interface Gruplar {
  hepsi: Grup[];
  bul: (id: GrupId | null) => Grup | null;
  tazele: () => void;
}

export function useGruplar(): Gruplar {
  const [hepsi, ayarla] = useState<Grup[]>([]);

  const tazele = useCallback(() => {
    void grupListesi().then(ayarla).catch(() => {
      /* sürücü hazır değil; gruplar boş kalıyor ve sekmeler düz çiziliyor */
    });
  }, []);

  useEffect(() => {
    tazele();
    // Grup değişikliği `sekme-degisti` ile geliyor; ayrı bir olay yok.
    const soz = listen(OLAY.sekmeDegisti, () => tazele());
    return () => {
      void soz.then((kapat) => kapat());
    };
  }, [tazele]);

  return {
    hepsi,
    bul: (id) => (id === null ? null : (hepsi.find((g) => g.id === id) ?? null)),
    tazele,
  };
}
