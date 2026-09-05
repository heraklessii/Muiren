/**
 * Engelleme rozeti.
 *
 * `docs/Roadmap.md` Faz 4 ve `src-tauri/src/engel/mod.rs`: engellenen şey
 * **sessizce kaybolmuyor.** Sessiz engelleme, kullanıcının tarayıcıyı bozuk
 * sanması demek — özellikle pop-up'ta, çünkü bazı siteler ödeme ya da giriş
 * akışını pencerede açıyor.
 *
 * Sayaç **sekme ve sayfa başına**: yeni bir gezinme başladığında backend
 * sıfırlıyor. Rozet "bu sayfada 12 istek engellendi" diyor, "bu sekmenin
 * ömrü boyunca" değil.
 */

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { engelSayaci } from "../ipc";
import { OLAY, type EngelOlayi, type SekmeId } from "../ipc/tipler";

export interface Engel {
  /** Etkin sekmede bu sayfada engellenen istek sayısı. */
  sayi: number;
  /** Son engellenen şey; rozetin ipucunda görünüyor. */
  son: EngelOlayi | null;
}

export function useEngel(etkinId: SekmeId | null): Engel {
  const [sayi, ayarlaSayi] = useState(0);
  const [son, ayarlaSon] = useState<EngelOlayi | null>(null);

  // Sekme değişince sayaç backend'den bir kez okunuyor: olaylar yalnız
  // ileriye doğru geliyor ve sekme değiştiren kullanıcı, o sekmede daha
  // önce engellenenleri de görmeli.
  useEffect(() => {
    ayarlaSon(null);
    if (etkinId === null) {
      ayarlaSayi(0);
      return;
    }
    let canli = true;
    void engelSayaci(etkinId)
      .then((n) => {
        if (canli) ayarlaSayi(n);
      })
      .catch(() => {});
    return () => {
      canli = false;
    };
  }, [etkinId]);

  useEffect(() => {
    const soz = listen<EngelOlayi>(OLAY.engellendi, (e) => {
      // Başka bir sekmenin olayı bu sekmenin rozetini değiştirmiyor.
      if (e.payload.id !== etkinId) return;
      ayarlaSayi(e.payload.sayi);
      ayarlaSon(e.payload);
    });
    return () => {
      void soz.then((kapat) => kapat());
    };
  }, [etkinId]);

  return { sayi, son };
}
