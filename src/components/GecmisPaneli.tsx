/**
 * Geçmiş paneli — yan panelin üçüncü sekmesi (`docs/Frontend.md`).
 *
 * Arama **yerelde** koşuyor ve Türkçe küçültme backend'de yapılıyor:
 * `"İSTANBUL"` kaydı `"istanbul"` sorgusuyla eşleşiyor (CLAUDE.md #10,
 * `docs/Depolama.md`). Arayüz sorguyu olduğu gibi gönderiyor; küçültmeyi
 * burada tekrar etmek iki farklı kural demek olurdu.
 *
 * "Temizle" düğmesi yalnız geçmişi siliyor. Yer imleri bu ekranda **yok**:
 * onlar kullanıcının kendi ürettiği içerik ve bir temizleme düğmesiyle
 * kaybolmamalı (`docs/Depolama.md`).
 */

import { useCallback, useEffect, useState } from "react";

import { Ara, Kapat, Saat } from "./Ikonlar";
import { gecmisAra, gecmisSil, gecmisTemizle } from "../ipc";
import type { Aralik, GecmisKaydi } from "../ipc/tipler";

const ARALIKLAR: { deger: Aralik; ad: string }[] = [
  { deger: "sonSaat", ad: "Son saat" },
  { deger: "sonGun", ad: "Son gün" },
  { deger: "sonHafta", ad: "Son hafta" },
  { deger: "hepsi", ad: "Hepsi" },
];

/** Unix damgasını okunur tarihe çevirir. */
function tarih(saniye: number): string {
  return new Date(saniye * 1000).toLocaleString("tr-TR", {
    day: "2-digit",
    month: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

interface Ozellik {
  onGezin: (url: string) => void;
}

export function GecmisPaneli({ onGezin }: Ozellik) {
  const [sorgu, ayarlaSorgu] = useState("");
  const [kayitlar, ayarlaKayitlar] = useState<GecmisKaydi[]>([]);
  const [temizleAcik, ayarlaTemizle] = useState(false);

  const tazele = useCallback((s: string) => {
    void gecmisAra(s, 200)
      .then(ayarlaKayitlar)
      .catch(() => ayarlaKayitlar([]));
  }, []);

  useEffect(() => {
    // Aynı gecikme mantığı `useOneriler` içinde de var: her tuşta bir IPC
    // turu + SQLite sorgusu çalıştırmanın karşılığı yok.
    const zaman = window.setTimeout(() => tazele(sorgu), 150);
    return () => window.clearTimeout(zaman);
  }, [sorgu, tazele]);

  return (
    <div className="gecmis">
      <div className="gecmis__ara">
        <Ara boyut={13} />
        <input
          className="gecmis__girdi"
          value={sorgu}
          placeholder="Geçmişte ara"
          spellCheck={false}
          onChange={(e) => ayarlaSorgu(e.target.value)}
        />
      </div>

      <div className="gecmis__araclar">
        <button
          type="button"
          className="dugme gecmis__temizle"
          onClick={() => ayarlaTemizle((a) => !a)}
        >
          Geçmişi temizle
        </button>
      </div>

      {temizleAcik && (
        <div className="gecmis__aralik">
          {ARALIKLAR.map((a) => (
            <button
              key={a.deger}
              type="button"
              className="dugme"
              onClick={() => {
                void gecmisTemizle(a.deger).then(() => {
                  ayarlaTemizle(false);
                  tazele(sorgu);
                });
              }}
            >
              {a.ad}
            </button>
          ))}
        </div>
      )}

      {kayitlar.length === 0 ? (
        <p className="yan__bos">
          {sorgu.trim() === "" ? "Geçmiş boş." : "Eşleşen kayıt yok."}
        </p>
      ) : (
        <ul className="gecmis__liste">
          {kayitlar.map((k) => (
            <li key={k.id} className="gecmis__satir">
              <button
                type="button"
                className="gecmis__oge"
                onClick={() => onGezin(k.url)}
                title={`${k.url}\n${tarih(k.ziyaret)} · ${k.sayac} ziyaret`}
              >
                <Saat boyut={12} />
                <span className="gecmis__ad">{k.baslik || k.alan}</span>
                <span className="gecmis__zaman">{tarih(k.ziyaret)}</span>
              </button>
              <button
                type="button"
                className="dikey__dugme"
                aria-label="Bu kaydı sil"
                onClick={() => {
                  void gecmisSil(k.id).then(() => tazele(sorgu));
                }}
              >
                <Kapat boyut={11} />
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
