/**
 * Yeni sekme sayfasının verisi — **yalnız yerel**.
 *
 * İki kaynak var ve ikisi de diskte: geçmiş (`gecmis_ara`, boş sorgu en sık
 * gidilenleri veriyor) ve yer imleri (`yer_imi_listesi`). **Ağ isteği yok**
 * (`docs/Roadmap.md` karar #5): bir tarayıcının açtığı ilk sayfa, henüz
 * hiçbir siteye gitmemişken bir sunucuya bağlanıyorsa o tarayıcı telemetri
 * topluyor demektir.
 *
 * Kanca sayfa göründüğünde bir kez çalışıyor; sayfa kapalıyken (yani
 * kullanıcı bir sitedeyken) hiç sorgu yapılmıyor — `YeniSekme` bileşeni
 * `App.tsx` içinde koşullu çiziliyor ve kancanın ömrü onunla aynı.
 */

import { useEffect, useState } from "react";

import { gecmisAra, yerImiListesi } from "../ipc";
import type { GecmisKaydi, YerImi } from "../ipc/tipler";

/** Kaç kutu. Bir satıra sığan sayının katı; fazlası duvar gibi görünüyor. */
const SIK_LIMIT = 8;
const YER_IMI_LIMIT = 10;

interface Veri {
  sikGidilenler: GecmisKaydi[];
  yerImleri: YerImi[];
  /** İlk tur bitti mi. Bitmeden "geçmiş boş" demek yanlış olurdu. */
  yuklendi: boolean;
}

export function useYeniSekme(acik: boolean): Veri {
  const [sikGidilenler, ayarlaSik] = useState<GecmisKaydi[]>([]);
  const [yerImleri, ayarlaYerImleri] = useState<YerImi[]>([]);
  const [yuklendi, ayarlaYuklendi] = useState(false);

  useEffect(() => {
    if (!acik) return;
    let canli = true;

    // Boş sorgu = "sayaca göre en üsttekiler" (`history/depo.rs`, `ara`).
    // Ayrı bir komut açılmadı: aynı sıralama zaten adres çubuğu önerilerinin
    // sıralaması ve iki farklı "en sık" tanımı, iki farklı liste demek olurdu.
    void Promise.allSettled([gecmisAra("", SIK_LIMIT), yerImiListesi(null)]).then(
      ([g, y]) => {
        if (!canli) return;
        if (g.status === "fulfilled") ayarlaSik(g.value);
        if (y.status === "fulfilled") ayarlaYerImleri(y.value.slice(0, YER_IMI_LIMIT));
        // Veritabanı açılamamış olabilir (`docs/Depolama.md`): sayfa yine
        // çiziliyor, yalnız listeler boş. Yer imi tutamayan bir tarayıcı
        // hâlâ gezinebilmeli.
        ayarlaYuklendi(true);
      },
    );

    return () => {
      canli = false;
    };
  }, [acik]);

  return { sikGidilenler, yerImleri, yuklendi };
}
