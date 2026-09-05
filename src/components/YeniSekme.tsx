/**
 * `muiren://yeni` — kabuğun kendi sayfası.
 *
 * Ağ isteği yok, reklam yok, öneri akışı yok (`docs/Sekmeler.md`). Sayfanın
 * webview'i de **yok**: kabuk çiziyor, dolayısıyla yeni sekmenin render
 * maliyeti sıfır. "Boş tarayıcı < 150 MB" hedefi buna dayanıyor
 * (`docs/Bellek.md`).
 *
 * ## Ne var, ne yok
 *
 * Uzun süre yalnız bir arama kutusu vardı. Bir tarayıcının en çok açılan
 * sayfası bu ve boş durması iki şeyi birden kaçırıyordu:
 *
 * - **Gitmek istenen yer.** Kullanıcı her seferinde adresi yazıyordu; oysa
 *   nereye sık gittiği zaten diskte (`history/depo.rs`).
 * - **Programın ne yaptığı.** Bellek politikası çalışıyor ama kullanıcı
 *   sonucu görmüyordu; alttaki kart o boşluğu dolduruyor ve rakamı
 *   `BellekOzeti` içinden alıyor — **hiçbir sekme uyandırılmadan**
 *   (CLAUDE.md #5).
 *
 * İkisinin de kaynağı yerel: `useYeniSekme` yalnız SQLite'a bakıyor, ağa
 * çıkmıyor (karar #5).
 *
 * **Gizli sekmede geçmiş listesi çizilmiyor.** Gizli sekme açan kullanıcının
 * ekranında sık gittiği siteleri sıralamak, gizliliğin tam da kaçındığı
 * şeyi yapmak olurdu: omuz üstünden bakan biri listeyi görüyor.
 */

import { useState } from "react";

import { Ara, Saat, Uyku, Yildiz, Yonga } from "./Ikonlar";
import { useYeniSekme } from "../hooks/useYeniSekme";
import { bellek, durumOzeti } from "../lib/bicim";
import type { BellekOzeti, GecmisKaydi, YerImi } from "../ipc/tipler";

interface Ozellik {
  onGezin: (girdi: string) => void;
  /** Motor derlemede yoksa arama kutusu ölü düğme olurdu; onun yerine sebebi
   *  yazılıyor. */
  motorVar: boolean;
  /** Bellek özeti; `null` ise gözcü henüz ilk turunu koşmamış. */
  ozet: BellekOzeti | null;
  /** Etkin sekme gizli mi. Gizliyse geçmiş listeleri hiç çizilmiyor. */
  gizli: boolean;
}

/**
 * Kutunun rengi — alan adından türetilmiş, **sabit renk yazılmadan**.
 *
 * Renkler `tabs::GrupRengi` enum'ının CSS karşılıkları (`renk--*` sınıfları,
 * `styles.css`). Bileşende renk sabiti yok (CLAUDE.md #9); burada seçilen
 * şey bir renk değil, var olan bir listedeki sıra. Aynı alan adı hep aynı
 * rengi alıyor: kullanıcı kutunun yerini rengiyle hatırlıyor.
 */
const RENKLER = ["teal", "mor", "kehribar", "kirmizi", "yesil", "mavi", "gri"] as const;

function renkSinifi(alan: string): string {
  let n = 0;
  for (const k of alan) n = (n + k.charCodeAt(0)) % 9973;
  return `renk--${RENKLER[n % RENKLER.length]}`;
}

/**
 * Kutunun harfi. Türkçe büyültme (CLAUDE.md #10'un simetriği): düz
 * `toUpperCase()` "istanbul" için "I" veriyor, doğrusu "İ".
 */
function basHarf(metin: string): string {
  const harf = [...metin.trim()][0];
  return harf ? harf.toLocaleUpperCase("tr") : "?";
}

/** Sık gidilenler ızgarası. */
function SikGidilenler({
  kayitlar,
  onGezin,
}: {
  kayitlar: GecmisKaydi[];
  onGezin: (url: string) => void;
}) {
  return (
    <section className="yeni__bolum">
      <h2 className="yeni__baslik">
        <Saat boyut={12} /> Sık gidilenler
      </h2>
      <ul className="kutular">
        {kayitlar.map((k) => (
          <li key={k.id}>
            <button
              type="button"
              className={`kutu ${renkSinifi(k.alan)}`}
              onClick={() => onGezin(k.url)}
              // Başlık ve adres sayfadan geldi; backend temizledi, React
              // kaçışlıyor (CLAUDE.md #8).
              title={`${k.baslik || k.alan}\n${k.url}`}
            >
              <span className="kutu__harf" aria-hidden>
                {basHarf(k.alan)}
              </span>
              <span className="kutu__ad">{k.baslik || k.alan}</span>
              <span className="kutu__alan">{k.alan}</span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

/** Yer imi satırı — ızgara değil, tek satırlık şerit. */
function YerImleri({
  kayitlar,
  onGezin,
}: {
  kayitlar: YerImi[];
  onGezin: (url: string) => void;
}) {
  return (
    <section className="yeni__bolum">
      <h2 className="yeni__baslik">
        <Yildiz boyut={12} dolu /> Yer imleri
      </h2>
      <ul className="yeni__serit">
        {kayitlar.map((y) => (
          <li key={y.id}>
            <button
              type="button"
              className="yeni__yerimi"
              onClick={() => onGezin(y.url)}
              title={y.url}
            >
              {y.baslik || y.url}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

/**
 * Bellek kartı — politikanın **görünür** olduğu yer.
 *
 * Özet `null` iken çizilmiyor: gözcü ilk turunu koşmadan sıfır göstermek,
 * ölçümün yokluğunu bir ölçüm sonucu gibi sunmak olurdu (`useBellek` ile
 * aynı gerekçe).
 */
function BellekKarti({ ozet }: { ozet: BellekOzeti }) {
  const uyanik = ozet.etkin + ozet.arkaplan;
  const dagilim = durumOzeti(uyanik, ozet.uyuyan, ozet.atilmis);
  if (dagilim === "") return null;

  return (
    <div className="yeni__kart">
      <span className="yeni__kartIkon" aria-hidden>
        <Yonga boyut={15} />
      </span>
      <div className="yeni__kartMetin">
        <b>{dagilim}</b>
        <span>
          Sayfa katmanı {bellek(ozet.toplamMb, ozet.olcumYaklasik)}
          {ozet.tahminiKazancMb > 0 && (
            <>
              {" · uyutma kazancı "}
              <em>{bellek(ozet.tahminiKazancMb, ozet.olcumYaklasik)}</em>
            </>
          )}
        </span>
      </div>
      {ozet.uyuyan + ozet.atilmis > 0 && (
        <span className="rozet rozet--uyku" title="Uyuyan ve atılmış sekmeler">
          <Uyku boyut={11} />
          {ozet.uyuyan + ozet.atilmis}
        </span>
      )}
    </div>
  );
}

export function YeniSekme({ onGezin, motorVar, ozet, gizli }: Ozellik) {
  const [metin, ayarla] = useState("");
  // Motorsuz derlemede gidilecek bir yer yok; listeleri çekmenin de anlamı
  // yok. Gizli sekmede liste bilerek çekilmiyor (bileşen açıklaması).
  const veri = useYeniSekme(motorVar && !gizli);

  return (
    <div className="yeni">
      <div className="yeni__ust">
        <div className="yeni__marka">
          <svg viewBox="0 0 32 32" width="44" height="44" aria-hidden>
            <circle
              cx="16"
              cy="16"
              r="10.6"
              fill="none"
              stroke="var(--accent)"
              strokeWidth="2.4"
            />
            <path
              d="M16 24.4 19.2 16 12.8 16z"
              fill="none"
              stroke="var(--accent)"
              strokeWidth="2"
              strokeLinejoin="round"
            />
            <path d="M16 7.6 19.2 16 12.8 16z" fill="var(--accent)" />
          </svg>
          <h1>{gizli ? "Gizli sekme" : "Muiren"}</h1>
        </div>

        {motorVar ? (
          <form
            className="yeni__form"
            onSubmit={(e) => {
              e.preventDefault();
              if (metin.trim() !== "") onGezin(metin);
            }}
          >
            <span className="yeni__ikon">
              <Ara />
            </span>
            <input
              className="yeni__girdi"
              value={metin}
              autoFocus
              spellCheck={false}
              autoComplete="off"
              placeholder="Adres ya da arama"
              onChange={(e) => ayarla(e.target.value)}
            />
          </form>
        ) : (
          <p className="yeni__not">
            Bu derlemede sayfa motoru yok (<code>--no-default-features</code>).
            Sekme yönetimi ve oturum çalışıyor, sayfa açılmıyor.
          </p>
        )}
      </div>

      {gizli ? (
        <p className="yeni__alt">
          Bu sekme geçmişe, oturum dosyasına ve favicon deposuna yazmıyor.
          Çerezler ve önbellek sekme kapanınca kayboluyor. İndirdiğin dosyalar
          ve yer imlerin kalıcı.
        </p>
      ) : (
        <>
          {veri.sikGidilenler.length > 0 && (
            <SikGidilenler kayitlar={veri.sikGidilenler} onGezin={onGezin} />
          )}
          {veri.yerImleri.length > 0 && (
            <YerImleri kayitlar={veri.yerImleri} onGezin={onGezin} />
          )}

          {ozet && <BellekKarti ozet={ozet} />}

          {veri.yuklendi &&
            veri.sikGidilenler.length === 0 &&
            veri.yerImleri.length === 0 && (
              <p className="yeni__alt">
                Boştaki sekmeler uyutuluyor ve atılıyor. Sekme çubuğundaki
                yerleri, başlıkları ve adresleri duruyor.
              </p>
            )}
        </>
      )}
    </div>
  );
}
