/**
 * Sekme şeridi.
 *
 * Hedef kitle 50+ sekme açıyor. Klasik sekme çubuğu 20'de kullanılamaz hâle
 * geliyor; Muiren'in cevabı (`docs/Sekmeler.md`):
 *
 * - Sekme genişliği bir alt sınıra kadar daralıyor (favicon + ses göstergesi
 *   sığacak kadar), sonra **kaydırma** başlıyor. 12 piksellik sekme kimseye
 *   yaramıyor.
 * - Uyuyan sekme soluk, atılmış sekme daha soluk + kesikli kenarlık. Bu bir
 *   tanıtım öğesi değil, **güven** öğesi: kullanıcı neyin bellekte olduğunu
 *   bir bakışta görüyor.
 * - Solma **animasyonsuz**. 50 sekmede aynı anda 30 geçiş animasyonu tam da
 *   kaçındığımız türden bir maliyet (`docs/Frontend.md`).
 *
 * Burada karar yok: sıra, girinti ve durum backend'den geldiği gibi çiziliyor.
 * Sürükle-bırak bile yalnız **hedef sırayı** bildiriyor; taşımayı `agac::tasi`
 * yapıyor ve çocukları birlikte götürüyor (CLAUDE.md #2).
 *
 * Şeridin boş alanı pencerenin sürükleme bölgesi: pencere kenarlıksız ve
 * başlık çubuğunun işini bu satır görüyor (`docs/Frontend.md`).
 */

import { Fragment, useEffect, useRef, useState } from "react";

import {
  Arti,
  Ara,
  Disli,
  Gizli,
  Kapat,
  Ok,
  Panel,
  Sabit,
  Ses,
  Sessiz,
} from "./Ikonlar";
import { PencereDugmeleri } from "./PencereDugmeleri";
import { SekmeIkonu } from "./SekmeIkonu";
import { useFavicon } from "../hooks/useFavicon";
import type { Grup, GrupId, SekmeId, SekmeOzeti } from "../ipc/tipler";

interface Ozellik {
  sekmeler: SekmeOzeti[];
  gruplar: Grup[];
  onEtkinlestir: (id: SekmeId) => void;
  onKapat: (id: SekmeId) => void;
  onSessizeAl: (id: SekmeId, sessiz: boolean) => void;
  onYeni: () => void;
  onTasi: (id: SekmeId, hedef: number) => void;
  onArama: () => void;
  onYanPanel: () => void;
  yanPanelAcik: boolean;
  onAyarlar: () => void;
  /** Grubu katla/aç. Karar backend'de saklanıyor; burada yalnız tetik. */
  onGrupKatla: (id: GrupId, katli: boolean) => void;
  onGizliSekme: () => void;
  /** Köprü menüsü; kurulu kardeş yoksa `null` geliyor ve çizilmiyor. */
  kopruMenusu?: React.ReactNode;
}

/** Grup başlığının ipucu. */
function grupIpucu(g: Grup): string {
  const ad = g.ad || "Adsız grup";
  // Grubun kendi eşiği varsa yazılıyor: sekmelerinin neden erken uyuduğunu
  // görmeyen kullanıcı özelliği kapatıyor (`docs/Bellek.md`).
  return g.uykuEsigiSn !== null
    ? `${ad} — uyku eşiği ${Math.round(g.uykuEsigiSn / 60)} dk`
    : ad;
}

/** Bir sekmenin adres/durum ipucu. */
function ipucu(s: SekmeOzeti): string {
  const durum =
    s.durum === "uyuyan"
      ? "uykuda"
      : s.durum === "atilmis"
        ? "bellekten atıldı"
        : s.durum === "arkaplan"
          ? "arka planda"
          : "etkin";
  // Neden uyumadığını göstermek, kullanıcının özelliği kapatmasını
  // engelliyor (`docs/Bellek.md`, `sebep` alanıyla aynı gerekçe).
  const koruma = s.sabit
    ? "\nsabitlenmiş — uyutulmuyor"
    : s.uyutmaIstisnasi
      ? "\nuyutma istisnası"
      : s.sesCaliyor && !s.sessiz
        ? "\nses çalıyor — uyutulmuyor"
        : s.tamEkran
          ? "\ntam ekran — uyutulmuyor"
          : s.formDolu
            ? "\nformu dolu — atılmıyor"
            : "";
  // Gizli sekmede adres ipucunda da görünüyor: kullanıcının hangi sekmenin
  // gizli olduğunu görmesi şart (`tabs::SekmeOzeti::gizli`).
  const gizli = s.gizli ? "\ngizli sekme — geçmişe yazılmıyor" : "";
  // Grup eşiği genel ayardan farklıysa söyleniyor: sekmesinin neden erken
  // uyuduğunu bilmeyen kullanıcı özelliği kapatıyor (`docs/Bellek.md`).
  const grupEsigi =
    s.grupUykuEsigiSn !== null
      ? `\ngrup uyku eşiği: ${Math.round(s.grupUykuEsigiSn / 60)} dk`
      : "";
  return `${s.gorunenAd || "Yeni sekme"}\n${s.url}\n${durum}${koruma}${gizli}${grupEsigi}`;
}

export function SekmeSeridi({
  sekmeler,
  gruplar,
  onEtkinlestir,
  onKapat,
  onSessizeAl,
  onYeni,
  onTasi,
  onArama,
  onYanPanel,
  yanPanelAcik,
  onAyarlar,
  onGrupKatla,
  onGizliSekme,
  kopruMenusu,
}: Ozellik) {
  // Sürüklenen sekme ve üstünde durulan sıra. Yalnız görsel geri bildirim
  // için; asıl taşıma bırakıldığında tek `sekme_tasi` çağrısıyla oluyor.
  const [suruklenen, ayarlaSuruklenen] = useState<SekmeId | null>(null);
  const [hedef, ayarlaHedef] = useState<number | null>(null);
  const favicon = useFavicon();

  /**
   * Klavye odağındaki sekme — **etkin sekme değil**.
   *
   * `role="tablist"` içinde her sekmeye `tabIndex=0` vermek, 50 sekmelik bir
   * şeritte Tab tuşunu 50 kez basmak demekti: klavyeyle adres çubuğuna
   * geçmek pratikte imkânsızdı. Doğrusu **dolaşan odak** (roving tabindex):
   * şerit tek bir durak, içinde ok tuşlarıyla geziliyor.
   *
   * Ok tuşları sekmeyi **etkinleştirmiyor**, yalnız odağı taşıyor
   * (WAI-ARIA'nın "elle etkinleştirme" kalıbı). Bu tercih burada mecburi:
   * otomatik etkinleştirme uyuyan bir sekmenin üstünden geçerken onu
   * uyandırırdı ve klavyeyle şeridi taramak projenin tek iddiasını
   * çökertirdi (CLAUDE.md #5).
   */
  const [odakId, ayarlaOdakId] = useState<SekmeId | null>(null);
  const ogeler = useRef(new Map<SekmeId, HTMLDivElement | null>());

  /** Katlanmış grubun sekmeleri çizilmiyor; odak da onlara gitmiyor. */
  const gezilebilir = sekmeler.filter((s) => !s.katli);
  const odakHedefi =
    gezilebilir.find((s) => s.id === odakId)?.id ??
    gezilebilir.find((s) => s.etkin)?.id ??
    gezilebilir[0]?.id ??
    null;

  /**
   * Etkin sekme **görünür** olmak zorunda.
   *
   * 50 sekmede şerit kaydırmaya geçiyor ve etkin sekme kolayca ekranın
   * dışında kalıyor: `Ctrl+Tab` ile gezinen ya da `Ctrl+T` ile yeni sekme
   * açan kullanıcı, açtığı sekmeyi göremiyordu. Kaydırma buradan yapılıyor
   * çünkü hangi sekmenin etkin olduğu **backend'den** geliyor: kapı
   * hangisiyse (kısayol, köprü, oturum geri yükleme) sonuç aynı.
   *
   * `smooth` DEĞİL: 50 sekmelik bir şeritte yumuşak kaydırma her sekme
   * değişiminde bir animasyon demek ve `docs/Frontend.md` bunu bilerek
   * dışarıda bırakıyor. Hareket azaltma isteği olan kullanıcı için de tek
   * doğru cevap bu.
   */
  const etkinId = sekmeler.find((s) => s.etkin && !s.katli)?.id ?? null;
  useEffect(() => {
    if (etkinId === null) return;
    ogeler.current.get(etkinId)?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [etkinId]);

  /** Odağı listede `adim` kadar kaydırır; uçlarda duruyor, sarmıyor. */
  const odagiTasi = (adim: number | "bas" | "son") => {
    if (gezilebilir.length === 0) return;
    const simdiki = gezilebilir.findIndex((s) => s.id === odakHedefi);
    const yeni =
      adim === "bas"
        ? 0
        : adim === "son"
          ? gezilebilir.length - 1
          : Math.min(Math.max(simdiki + adim, 0), gezilebilir.length - 1);
    const id = gezilebilir[yeni].id;
    ayarlaOdakId(id);
    ogeler.current.get(id)?.focus();
  };

  const grupBul = (id: GrupId | null) =>
    id === null ? null : (gruplar.find((g) => g.id === id) ?? null);

  /** Sekmenin favicon kimliğini `data:` adresine çeviriyor; yoksa `null`. */
  const faviconAdresi = (s: SekmeOzeti) => favicon(s.favicon);

  /**
   * Bu sıradaki sekme, grubunun **ilki** mi.
   *
   * Grup başlığı (renk + ad + katla düğmesi) yalnız ilkinin önüne
   * çiziliyor. Sıra backend'den geldiği gibi kullanılıyor: bir grubun
   * sekmeleri bitişik olmayabiliyor (kullanıcı araya başka bir sekme
   * sürüklemiş olabilir) ve o durumda başlık ikinci kez çıkıyor. Bunu
   * arayüzde "düzeltmek", backend'in sırasıyla ayrışmak olurdu
   * (CLAUDE.md #2).
   */
  const grubunIlki = (sira: number) => {
    const g = sekmeler[sira].grup;
    if (g === null) return false;
    return sira === 0 || sekmeler[sira - 1].grup !== g;
  };

  const birak = (sira: number) => {
    if (suruklenen !== null) {
      const kaynak = sekmeler.findIndex((s) => s.id === suruklenen);
      if (kaynak !== -1 && kaynak !== sira) onTasi(suruklenen, sira);
    }
    ayarlaSuruklenen(null);
    ayarlaHedef(null);
  };

  return (
    // `data-tauri-drag-region`: şeridin zemini pencereyi sürüklüyor. Çift
    // tıklama büyütüyor — Windows davranışı (`docs/Frontend.md`).
    <div className="serit" role="tablist" aria-label="Sekmeler" data-tauri-drag-region>
      <div className="serit__liste">
        {sekmeler.map((s, sira) => {
          const grup = grupBul(s.grup);
          const baslik =
            grup && grubunIlki(sira) ? (
              <button
                type="button"
                className={`grup renk--${grup.renk}`}
                aria-expanded={!grup.katli}
                title={grupIpucu(grup)}
                onClick={() => onGrupKatla(grup.id, !grup.katli)}
              >
                <Ok boyut={11} dondur={!grup.katli} />
                <span className="grup__ad">{grup.ad || "Adsız grup"}</span>
                {grup.katli && (
                  <span className="grup__sayi">
                    {sekmeler.filter((x) => x.grup === grup.id).length}
                  </span>
                )}
              </button>
            ) : null;

          return (
            <Fragment key={s.id}>
              {baslik}
              {/* Katlanmış grubun sekmeleri çizilmiyor — başlığı hariç.
                  Backend listeyi yine de TAM gönderiyor: katlama bir
                  görüntü tercihi ve "grubu aç" düğmesinin kaç sekme
                  olduğunu söylemesi gerekiyor. */}
              {!s.katli && (
                <div
                  role="tab"
                  ref={(el) => {
                    ogeler.current.set(s.id, el);
                  }}
                  tabIndex={s.id === odakHedefi ? 0 : -1}
                  aria-selected={s.etkin}
                  title={ipucu(s)}
                  draggable
                  className={[
                    "sekme",
                    `sekme--${s.durum}`,
                    s.etkin ? "sekme--etkin" : "",
                    s.sabit ? "sekme--sabit" : "",
                    s.gizli ? "sekme--gizli" : "",
                    grup ? `sekme--grupta renk--${grup.renk}` : "",
                    suruklenen === s.id ? "sekme--suruklenen" : "",
                    hedef === sira && suruklenen !== s.id ? "sekme--hedef" : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                  style={{ "--derinlik": s.derinlik } as React.CSSProperties}
                  onClick={() => onEtkinlestir(s.id)}
                  onFocus={() => ayarlaOdakId(s.id)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      onEtkinlestir(s.id);
                    } else if (e.key === "ArrowRight") {
                      e.preventDefault();
                      odagiTasi(1);
                    } else if (e.key === "ArrowLeft") {
                      e.preventDefault();
                      odagiTasi(-1);
                    } else if (e.key === "Home") {
                      e.preventDefault();
                      odagiTasi("bas");
                    } else if (e.key === "End") {
                      e.preventDefault();
                      odagiTasi("son");
                    } else if (e.key === "Delete" && !s.sabit) {
                      // Sabitlenmiş sekme kapatma düğmesi bile çizmiyor;
                      // klavyenin ondan fazlasını yapmaması gerekiyor.
                      e.preventDefault();
                      onKapat(s.id);
                    }
                  }}
                  onAuxClick={(e) => {
                    // Orta tık kapatıyor — tarayıcı alışkanlığı.
                    if (e.button === 1) {
                      e.preventDefault();
                      onKapat(s.id);
                    }
                  }}
                  onDragStart={(e) => {
                    ayarlaSuruklenen(s.id);
                    e.dataTransfer.effectAllowed = "move";
                    // Firefox sürüklemeyi ancak veri konulduğunda
                    // başlatıyor. Sekme kimliği DIŞARI çıkmasın diye boş
                    // dize yazılıyor: bu veri başka bir uygulamaya
                    // bırakılabilir.
                    e.dataTransfer.setData("text/plain", "");
                  }}
                  onDragOver={(e) => {
                    if (suruklenen === null) return;
                    e.preventDefault();
                    e.dataTransfer.dropEffect = "move";
                    ayarlaHedef(sira);
                  }}
                  onDrop={(e) => {
                    e.preventDefault();
                    birak(sira);
                  }}
                  onDragEnd={() => {
                    ayarlaSuruklenen(null);
                    ayarlaHedef(null);
                  }}
                >
                  {s.sabit && (
                    <span className="sekme__sabit" aria-label="Sabitlenmiş">
                      <Sabit boyut={12} />
                    </span>
                  )}

                  {s.gizli && (
                    <span className="sekme__gizli" aria-label="Gizli sekme">
                      <Gizli boyut={12} />
                    </span>
                  )}

                  {/* İkon + durum halkası tek öğe (`styles.css`, "Durum
                      halkası"): 44 piksele inmiş bir sekmede görünen tek şey
                      bu ve hem "hangi site" hem "bellekte mi" sorusunu
                      birlikte cevaplaması gerekiyor.

                      Favicon sayfadan gelen bir görsel ama **adres değil**:
                      backend baytları alıp diske yazdı, buraya bir `data:`
                      adresi olarak geliyor. Kabuk hiçbir uzak istek
                      yapmıyor (`src-tauri/src/favicon/mod.rs`). */}
                  <SekmeIkonu sekme={s} adres={faviconAdresi(s)} />

                  {/* Başlık sayfadan geliyor: backend temizledi, React
                      kaçışlıyor. `dangerouslySetInnerHTML` yok
                      (`docs/Sekmeler.md`). */}
                  <span className="sekme__ad">{s.gorunenAd || "Yeni sekme"}</span>

                  {(s.sesCaliyor || s.sessiz) && (
                    <button
                      type="button"
                      className="sekme__ses"
                      aria-label={s.sessiz ? "Sesi aç" : "Sesi kapat"}
                      onClick={(e) => {
                        e.stopPropagation();
                        onSessizeAl(s.id, !s.sessiz);
                      }}
                    >
                      {s.sessiz ? <Sessiz boyut={13} /> : <Ses boyut={13} />}
                    </button>
                  )}

                  {!s.sabit && (
                    <button
                      type="button"
                      className="sekme__kapat"
                      aria-label="Sekmeyi kapat"
                      onClick={(e) => {
                        e.stopPropagation();
                        onKapat(s.id);
                      }}
                    >
                      <Kapat boyut={12} />
                    </button>
                  )}
                </div>
              )}
            </Fragment>
          );
        })}
      </div>

      <button
        type="button"
        className="dugme dugme--ikon serit__yeni"
        onClick={onYeni}
        title="Yeni sekme (Ctrl+T)"
      >
        <Arti boyut={14} />
        <span className="gorunmez">Yeni sekme</span>
      </button>

      <div className="serit__araclar">
        {kopruMenusu}
        <button
          type="button"
          className="dugme dugme--ikon"
          onClick={onGizliSekme}
          title="Gizli sekme (Ctrl+Shift+N)"
        >
          <Gizli boyut={14} />
          <span className="gorunmez">Gizli sekme</span>
        </button>
        <button
          type="button"
          className="dugme dugme--ikon"
          onClick={onArama}
          title="Sekmelerde ara (Ctrl+Shift+A)"
        >
          <Ara boyut={14} />
          <span className="gorunmez">Sekmelerde ara</span>
        </button>
        <button
          type="button"
          className={`dugme dugme--ikon ${yanPanelAcik ? "dugme--acik" : ""}`}
          onClick={onYanPanel}
          aria-pressed={yanPanelAcik}
          title="Yan panel (Ctrl+Shift+E)"
        >
          <Panel boyut={14} />
          <span className="gorunmez">Yan panel</span>
        </button>
        <button
          type="button"
          className="dugme dugme--ikon"
          onClick={onAyarlar}
          title="Ayarlar"
        >
          <Disli boyut={14} />
          <span className="gorunmez">Ayarlar</span>
        </button>
      </div>

      <PencereDugmeleri />
    </div>
  );
}
