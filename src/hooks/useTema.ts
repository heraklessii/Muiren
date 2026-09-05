/**
 * Temanın arayüzdeki yansıması.
 *
 * Jetonlar `document.documentElement.style` üzerine **tek tek** yazılıyor.
 * `<style>` etiketi enjekte edilmiyor, `innerHTML` kullanılmıyor
 * (`docs/Temalar.md`) — ikisi de temaya rastgele CSS yazma yolu açardı ve
 * karar #3'ün tamamı "tema veri, kod değil" üzerine kurulu.
 *
 * **Burada ikinci bir doğrulama yok ve olmamalı.** Değerler backend'in
 * `theme/jeton.rs` süzgecinden geçti; buraya bir süzgeç daha koymak, biri
 * gevşediğinde fark edilmeyen bir açık demek. Arayüzün işi yazmak.
 */

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { temaArkaplan, temaEtkin } from "../ipc";
import { OLAY, type TemaOzeti } from "../ipc/tipler";

/** Temanın dokunabildiği jetonlar; tema değişince eskiler temizleniyor. */
const JETONLAR = [
  "--bg",
  "--bg-panel",
  "--bg-elevated",
  "--bg-sunken",
  "--border",
  "--border-strong",
  "--text",
  "--text-muted",
  "--accent",
  "--accent-strong",
  "--on-accent",
  "--radius",
  "--radius-lg",
] as const;

function uygula(tema: TemaOzeti | null) {
  const kok = document.documentElement;

  // Önce hepsini kaldır: yeni tema bir jetonu tanımlamıyorsa eskisinin
  // kalması, tema yarım uygulanmış gibi görünmesine yol açardı.
  for (const j of JETONLAR) kok.style.removeProperty(j);
  kok.style.removeProperty("--arkaplan-gorsel");
  kok.style.removeProperty("--arkaplan-karartma");
  kok.style.removeProperty("--arkaplan-bulanik");
  kok.removeAttribute("data-arkaplan");

  if (!tema) return;

  for (const [ad, deger] of tema.jetonlar) kok.style.setProperty(ad, deger);

  if (tema.arkaplan) {
    // Karartma ve bulanıklık hemen; **görsel ayrı ve eşzamansız** geliyor
    // (`gorseliYukle`). Sebebi boyut: duvar kâğıdı megabaytlarca olabiliyor
    // ve `TemaOzeti` her tema listesinde gidiyor — görseli oraya koymak,
    // tema listesini açan kullanıcıya üç duvar kâğıdını birden göndermek
    // olurdu.
    kok.style.setProperty("--arkaplan-karartma", String(tema.arkaplan.karartma));
    kok.style.setProperty("--arkaplan-bulanik", `${tema.arkaplan.bulanik}px`);
    kok.setAttribute("data-arkaplan", tema.arkaplan.yerlesim);
  }
}

/**
 * Arka plan görselini yükler.
 *
 * Adres bir `data:` URL'i; dosya yolu **değil**. Kabuğa dosya sistemi kanalı
 * (Tauri `asset` protokolü) açmamak için — favicon deposuyla birebir aynı
 * gerekçe (`src-tauri/src/theme/mod.rs`, `arkaplan_veri`).
 *
 * Değer `url("...")` olarak sarılıyor ve içine **tırnak kaçışı** eklenmiyor:
 * `data:` adresi base64 alfabesi dışında karakter taşımıyor, dolayısıyla
 * tırnak kırmak mümkün değil. Kullanıcı yolu buraya hiç gelmiyor.
 */
function gorseliYukle(tema: TemaOzeti | null) {
  const kok = document.documentElement;
  if (!tema?.arkaplan) {
    kok.style.removeProperty("--arkaplan-gorsel");
    return;
  }
  void temaArkaplan()
    .then((adres) => {
      if (adres) kok.style.setProperty("--arkaplan-gorsel", `url("${adres}")`);
      else kok.style.removeProperty("--arkaplan-gorsel");
    })
    .catch(() => {
      // Görsel okunamadı (silinmiş, çok büyük). Tema yine de uygulanmış
      // durumda; arka plan olmadan çalışmaya devam ediyor.
      kok.style.removeProperty("--arkaplan-gorsel");
    });
}

export function useTema(): {
  tema: TemaOzeti | null;
  tazele: () => void;
} {
  const [tema, ayarla] = useState<TemaOzeti | null>(null);

  const tazele = useCallback(() => {
    void temaEtkin()
      .then((t) => {
        ayarla(t);
        uygula(t);
        gorseliYukle(t);
      })
      .catch(() => {
        /* sürücü hazır değil; `styles.css` varsayılanları duruyor */
      });
  }, []);

  useEffect(() => {
    tazele();
    const soz = listen<TemaOzeti>(OLAY.temaDegisti, (e) => {
      ayarla(e.payload);
      uygula(e.payload);
      gorseliYukle(e.payload);
    });
    return () => {
      void soz.then((kapat) => kapat());
    };
  }, [tazele]);

  return { tema, tazele };
}
