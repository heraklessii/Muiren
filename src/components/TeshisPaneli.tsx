/**
 * Teşhis ekranı — `docs/Medya.md` içindeki üç maddenin ikisi ve üçüncüsü.
 *
 * Belge şunu istiyordu: donanım kod çözmenin **düştüğünü fark etmek** ve
 * **ölçüm sunmak**. Gerekçesi de yazılıydı: "Kullanıcı sürücü/GPU sorununu
 * bilmiyorsa tarayıcıyı suçluyor."
 *
 * Ekran dört şeyi gösteriyor ve dördü de bir belgedeki açık soruya karşılık
 * geliyor:
 *
 * | Bölüm | Karşılığı |
 * |---|---|
 * | GPU | `docs/Medya.md` tablosunun 1. satırı (kare atlıyor, CPU %100) |
 * | Codec | `docs/Medya.md`, "Codec durumu (doğrulanacak, Faz 0)" |
 * | DRM | `docs/Medya.md`, "DRM — projenin en büyük riski" (Faz 0/R2) |
 * | Rapor | `docs/olcumler/README.md` başlık tablosu |
 *
 * **Hiçbir satır sekmeye dokunmuyor** (CLAUDE.md #5): ölçümler kabuk
 * webview'inin kendi içinde koşuyor ve bütün webview'ler aynı WebView2
 * ortamını paylaştığı için cevap temsili (CLAUDE.md #16).
 */

import { useState } from "react";

import { Kapat } from "./Ikonlar";
import { useTeshis } from "../hooks/useTeshis";
import { useYetenekler } from "../hooks/useYetenekler";
import { raporBasligi } from "../lib/teshis";
import type { CodecDurumu } from "../lib/teshis";
import type { BellekOzeti, Settings } from "../ipc/tipler";

interface Ozellik {
  onKapat: () => void;
  ayarlar: Settings | null;
  ozet: BellekOzeti | null;
}

const CODEC_METNI: Record<CodecDurumu, string> = {
  donanim: "donanım",
  yazilim: "yazılım",
  yok: "yok",
};

const PROFIL_ADI: Record<string, string> = {
  siki: "Sıkı",
  dengeli: "Dengeli",
  rahat: "Rahat",
};

export function TeshisPaneli({ onKapat, ayarlar, ozet }: Ozellik) {
  const { sonuc, tazele } = useTeshis(true);
  const yetenekler = useYetenekler();
  const [kopyalandi, ayarlaKopyalandi] = useState(false);

  const rapor = () =>
    raporBasligi({
      tarih: new Date().toISOString().slice(0, 10),
      cekirdek: sonuc?.cekirdek ?? null,
      // RAM **backend ölçümünden** geliyor: `navigator.deviceMemory` gizlilik
      // gerekçesiyle 8 GB'da tavanlanıyor ve 32 GB'lık bir makineyi 8 GB
      // gösterirdi (`memory/olcum.rs` gerçek değeri veriyor).
      sistemToplamMb: ozet?.sistemToplamMb ?? null,
      platform: sonuc?.platform ?? "",
      gpu: sonuc?.gpu ?? null,
      webview2: sonuc?.webview2 ?? null,
      muiren: sonuc?.muiren ?? null,
      profil: ayarlar ? (PROFIL_ADI[ayarlar.profil] ?? ayarlar.profil) : "?",
      surecPolitikasi:
        ayarlar?.surecPolitikasi === "birlesik"
          ? "process-per-site açık"
          : "Chromium varsayılanı",
    });

  const kopyala = () => {
    void navigator.clipboard
      .writeText(rapor())
      .then(() => {
        ayarlaKopyalandi(true);
        window.setTimeout(() => ayarlaKopyalandi(false), 2000);
      })
      .catch(() => {
        /* pano yoksa metin aşağıda zaten duruyor */
      });
  };

  return (
    <div className="ortu" onClick={onKapat}>
      <div
        className="ayarlar"
        role="dialog"
        aria-label="Teşhis"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="ayarlar__baslik">
          <h2>Teşhis</h2>
          <button
            type="button"
            className="dugme dugme--ikon"
            onClick={onKapat}
            aria-label="Kapat"
          >
            <Kapat boyut={14} />
          </button>
        </div>

        <div className="ayarlar__govde">
          <section className="ayarlar__bolum">
            <h3>Görüntü</h3>
            {sonuc?.gpuYazilim && (
              <p className="ayarlar__uyari">
                <b>GPU hızlandırma kapalı.</b> Sayfalar ve video işlemciyle
                çiziliyor: kare atlaması ve yüksek CPU kullanımı bundan.
                Genelde sebebi sürücü ya da uzak masaüstü oturumu — Muiren GPU
                hızlandırmayı kapatan bir bayrak geçirmiyor.
              </p>
            )}
            <div className="teshis__satir">
              <span>Oluşturucu</span>
              <b>{sonuc?.gpu ?? "ölçülemedi"}</b>
            </div>
            <div className="teshis__satir">
              <span>Kod çözme</span>
              <b>{sonuc ? (sonuc.gpuYazilim ? "yazılım" : "donanım") : "…"}</b>
            </div>
            <p className="ayar__not">
              Bu satır sayfaların <b>çizimi</b> için; video kod çözmenin
              donanımda olup olmadığı aşağıdaki tabloda codec başına duruyor.
            </p>
          </section>

          <section className="ayarlar__bolum">
            <h3>Codec</h3>
            <p className="ayar__not">
              <code>MediaCapabilities</code> cevabı. <b>donanım</b> = tarayıcı
              bu codec için verimli çözüm bildiriyor, <b>yazılım</b> = açılıyor
              ama işlemciden yiyor. HEVC'nin yokluğu web'de nadiren sorun;
              yerel dosya Muiply'a devrediliyor (`docs/Medya.md`).
            </p>
            {(sonuc?.codecler ?? []).map((c) => (
              <div className="teshis__satir" key={c.ad}>
                <span>
                  {c.ad} <small>{c.tur === "video" ? "video" : "ses"}</small>
                </span>
                <b className={c.durum === "yok" ? "teshis__yok" : undefined}>
                  {c.durum ? CODEC_METNI[c.durum] : "sorulamadı"}
                </b>
              </div>
            ))}
          </section>

          <section className="ayarlar__bolum">
            <h3>DRM</h3>
            <p className="ayar__not">
              Crunchyroll, Netflix ve Disney+ için gereken anahtar sistemleri.
              Burada <b>bulundu</b> yazması bir bölümün baştan sona oynayacağı
              anlamına <b>gelmiyor</b> — Faz 0/R2 kabul kriteri hâlâ gerçek bir
              bölüm (`docs/Roadmap.md`).
            </p>
            {(sonuc?.drm ?? []).map((d) => (
              <div className="teshis__satir" key={d.ad}>
                <span>{d.ad}</span>
                <b className={d.var === false ? "teshis__yok" : undefined}>
                  {d.var === null ? "sorulamadı" : d.var ? "bulundu" : "bulunamadı"}
                </b>
              </div>
            ))}
          </section>

          <section className="ayarlar__bolum">
            <h3>Motor</h3>
            <div className="teshis__satir">
              <span>WebView2 Runtime</span>
              <b>{sonuc?.webview2 ?? "ölçülemedi"}</b>
            </div>
            <div className="teshis__satir">
              <span>Muiren</span>
              <b>{sonuc?.muiren ?? "—"}</b>
            </div>
            {/* Yetenekler eksikse bu bir hata değil, eski bir Runtime
                (`docs/Setup.md`). Panel bunu arıza gibi göstermiyor. */}
            <div className="teshis__satir">
              <span>Askıya alma</span>
              <b>{yetenekler.askiyaAlma ? "var" : "yok"}</b>
            </div>
            <div className="teshis__satir">
              <span>Bellek hedefi</span>
              <b>{yetenekler.bellekHedefi ? "var" : "yok"}</b>
            </div>
            <div className="teshis__satir">
              <span>Süreç → sekme eşlemesi</span>
              <b>{yetenekler.surecBilgisi ? "var" : "yok"}</b>
            </div>
            <div className="teshis__satir">
              <span>İndirme olayı</span>
              <b>{yetenekler.indirmeOlayi ? "var" : "yok"}</b>
            </div>
            <div className="teshis__satir">
              <span>Favicon</span>
              <b>{yetenekler.favicon ? "var" : "yok"}</b>
            </div>
          </section>

          <section className="ayarlar__bolum">
            <h3>Ölçüm raporu</h3>
            <p className="ayar__not">
              <code>docs/olcumler/</code> altındaki her raporun başında duran
              tablo. Sürüm alanları boş bırakılmıyor: tarayıcının bilemediği
              satırlar <code>?</code> ile işaretli ve elle dolduruluyor.
            </p>
            <pre className="teshis__rapor">{rapor()}</pre>
            <div className="teshis__dugmeler">
              <button type="button" className="dugme" onClick={kopyala}>
                {kopyalandi ? "Kopyalandı" : "Panoya kopyala"}
              </button>
              <button type="button" className="dugme" onClick={tazele}>
                Yeniden ölç
              </button>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
