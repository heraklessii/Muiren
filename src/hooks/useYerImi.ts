/**
 * Etkin sekmenin adresi yer imlerinde mi.
 *
 * Adres çubuğundaki yıldız bunu okuyor. Kanca aynı zamanda ekleme/çıkarma
 * eylemini de veriyor çünkü ikisi tek durumu paylaşıyor: eklendikten sonra
 * yıldızın dolması için ayrı bir sorgu turu atmanın karşılığı yok.
 *
 * Yer imleri kullanıcının kendi ürettiği içerik: "verileri temizle" ekranında
 * **yok** ve silinmeleri tek tek (`docs/Depolama.md`).
 */

import { useCallback, useEffect, useState } from "react";

import { yerImiEkle, yerImiMi, yerImiSil } from "../ipc";

export function useYerImi(
  url: string | undefined,
  baslik: string | undefined,
): { yerImiId: number | null; degistir: () => void } {
  const [yerImiId, ayarla] = useState<number | null>(null);

  useEffect(() => {
    if (!url) {
      ayarla(null);
      return;
    }
    let canli = true;
    void yerImiMi(url)
      .then((id) => {
        if (canli) ayarla(id);
      })
      .catch(() => {
        if (canli) ayarla(null);
      });
    return () => {
      canli = false;
    };
  }, [url]);

  const degistir = useCallback(() => {
    if (!url) return;
    if (yerImiId !== null) {
      void yerImiSil(yerImiId).then(() => ayarla(null));
    } else {
      // Başlık yoksa adres kullanılıyor: başlıksız bir yer imi listede boş
      // satır olarak görünürdü.
      void yerImiEkle(url, baslik || url).then((id) => ayarla(id));
    }
  }, [url, baslik, yerImiId]);

  return { yerImiId, degistir };
}
