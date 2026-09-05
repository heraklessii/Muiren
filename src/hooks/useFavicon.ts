/**
 * Favicon önbelleği.
 *
 * Sekme özeti bir **kimlik** taşıyor, adres değil: uzak bir adresi kabuğa
 * basmak, açılan her sitenin kabuk penceresine ağ isteği yaptırabilmesi demek
 * olurdu (`src-tauri/src/favicon/mod.rs`).
 *
 * Kimlik içeriğin karması — yani aynı kimlik **hep** aynı baytlar. Bunun iki
 * sonucu var ve ikisi de burada kullanılıyor:
 *
 * - Önbellek hiç geçersizleşmiyor. Bir kez okunan kimlik bir daha
 *   sorulmuyor.
 * - Aynı sitenin 30 sekmesi tek bir okuma yapıyor: kimlik aynı.
 *
 * Önbellek **modül düzeyinde**, bileşen düzeyinde değil. Sekme listesi
 * yeniden çizildiğinde (saniyede birkaç kez olabiliyor) hook yeniden
 * kurulmuyor ve okunmuş ikonlar kaybolmuyor.
 */

import { useEffect, useState } from "react";

import { faviconOku } from "../ipc";

/** kimlik → `data:` adresi. `null`: okundu ve yoktu; tekrar sorulmuyor. */
const ONBELLEK = new Map<string, string | null>();
/** Aynı kimlik için ikinci bir okuma başlatmamak için. */
const UCUSTA = new Map<string, Promise<void>>();

/** Önbelleği değişince haberdar olacak bileşenler. */
const DINLEYICILER = new Set<() => void>();

function duyur() {
  for (const d of DINLEYICILER) d();
}

function yukle(kimlik: string) {
  if (ONBELLEK.has(kimlik) || UCUSTA.has(kimlik)) return;
  const soz = faviconOku(kimlik)
    .then((adres) => {
      ONBELLEK.set(kimlik, adres);
      duyur();
    })
    .catch(() => {
      // Okunamadı: `null` yazılıyor ki her çizimde yeniden denenmesin.
      // Sekme çubuğunda ikon yerine durum noktası kalıyor.
      ONBELLEK.set(kimlik, null);
    })
    .finally(() => {
      UCUSTA.delete(kimlik);
    });
  UCUSTA.set(kimlik, soz);
}

/**
 * Kimlikleri `data:` adreslerine çeviren bir okuyucu döndürür.
 *
 * Çağıran her çizimde `bul(kimlik)` diyor; okunmamış kimlik arka planda
 * yükleniyor ve geldiğinde bileşen bir kez yeniden çiziliyor.
 */
export function useFavicon(): (kimlik: string | null) => string | null {
  const [, tazele] = useState(0);

  useEffect(() => {
    const dinleyici = () => tazele((n) => n + 1);
    DINLEYICILER.add(dinleyici);
    return () => {
      DINLEYICILER.delete(dinleyici);
    };
  }, []);

  return (kimlik) => {
    if (!kimlik) return null;
    if (!ONBELLEK.has(kimlik)) {
      yukle(kimlik);
      return null;
    }
    return ONBELLEK.get(kimlik) ?? null;
  };
}
