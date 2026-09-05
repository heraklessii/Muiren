/**
 * Sekme ikonu + **durum halkası**.
 *
 * Sekmenin bellek durumu ayrı bir noktada değil, favicon'un çevresinde bir
 * halkada. İki sebebi var ve ikisi de yer:
 *
 * - 44 piksele inmiş bir sekmede favicon dışında hiçbir şey görünmüyor.
 *   Durum ayrı bir nokta olsaydı tam da en çok sekme açıkken — yani durumu
 *   bilmenin en çok işe yaradığı anda — kaybolurdu.
 * - Favicon zaten "bu hangi site" sorusunun cevabı; halkası "ve şu an
 *   bellekte mi" sorusununki. İki bilgi tek bakışta.
 *
 * Bu bir tanıtım öğesi değil, **güven** öğesi (`docs/Sekmeler.md`):
 * kullanıcı neyin bellekte olduğunu görüyor.
 *
 * Bileşen olmasının sebebi tekrar değil **tutarlılık**: aynı dil hem yatay
 * şeritte hem dikey listede geçerli. İki yerde iki ayrı çizim, kullanıcının
 * öğrendiği göstergeyi ikinci panelde yeniden öğrenmesi demek olurdu.
 */

import { Pusula } from "./Ikonlar";
import type { SekmeOzeti } from "../ipc/tipler";

interface Ozellik {
  sekme: SekmeOzeti;
  /** Favicon'un `data:` adresi; yoksa `null` ve harf çiziliyor. */
  adres: string | null;
  /** Kenar uzunluğu. Dikey listede biraz küçük. */
  boyut?: number;
}

/**
 * Favicon'u olmayan sekmenin harfi.
 *
 * `gorunenAd` sayfadan geliyor ve boş olabiliyor; o durumda adresin ilk
 * harfi, o da yoksa soru işareti. Türkçe büyültme (CLAUDE.md #10'un
 * simetriği): düz `toUpperCase()` "istanbul" dizesini "ISTANBUL" yapıp
 * baştaki harfi "I" gösteriyor, doğrusu "İ".
 *
 * Tek karakter: sayfadan gelen metin düzeni bozamıyor (CLAUDE.md #8).
 */
export function basHarf(s: SekmeOzeti): string {
  const kaynak = s.gorunenAd || s.url.replace(/^[a-z]+:\/\/(www\.)?/i, "");
  const harf = [...kaynak.trim()][0];
  return harf ? harf.toLocaleUpperCase("tr") : "?";
}

export function SekmeIkonu({ sekme, adres, boyut = 18 }: Ozellik) {
  // Kabuğun kendi sayfaları (`muiren://yeni`) harf almıyor: sayfanın sahibi
  // biziz ve ailenin glyph'i var. Eskiden "Y" harfi çizilirdi ve yeni sekme,
  // "Y" ile başlayan herhangi bir siteyle aynı görünüyordu.
  const kabuk = sekme.url.startsWith("muiren://");

  return (
    <span
      className={`durum durum--${sekme.durum}${kabuk ? " durum--kabuk" : ""}`}
      style={{ "--durum-boyut": `${boyut}px` } as React.CSSProperties}
      aria-hidden
    >
      {kabuk ? (
        <Pusula boyut={boyut - 2} />
      ) : adres ? (
        /* Favicon sayfadan gelen bir görsel ama **adres değil**: backend
           baytları alıp diske yazdı, buraya bir `data:` adresi olarak
           geliyor. Kabuk hiçbir uzak istek yapmıyor
           (`src-tauri/src/favicon/mod.rs`). */
        <img className="durum__favicon" src={adres} alt="" />
      ) : (
        <span className="durum__harf">{basHarf(sekme)}</span>
      )}
    </span>
  );
}
