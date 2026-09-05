/**
 * Teşhis ölçümleri — **yan etkiler burada** (`docs/Medya.md`).
 *
 * Yorumlama `lib/teshis.ts` içinde ve saf; bu dosya yalnız tarayıcıya soruyor.
 * Ayrım `memory/olcum.rs` ile `memory/esik.rs` arasındaki ayrımın aynısı.
 *
 * **Hiçbir sorusu sekmeye inmiyor.** Bütün ölçümler kabuk webview'inin kendi
 * içinde koşuyor ve cevap temsili: bütün webview'ler aynı WebView2 ortamını,
 * aynı bayrak dizesini ve aynı GPU süreçlerini paylaşıyor (CLAUDE.md #16).
 * Sekmeye betik göndermek uyuyan bir sekmeyi uyandırma riski demekti
 * (CLAUDE.md #5) ve buradaki soruların hiçbiri buna değmiyor.
 *
 * Ölçüm **istendiğinde** koşuyor, açılışta değil: panel açılmadıkça bir WebGL
 * bağlamı yaratılmıyor ve bir codec sorgusu yapılmıyor.
 */

import { useCallback, useEffect, useState } from "react";

import { codecDurumu, webview2Surumu, yazilimOlusturucu } from "../lib/teshis";
import type { CodecDurumu } from "../lib/teshis";

/** `docs/Medya.md` içindeki codec tablosunun sorulabilir hâli. */
const CODECLER: { ad: string; tur: "video" | "audio"; mime: string }[] = [
  { ad: "H.264", tur: "video", mime: 'video/mp4; codecs="avc1.42E01E"' },
  { ad: "VP9", tur: "video", mime: 'video/webm; codecs="vp09.00.10.08"' },
  { ad: "AV1", tur: "video", mime: 'video/mp4; codecs="av01.0.05M.08"' },
  { ad: "HEVC", tur: "video", mime: 'video/mp4; codecs="hvc1.1.6.L93.B0"' },
  { ad: "AAC", tur: "audio", mime: 'audio/mp4; codecs="mp4a.40.2"' },
  { ad: "Opus", tur: "audio", mime: 'audio/webm; codecs="opus"' },
];

/** `docs/Medya.md`, "DRM — projenin en büyük riski". */
const ANAHTAR_SISTEMLERI = [
  { ad: "Widevine", kimlik: "com.widevine.alpha" },
  { ad: "PlayReady", kimlik: "com.microsoft.playready.recommendation" },
];

export interface CodecSatiri {
  ad: string;
  tur: "video" | "audio";
  durum: CodecDurumu | null;
}

export interface TeshisSonucu {
  webview2: string | null;
  gpu: string | null;
  gpuYazilim: boolean;
  codecler: CodecSatiri[];
  drm: { ad: string; var: boolean | null }[];
  cekirdek: number | null;
  platform: string;
  muiren: string | null;
}

/**
 * WebGL oluşturucu dizesi.
 *
 * Bağlam **hemen bırakılıyor** (`loseContext`): teşhis için açılan bir WebGL
 * bağlamının panelden sonra da GPU belleği tutması, bellek iddiası olan bir
 * programda kendi ölçüm aracımızın bir bellek kalemi olması demekti.
 */
function gpuDizesi(): string | null {
  try {
    const tuval = document.createElement("canvas");
    const gl = (tuval.getContext("webgl2") ??
      tuval.getContext("webgl")) as WebGLRenderingContext | null;
    if (!gl) return null;
    const ek = gl.getExtension("WEBGL_debug_renderer_info");
    const dize = ek
      ? (gl.getParameter(ek.UNMASKED_RENDERER_WEBGL) as string)
      : (gl.getParameter(gl.RENDERER) as string);
    gl.getExtension("WEBGL_lose_context")?.loseContext();
    return typeof dize === "string" && dize.trim() !== "" ? dize : null;
  } catch {
    // GPU süreci düşmüş olabilir; teşhis ekranının kendisi çökmemeli.
    return null;
  }
}

async function codecSor(mime: string, tur: "video" | "audio"): Promise<CodecDurumu | null> {
  const yetenek = navigator.mediaCapabilities;
  if (!yetenek?.decodingInfo) return null;
  try {
    const cevap = await yetenek.decodingInfo(
      tur === "video"
        ? {
            type: "media-source",
            video: { contentType: mime, width: 1920, height: 1080, bitrate: 4_000_000, framerate: 30 },
          }
        : { type: "media-source", audio: { contentType: mime } },
    );
    return codecDurumu({ supported: cevap.supported, powerEfficient: cevap.powerEfficient });
  } catch {
    // Tanınmayan `contentType` bazı sürümlerde hata atıyor; bu "yok" demek
    // değil, "sorulamadı" demek (`lib/teshis.ts`, `codecDurumu`).
    return null;
  }
}

async function drmSor(kimlik: string): Promise<boolean | null> {
  if (!navigator.requestMediaKeySystemAccess) return null;
  try {
    await navigator.requestMediaKeySystemAccess(kimlik, [
      {
        initDataTypes: ["cenc"],
        videoCapabilities: [{ contentType: 'video/mp4; codecs="avc1.42E01E"' }],
      },
    ]);
    return true;
  } catch {
    // Anahtar sistemi yok ya da bu yapılandırmayla verilmiyor. İkisini
    // ayırt edemiyoruz ve **ayırt ediyormuş gibi yapmıyoruz**: panel bunu
    // "bulunamadı" diye yazıyor, "DRM çalışmaz" diye değil.
    return false;
  }
}

async function muirenSurumu(): Promise<string | null> {
  try {
    const { getVersion } = await import("@tauri-apps/api/app");
    return await getVersion();
  } catch {
    // Tarayıcıda (`npm run dev`) Tauri yok.
    return null;
  }
}

/**
 * Teşhis ölçümlerini koşturur.
 *
 * `tazele` ile yeniden koşuyor: GPU hızlandırma oturum ortasında da düşebilir
 * (sürücü çökmesi) ve o an paneli yeniden açmak zorunda kalmak, teşhis
 * aracının kendisini kullanışsız yapardı.
 */
export function useTeshis(acik: boolean) {
  const [sonuc, ayarlaSonuc] = useState<TeshisSonucu | null>(null);
  const [sayac, ayarlaSayac] = useState(0);

  useEffect(() => {
    if (!acik) return;
    let iptal = false;

    void (async () => {
      const gpu = gpuDizesi();
      const codecler = await Promise.all(
        CODECLER.map(async (c) => ({
          ad: c.ad,
          tur: c.tur,
          durum: await codecSor(c.mime, c.tur),
        })),
      );
      const drm = await Promise.all(
        ANAHTAR_SISTEMLERI.map(async (a) => ({ ad: a.ad, var: await drmSor(a.kimlik) })),
      );
      const muiren = await muirenSurumu();
      if (iptal) return;

      ayarlaSonuc({
        webview2: webview2Surumu(navigator.userAgent),
        gpu,
        gpuYazilim: gpu !== null && yazilimOlusturucu(gpu),
        codecler,
        drm,
        cekirdek: navigator.hardwareConcurrency || null,
        // `userAgentData` her sürümde tipli değil; varsa daha doğru cevabı o
        // veriyor (`navigator.platform` kullanımdan kalkmış sayılıyor).
        platform:
          (navigator as Navigator & { userAgentData?: { platform?: string } })
            .userAgentData?.platform ??
          navigator.platform ??
          "",
        muiren,
      });
    })();

    return () => {
      iptal = true;
    };
  }, [acik, sayac]);

  const tazele = useCallback(() => ayarlaSayac((s) => s + 1), []);
  return { sonuc, tazele };
}
