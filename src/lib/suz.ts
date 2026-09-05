/**
 * Sekme arama (`Ctrl+Shift+A`) — saf ve testli.
 *
 * Uyuyan ve atılmış sekmeler de aranıyor ve **uyandırılmıyor** (CLAUDE.md #5):
 * başlık ve adres zaten bellekteki kayıtta, arama onlara bakıyor.
 */

import type { SekmeOzeti } from "../ipc/tipler";

/**
 * Türkçe küçültme.
 *
 * Düz `toLowerCase()` "İSTANBUL" dizesini `i̇stanbul` (i + birleşik nokta)
 * yapıyor ve kullanıcının yazdığı "istanbul" ile eşleşmiyor; "I" ise "i"
 * oluyor, oysa Türkçede "ı" olmalı. Aramada kaçırılan tek harf, kullanıcı
 * için "arama çalışmıyor" demek (CLAUDE.md #10).
 */
export function kucult(metin: string): string {
  return metin.toLocaleLowerCase("tr");
}

/**
 * Sorguyu boşluklardan ayırıp **her parçası** eşleşen sekmeleri döndürür.
 *
 * "yt anime" yazan biri hem "youtube" hem "anime" geçen sekmeyi arıyor; tek
 * parça olarak aransaydı hiçbir şey bulunmazdı.
 */
export function suz(sekmeler: SekmeOzeti[], sorgu: string): SekmeOzeti[] {
  const parcalar = kucult(sorgu.trim())
    .split(/\s+/)
    .filter((p) => p !== "");
  if (parcalar.length === 0) return sekmeler;

  return sekmeler.filter((s) => {
    const havuz = `${kucult(s.gorunenAd)} ${kucult(s.baslik)} ${kucult(s.url)}`;
    return parcalar.every((p) => havuz.includes(p));
  });
}

/**
 * Sekme listesini arama sonucu için sıralar.
 *
 * Başlığı sorguyla **başlayan** sekmeler önce: 50 sekme arasında aradığını
 * bulmanın en hızlı yolu bu. Eşitlikte sekme çubuğundaki sıra korunuyor —
 * kullanıcının zihnindeki düzen o.
 */
export function sirala(sekmeler: SekmeOzeti[], sorgu: string): SekmeOzeti[] {
  const ilk = kucult(sorgu.trim()).split(/\s+/)[0] ?? "";
  if (ilk === "") return sekmeler;

  return sekmeler
    .map((s, sira) => ({ s, sira, basliyor: kucult(s.gorunenAd).startsWith(ilk) }))
    .sort((a, b) => Number(b.basliyor) - Number(a.basliyor) || a.sira - b.sira)
    .map((x) => x.s);
}
