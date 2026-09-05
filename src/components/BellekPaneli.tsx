/**
 * Bellek paneli — **projenin vitrini** (`docs/Frontend.md`).
 *
 * Kullanıcının "bu program ne işe yarıyor" sorusuna bir bakışta cevap veren
 * yer. Üstte tek büyük rakam, altında durum dağılımı ve sistem baskısı.
 *
 * İki kural burada özellikle geçerli:
 *
 * - **Uyuyan sekmeler uyandırılmıyor** (CLAUDE.md #5). Bu panel yalnız
 *   backend'in gönderdiği özeti ve sekme listesini çiziyor; hiçbir sekmeye
 *   dokunmuyor. Panelin açık olması bellek tüketimini artırmamalı — yoksa
 *   bellek panelinin kendisi ironik biçimde bellek harcayan şey olur.
 * - **Yaklaşık değer kesin gibi gösterilmiyor** (`docs/Bellek.md`).
 *   `olcumYaklasik` true ise rakamın başında `~` ve yanında sebebi var.
 *
 * Burada karar yok: "hepsini uyut" düğmesi bile korumalıları backend'de
 * eliyor (`memory/esik.rs`), arayüz yalnız komutu gönderiyor.
 */

import { Uyku, Yonga } from "./Ikonlar";
import { useGecikme } from "../hooks/useGecikme";
import { bellek, durumOzeti, sure } from "../lib/bicim";
import type {
  BellekOzeti,
  Dagilim,
  SekmeBellegi,
  SekmeId,
  SekmeOzeti,
} from "../ipc/tipler";

interface Ozellik {
  ozet: BellekOzeti | null;
  sekmeler: SekmeOzeti[];
  onHepsiniUyut: () => void;
  onEtkinlestir: (id: SekmeId) => void;
}

/** Baskı seviyesinin Türkçe karşılığı ve çubuk oranı. */
const BASKI = {
  dusuk: { ad: "rahat", sinif: "ok" },
  orta: { ad: "orta", sinif: "uyari" },
  yuksek: { ad: "yüksek", sinif: "uyari" },
  kritik: { ad: "kritik", sinif: "hata" },
} as const;

/**
 * Sekme başına rakamın ipucu.
 *
 * İki farklı belirsizlik var ve karıştırılmıyorlar: **paylaşımlı** rakam
 * bugün ölçüldü ama birden çok sekmeye bölündü, **bayat** rakam hiç
 * bölünmedi ama bugüne ait değil. Tek bir "~" ikisini de aynı kefeye koyar
 * ve kullanıcı hangi sayıya ne kadar güveneceğini bilemez.
 */
function bellekIpucu(v: SekmeBellegi): string {
  const satirlar: string[] = [];
  if (v.bayat) {
    satirlar.push(
      "Son bilinen değer: sekmenin bu turda ölçülecek bir süreci yoktu. " +
        "Ölçmek için uyandırılmadı.",
    );
  }
  if (v.paylasimli) {
    satirlar.push(
      "Render süreci başka sekmelerle paylaşılıyor; rakam eşit bölüştürüldü.",
    );
  }
  return satirlar.join("\n");
}

/** Sekmenin durumunun okunur karşılığı. */
function durumAdi(s: SekmeOzeti): string {
  switch (s.durum) {
    case "uyuyan":
      return "uykuda";
    case "atilmis":
      return "bellekten atıldı";
    case "etkin":
      return "etkin";
    default:
      return "arka planda";
  }
}

/**
 * Uyanma dağılımının tek satırı.
 *
 * `dagilim` null ise satır **çizilmiyor değil, "henüz ölçülmedi" diyor**:
 * satırı hiç göstermemek, ölçümün var olduğunu ama sonucun kötü olduğunu
 * düşündürürdü. Ölçülmemiş olmak bir sonuç değil ve öyle görünmemeli.
 *
 * Hedefi geçen medyan uyarı sınıfı alıyor. Kırmızı değil: hedef bir kabul
 * kriteri, bir hata değil — makinesi yavaş olan kullanıcıya tarayıcısı
 * bozukmuş gibi görünmemeli.
 */
function GecikmeSatiri({
  ad,
  dagilim,
  hedefMs,
  ipucu,
}: {
  ad: string;
  dagilim: Dagilim | null;
  hedefMs: number;
  ipucu: string;
}) {
  if (!dagilim) {
    return (
      <div className="bellek__satir" title={ipucu}>
        <span>{ad}</span>
        <b className="bellek__olculmedi">henüz ölçülmedi</b>
      </div>
    );
  }
  const asti = dagilim.medyanMs > hedefMs;
  return (
    <div
      className="bellek__satir"
      title={`${ipucu}\n${dagilim.n} örnek · p95 ${dagilim.p95Ms} ms · en kötü ${dagilim.enKotuMs} ms\nhedef < ${hedefMs} ms (docs/Bellek.md)`}
    >
      <span>{ad}</span>
      <b className={asti ? "bellek__hedef--asti" : undefined}>
        {dagilim.medyanMs} ms
      </b>
    </div>
  );
}

export function BellekPaneli({
  ozet,
  sekmeler,
  onHepsiniUyut,
  onEtkinlestir,
}: Ozellik) {
  // Gecikme **burada** çekiliyor, `App.tsx`ten geçirilmiyor: yalnız bu panel
  // gösteriyor ve panel kapalıyken hiç sorulmuyor. Bellek panelinin kendisi
  // bir bellek/iş kalemi olmamalı (`GecmisPaneli` ile aynı kalıp).
  const { gecikme, sifirla } = useGecikme();

  if (!ozet) {
    // Sıfır göstermek yanlış olurdu: sıfır bir ölçüm sonucu değil, ölçümün
    // yokluğu. Gözcü ilk turunu koşunca olay gelecek.
    return (
      <div className="bellek bellek--bos">
        <Yonga boyut={20} />
        <p>Bellek ölçülüyor…</p>
        <p className="bellek__ipucu">
          Gözcü ilk turunu koştuğunda burası dolacak.
        </p>
      </div>
    );
  }

  const uyanik = ozet.etkin + ozet.arkaplan;
  const baski = BASKI[ozet.baski];
  const kullanilanYuzde =
    ozet.sistemToplamMb > 0
      ? Math.round(
          ((ozet.sistemToplamMb - ozet.sistemBosMb) / ozet.sistemToplamMb) * 100,
        )
      : 0;

  // Sekme başına rakamlar ayrı bir listede geliyor (`docs/IPC.md`): o liste
  // eşik kararının girdisi olan `SekmeOzeti`ye karıştırılmıyor. Birleştirme
  // `id` üzerinden ve burada.
  const sekmeBellekleri = new Map(ozet.sekmeMb.map((s) => [s.id, s]));

  // Uyuyanlar ve atılmışlar önce: panelin asıl anlattığı şey onlar. İçlerinde
  // en uzun süredir dokunulmamış olan başta.
  const sirali = [...sekmeler].sort((a, b) => {
    const agirlik = (s: SekmeOzeti) =>
      s.durum === "atilmis" ? 0 : s.durum === "uyuyan" ? 1 : 2;
    return agirlik(a) - agirlik(b) || b.bostaSn - a.bostaSn;
  });

  return (
    <div className="bellek">
      <div className="bellek__vitrin">
        <span className="bellek__rakam">
          {bellek(ozet.tahminiKazancMb, ozet.olcumYaklasik)}
        </span>
        <span className="bellek__etiket">şu an tasarruf ediliyor</span>
        {ozet.olcumYaklasik && (
          <span className="bellek__ipucu">
            {sekmeBellekleri.size > 0
              ? "Kısmen yaklaşık: bazı sekmeler uyanıkken hiç ölçülmedi, " +
                "onlar için ortalama kullanılıyor."
              : "Yaklaşık: WebView2 süreçlerini sekmelere eşleyemiyoruz, " +
                "rakam uyanık sekmelerin ortalamasından hesaplanıyor."}
          </span>
        )}
      </div>

      <p className="bellek__dagilim">
        {durumOzeti(uyanik, ozet.uyuyan, ozet.atilmis) || "sekme yok"}
      </p>

      <div className="bellek__olcum">
        <div className="bellek__satir">
          <span>Sayfalar</span>
          <b>{bellek(ozet.toplamMb, ozet.olcumYaklasik)}</b>
        </div>
        {/* Sekme başına rakamların toplamı `toplamMb`ye eşit çıkmıyor;
            farkı söylemeyen bir panel kullanıcıya ölçümün bozuk olduğunu
            düşündürür. */}
        {ozet.ortakMb > 0 && (
          <div
            className="bellek__satir"
            title="Sekmeye düşmeyen WebView2 süreçleri: tarayıcı, GPU, ağ, yardımcılar ve Muiren arayüzünün kendi render süreci. Sekme kapatarak azalmıyor."
          >
            <span>Ortak</span>
            <b>{bellek(ozet.ortakMb)}</b>
          </div>
        )}
        <div className="bellek__satir">
          <span>Kabuk</span>
          <b>{bellek(ozet.kabukMb)}</b>
        </div>
        <div className="bellek__satir">
          {/* Faz 0/R4'ün ölçüm aracı: aynı siteden 10 sekme açınca bu sayı
              10'da kalıyorsa `--process-per-site` geçmemiş demektir
              (`docs/Setup.md`). */}
          <span>WebView2 süreci</span>
          <b>{ozet.surecSayisi}</b>
        </div>
      </div>

      {/* Kabul tablosunun iki satırı (`docs/Bellek.md`). Panelde duruyorlar
          çünkü "hızlı olduğunu hissediyorum" kabul edilmiyor ve ölçüm ancak
          görünürse yapılıyor. */}
      {gecikme && (
        <div className="bellek__gecikme">
          <GecikmeSatiri
            ad="Uykudan dönüş"
            dagilim={gecikme.uyuyan}
            hedefMs={gecikme.hedefUyuyanMs}
            ipucu="Tıklamadan sayfanın etkileşime hazır olmasına kadar geçen süre (medyan)."
          />
          <GecikmeSatiri
            ad="Atılmıştan dönüş"
            dagilim={gecikme.atilmis}
            hedefMs={gecikme.hedefAtilmisMs}
            ipucu="Tıklamadan yüklemenin bitmesine kadar (medyan). İlk boyanın üst sınırı; ağ süresi dahil."
          />
          {(gecikme.uyuyan || gecikme.atilmis) && (
            <button
              type="button"
              className="dugme bellek__sifirla"
              onClick={sifirla}
              title="Ölçüm oturumuna temiz başlamak için dağılımı sıfırlar"
            >
              Ölçümü sıfırla
            </button>
          )}
        </div>
      )}

      <div className="bellek__baski">
        <div className="bellek__satir">
          <span>Sistem belleği</span>
          <b>
            {bellek(ozet.sistemToplamMb - ozet.sistemBosMb)} /{" "}
            {bellek(ozet.sistemToplamMb)}
          </b>
        </div>
        <div
          className="bellek__cubuk"
          role="meter"
          aria-valuenow={kullanilanYuzde}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label="Sistem bellek kullanımı"
        >
          <span
            className={`bellek__dolu bellek__dolu--${baski.sinif}`}
            style={{ width: `${kullanilanYuzde}%` }}
          />
        </div>
        <span className="bellek__ipucu">Baskı: {baski.ad}</span>
      </div>

      <button
        type="button"
        className="dugme dugme--birincil bellek__uyut"
        onClick={onHepsiniUyut}
        disabled={uyanik <= 1}
        title="Korumalı olmayan bütün sekmeleri uyut (Ctrl+Alt+Z)"
      >
        <Uyku boyut={14} />
        Hepsini uyut
      </button>

      <ul className="bellek__liste">
        {sirali.map((s) => {
          const olcum = sekmeBellekleri.get(s.id);
          return (
            <li key={s.id}>
              <button
                type="button"
                className={`bellek__oge bellek__oge--${s.durum}`}
                onClick={() => onEtkinlestir(s.id)}
                title={`${s.url}\n${durumAdi(s)}`}
              >
                <span className="bellek__ad">
                  {s.gorunenAd || "Yeni sekme"}
                </span>
                <span className="bellek__yan">
                  {/* Ölçümü olmayan sekme için sıfır **yazılmıyor**: sıfır bir
                      ölçüm sonucu değil, ölçümün yokluğu (panelin boş
                      durumuyla aynı kural). */}
                  {olcum && (
                    <span
                      className={
                        olcum.bayat ? "bellek__mb bellek__mb--bayat" : "bellek__mb"
                      }
                      title={bellekIpucu(olcum)}
                    >
                      {bellek(olcum.mb, olcum.paylasimli)}
                    </span>
                  )}
                  {(s.durum === "uyuyan" || s.durum === "atilmis") && (
                    <span className="rozet rozet--uyku">{durumAdi(s)}</span>
                  )}
                  <span className="bellek__bosta">{sure(s.bostaSn)}</span>
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
