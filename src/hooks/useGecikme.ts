/**
 * Uyanma gecikmesinin arayüzdeki yansıması (`docs/Bellek.md` kabul tablosu).
 *
 * Diğer kancalardan bir farkı var: **kendi olayı yok.** Gecikme kullanıcı
 * sekme değiştirdikçe değişiyor ve her uyanma için ayrı bir olay yayınlamak,
 * sekme değiştirmenin sıcak yoluna bir yayın daha eklemek olurdu.
 *
 * Onun yerine gözcünün turuna binildi: `muiren://bellek-ozeti` zaten
 * saniyeler ölçeğinde gelen tek düzenli nabız. İkinci bir zamanlayıcı kurmak
 * — bellek iddiası olan bir programda — karşılığı olmayan bir uyandırma
 * demekti.
 *
 * Bedeli, panelin gecikmeyi bir gözcü turu geriden göstermesi. Kabul edilebilir:
 * bu sayı anlık bir gösterge değil, bir ölçüm oturumunun sonucu.
 */

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { gecikmeOzeti, gecikmeSifirla } from "../ipc";
import { OLAY, type BellekOzeti, type GecikmeOzeti } from "../ipc/tipler";

export function useGecikme(): {
  gecikme: GecikmeOzeti | null;
  sifirla: () => void;
} {
  const [gecikme, ayarla] = useState<GecikmeOzeti | null>(null);

  const tazele = useCallback(() => {
    void gecikmeOzeti()
      .then(ayarla)
      .catch(() => {
        /* sürücü henüz hazır değil; bir sonraki gözcü turu düzeltiyor */
      });
  }, []);

  useEffect(() => {
    tazele();
    const soz = listen<BellekOzeti>(OLAY.bellekOzeti, () => tazele());
    return () => {
      void soz.then((kapat) => kapat());
    };
  }, [tazele]);

  const sifirla = useCallback(() => {
    void gecikmeSifirla()
      .then(tazele)
      .catch(() => {
        /* yok sayılıyor: sıfırlama başarısızsa eski dağılım duruyor */
      });
  }, [tazele]);

  return { gecikme, sifirla };
}
