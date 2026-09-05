/**
 * Adres çubuğu önerileri — **yerel, ağ isteği yok**.
 *
 * `docs/Frontend.md`: "Öneri listesi: geçmiş + yer imleri + açık sekmeler.
 * **Ağ isteği yok** — arama motorunun canlı öneri API'si telemetri demek."
 * Bu kanca o kuralın kod karşılığı: tek veri kaynağı `gecmis_ara` komutu ve
 * o da yalnız yerel SQLite'a bakıyor.
 *
 * Sorgu **geciktiriliyor**: kullanıcı yazarken her tuşta bir IPC turu +
 * SQLite sorgusu çalıştırmanın karşılığı yok. 120 ms, yazma hızının altında
 * kalıp listeyi canlı hissettiren aralık.
 */

import { useEffect, useState } from "react";

import { gecmisAra } from "../ipc";
import type { GecmisKaydi } from "../ipc/tipler";

const GECIKME_MS = 120;

export function useOneriler(sorgu: string | null, limit = 8): GecmisKaydi[] {
  const [oneriler, ayarla] = useState<GecmisKaydi[]>([]);

  useEffect(() => {
    if (sorgu === null) {
      ayarla([]);
      return;
    }

    let canli = true;
    const zaman = window.setTimeout(() => {
      void gecmisAra(sorgu, limit)
        .then((k) => {
          if (canli) ayarla(k);
        })
        .catch(() => {
          // Veritabanı açılamamış olabilir; öneri yokluğu adres çubuğunu
          // çalışmaz hâle getirmiyor.
          if (canli) ayarla([]);
        });
    }, GECIKME_MS);

    return () => {
      canli = false;
      window.clearTimeout(zaman);
    };
  }, [sorgu, limit]);

  return oneriler;
}
