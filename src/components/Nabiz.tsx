/**
 * Nabız — gezinme çubuğundaki bellek göstergesi.
 *
 * Muiren'in tek iddiası bellek ve o iddia bugüne kadar **saklıydı**: rakamı
 * görmek için `Ctrl+Shift+M` ile bir panel açmak gerekiyordu. Panelin
 * kendisi doğru bir yer (`BellekPaneli`, "projenin vitrini") ama vitrin
 * kapalı bir çekmecede duruyordu; kullanıcı politikanın çalıştığını hiç
 * görmeden aylarca kullanabiliyordu.
 *
 * Bu bileşen o çekmeceyi açık bırakıyor: üç renkli kısa bir çubuk ve tek bir
 * rakam. Çubuk sekmelerin **durum dağılımı** — uyanık, uyuyan, atılmış.
 * Tıklanınca bellek paneli açılıyor.
 *
 * ## Kurallar
 *
 * - **Hiçbir sekmeye dokunmuyor** (CLAUDE.md #5). Tek kaynağı backend'in
 *   yayınladığı `BellekOzeti`; kendi ölçümünü yapmıyor, uyuyan sekmeyi
 *   uyandırmıyor.
 * - **Yaklaşık değer kesin gibi gösterilmiyor** (`docs/Bellek.md`):
 *   `olcumYaklasik` true ise rakamın başında `~` var — biçimleme
 *   `lib/bicim.ts` içinde ve bellek paneliyle aynı fonksiyon, iki yerde iki
 *   farklı sayı görünmesin diye.
 * - Özet **yokken** çizilmiyor. Gözcü ilk turunu koşmadan sıfır göstermek,
 *   ölçümün yokluğunu bir ölçüm sonucu gibi sunmak olurdu.
 */

import { bellek, durumOzeti } from "../lib/bicim";
import type { BellekOzeti } from "../ipc/tipler";

interface Ozellik {
  ozet: BellekOzeti | null;
  onAc: () => void;
}

/** Baskı seviyesinin okunur karşılığı; ipucunda görünüyor. */
const BASKI_ADI = {
  dusuk: "sistem rahat",
  orta: "sistem orta yüklü",
  yuksek: "sistem yüksek yüklü",
  kritik: "sistem kritik yüklü",
} as const;

export function Nabiz({ ozet, onAc }: Ozellik) {
  if (!ozet) return null;

  const uyanik = ozet.etkin + ozet.arkaplan;
  const toplam = uyanik + ozet.uyuyan + ozet.atilmis;
  if (toplam === 0) return null;

  const oran = (n: number) => `${(n / toplam) * 100}%`;

  const ipucu = [
    `${bellek(ozet.toplamMb, ozet.olcumYaklasik)} — sayfa katmanı`,
    durumOzeti(uyanik, ozet.uyuyan, ozet.atilmis),
    ozet.tahminiKazancMb > 0
      ? `uyutma kazancı: ${bellek(ozet.tahminiKazancMb, ozet.olcumYaklasik)}`
      : "",
    BASKI_ADI[ozet.baski],
    "Bellek panelini açmak için tıkla (Ctrl+Shift+M)",
  ]
    .filter((s) => s !== "")
    .join("\n");

  return (
    <button
      type="button"
      className={`nabiz nabiz--${ozet.baski}`}
      onClick={onAc}
      title={ipucu}
    >
      {/*
        Çubuk `aria-hidden`: aynı bilgi yanındaki rakamda ve `title` içinde
        zaten var, ekran okuyucuya üç anlamsız kutu okutmanın karşılığı yok.
      */}
      <span className="nabiz__cubuk" aria-hidden>
        <i className="nabiz__dilim nabiz__dilim--uyanik" style={{ width: oran(uyanik) }} />
        <i
          className="nabiz__dilim nabiz__dilim--uyuyan"
          style={{ width: oran(ozet.uyuyan) }}
        />
        <i
          className="nabiz__dilim nabiz__dilim--atilmis"
          style={{ width: oran(ozet.atilmis) }}
        />
      </span>
      <span className="nabiz__sayi">{bellek(ozet.toplamMb, ozet.olcumYaklasik)}</span>
    </button>
  );
}
