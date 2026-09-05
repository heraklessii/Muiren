/**
 * İndirme önerisi.
 *
 * `IndirmePolitikasi::Sor` altında motor indirmeyi iptal ediyor ve backend
 * `muiren://indirme-onerisi` yayınlıyor. Kullanıcı iki şeyden birini
 * seçiyor: Muiget'e gönder ya da Muiren indirsin.
 *
 * **Neden bir modal değil** — indirme, sayfanın işini durdurmuyor;
 * kullanıcıyı da durdurmamalı. Öneri adres çubuğunun altında bir şerit
 * olarak çıkıyor ve kendiliğinden kayboluyor.
 *
 * "Muiren indirsin" seçildiğinde ne oluyor: iptal edilmiş bir WebView2
 * indirmesi **sürdürülemiyor**, o yüzden sekme aynı adrese yeniden
 * gönderiliyor ve backend o adres için tek atımlık bir bilet tutuyor
 * (`tabs::surucu::Surucu::indirme_izin_ver`).
 */

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { indirmeIzinVer, muigetGonder } from "../ipc";
import { OLAY, type IndirmeOlayi } from "../ipc/tipler";

/** Devredildi/başarısız bildirimlerinin ekranda kalma süresi. */
const BILDIRIM_SN = 6000;

export interface Indirme {
  /** Kullanıcının karar vermesi beklenen öneri; yoksa `null`. */
  oneri: IndirmeOlayi | null;
  /** Son sonuç bildirimi (devredildi / başarısız). */
  bildirim: IndirmeOlayi | null;
  muigeteGonder: () => void;
  muirenIndirsin: () => void;
  kapat: () => void;
}

export function useIndirme(): Indirme {
  const [oneri, ayarlaOneri] = useState<IndirmeOlayi | null>(null);
  const [bildirim, ayarlaBildirim] = useState<IndirmeOlayi | null>(null);

  useEffect(() => {
    const soz = listen<IndirmeOlayi>(OLAY.indirmeOnerisi, (e) => {
      if (e.payload.sonuc === "soruluyor") ayarlaOneri(e.payload);
      else ayarlaBildirim(e.payload);
    });
    return () => {
      void soz.then((kapat) => kapat());
    };
  }, []);

  // Bildirim kendiliğinden kayboluyor; öneri **kaybolmuyor**. Fark önemli:
  // bildirimi kaçırmak bir şey kaybettirmiyor, öneriyi kaçırmak indirmeyi
  // kaybettiriyor.
  useEffect(() => {
    if (!bildirim) return;
    const zaman = window.setTimeout(() => ayarlaBildirim(null), BILDIRIM_SN);
    return () => window.clearTimeout(zaman);
  }, [bildirim]);

  const muigeteGonder = useCallback(() => {
    if (!oneri) return;
    const { url, dosyaAdi } = oneri;
    ayarlaOneri(null);
    void muigetGonder(url, dosyaAdi).catch(() => {
      // Devretme hatası **sessiz kalmıyor** (`docs/Kopruler.md`): backend
      // zaten `basarisiz` sonuçlu bir olay yayınlıyor ve bildirim olarak
      // görünüyor.
    });
  }, [oneri]);

  const muirenIndirsin = useCallback(() => {
    if (!oneri) return;
    const { id, url } = oneri;
    ayarlaOneri(null);
    void indirmeIzinVer(id, url).catch(() => {});
  }, [oneri]);

  return {
    oneri,
    bildirim,
    muigeteGonder,
    muirenIndirsin,
    kapat: () => ayarlaOneri(null),
  };
}
