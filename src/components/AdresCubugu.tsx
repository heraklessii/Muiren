/**
 * Adres çubuğu.
 *
 * Odaklanınca ham adresi düzenlenebilir metin olarak gösteriyor; odak
 * gidince [`gosterim`] parçalarını çiziyor: şema gizli, **kayıtlanabilir alan
 * adı vurgulu**, geri kalanı soluk.
 *
 * Vurgu bir güvenlik özelliği (`docs/Frontend.md`): `banka.com.saldirgan.net`
 * adresinde göze çarpması gereken `saldirgan.net`. Kararın tamamı
 * `lib/url.ts` içinde ve testli — bu bileşen yalnız çiziyor.
 *
 * ## Öneriler
 *
 * Kaynak **yalnız yerel geçmiş** (`useOneriler`). Arama motorunun canlı öneri
 * API'si **hiç** kullanılmayacak: her tuş vuruşunu bir sunucuya göndermek
 * telemetri demek ve `docs/Roadmap.md` karar #5 telemetriyi kapsam dışı
 * bırakıyor.
 */

import { useEffect, useRef, useState } from "react";

import {
  Geri,
  Ileri,
  Kalkan,
  Kilit,
  Saat,
  Uyari,
  Yenile,
  Durdur,
  Yildiz,
} from "./Ikonlar";
import { useOneriler } from "../hooks/useOneriler";
import { gosterim } from "../lib/url";
import type { EngelSebebi, SekmeOzeti } from "../ipc/tipler";

interface Ozellik {
  sekme: SekmeOzeti | undefined;
  onGezin: (girdi: string) => void;
  onGeri: () => void;
  onIleri: () => void;
  onYenile: () => void;
  onDurdur: () => void;
  odakSayaci: number;
  /** Bu adres yer imlerinde mi (`yer_imi_mi` komutunun cevabı). */
  yerImiId: number | null;
  onYerImi: () => void;
  /** Bu SAYFADA engellenen istek sayısı; 0 ise rozet çizilmiyor. */
  engelSayisi: number;
  /** Son engelin sebebi; rozetin ipucunda görünüyor. */
  engelSebebi: EngelSebebi | null;
  /** Bellek nabzı. Çubuğun sağ ucunda, yer imi yıldızından sonra. */
  nabiz?: React.ReactNode;
}

/** Engelleme rozetinin ipucu. Sessiz engelleme kabul edilmiyor. */
function engelIpucu(sayi: number, sebep: EngelSebebi | null): string {
  const ne =
    sebep === "popUp"
      ? "kendiliğinden açılan pencere"
      : sebep === "yonlendirme"
        ? "istenmeyen yönlendirme"
        : "filtre kuralı";
  // Türkçe ek üretilmiyor (CLAUDE.md #11): sayı ne olursa olsun kalıp aynı.
  return `Bu sayfada engellenen: ${sayi}\nson sebep: ${ne}`;
}

export function AdresCubugu({
  sekme,
  onGezin,
  onGeri,
  onIleri,
  onYenile,
  onDurdur,
  odakSayaci,
  yerImiId,
  onYerImi,
  engelSayisi,
  engelSebebi,
  nabiz,
}: Ozellik) {
  const [taslak, ayarlaTaslak] = useState<string | null>(null);
  const [secili, ayarlaSecili] = useState(-1);
  const girdiRef = useRef<HTMLInputElement>(null);
  /** `Ctrl+L` odağı istedi ama girdi henüz boyanmadı. */
  const odakBekliyor = useRef(false);

  const adres = sekme?.url ?? "";
  const g = gosterim(adres);
  const oneriler = useOneriler(taslak);
  const sekmeId = sekme?.id ?? null;

  /**
   * Sekme değişince taslak **düşüyor**.
   *
   * Taslak o anki sekmenin adresinden doğdu; sekme değiştiğinde ekranda
   * duran metin artık başka bir sekmeye ait. Enter'a basıldığında yeni
   * sekme, eski sekmenin adresine gidiyordu — kullanıcının yazmadığı bir
   * adrese. Kısayolla sekme değiştirmek (Ctrl+Tab) adres çubuğu açıkken
   * yapılabildiği için yol da uzak değil.
   */
  useEffect(() => {
    ayarlaTaslak(null);
    ayarlaSecili(-1);
  }, [sekmeId]);

  // `Ctrl+L` sayacı arttığında düzenleme kipine geç ve odağı **iste**.
  useEffect(() => {
    if (odakSayaci === 0) return;
    odakBekliyor.current = true;
    ayarlaTaslak(g.ham);
    // `g.ham` bilerek bağımlılık değil: sekme değiştiğinde odak çalınmamalı.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [odakSayaci]);

  // Odak, girdi BOYANDIKTAN sonra veriliyor.
  //
  // `ayarlaTaslak` ile aynı turda `girdiRef.current` hâlâ `null`: o anda
  // ekranda düğme var, girdi yok. Odak isteği orada sessizce düşüyordu ve
  // belirtisi kullanıcı için "adres çubuğu açıldı ama yazmıyor" oluyordu —
  // harfler kabuğun gövdesine ya da sayfaya gidiyordu. Fare yolu (aşağıdaki
  // `onClick`) bunu `queueMicrotask` ile zaten çözüyordu; kısayol yolu
  // çözmüyordu.
  //
  // Bağımlılık dizisi YOK: bayrak ancak girdi gerçekten varken temizleniyor,
  // yani odak isteği kaybolmuyor. Etki gövdesi bayrak kapalıyken ilk satırda
  // dönüyor.
  useEffect(() => {
    if (!odakBekliyor.current) return;
    const girdi = girdiRef.current;
    if (!girdi) return;
    odakBekliyor.current = false;
    girdi.focus();
    girdi.select();
  });

  // Liste değişince seçim başa dönüyor: eski indeks yeni listede başka bir
  // adrese denk gelirdi ve kullanıcı Enter'a bastığında gitmek istemediği
  // yere giderdi.
  useEffect(() => {
    ayarlaSecili(-1);
  }, [taslak]);

  const duzenleniyor = taslak !== null;
  const listeAcik = duzenleniyor && oneriler.length > 0;

  const kapat = () => {
    ayarlaTaslak(null);
    ayarlaSecili(-1);
    girdiRef.current?.blur();
  };

  const git = (girdi: string) => {
    onGezin(girdi);
    kapat();
  };

  return (
    <div className="gezinme">
      <div className="gezinme__dugmeler">
        <button
          type="button"
          className="dugme dugme--ikon"
          disabled={!sekme?.geriVar}
          onClick={onGeri}
          title="Geri (Alt+Sol)"
        >
          <Geri />
          <span className="gorunmez">Geri</span>
        </button>
        <button
          type="button"
          className="dugme dugme--ikon"
          disabled={!sekme?.ileriVar}
          onClick={onIleri}
          title="İleri (Alt+Sağ)"
        >
          <Ileri />
          <span className="gorunmez">İleri</span>
        </button>
        <button
          type="button"
          className="dugme dugme--ikon"
          disabled={!sekme}
          onClick={sekme?.yukleniyor ? onDurdur : onYenile}
          title={sekme?.yukleniyor ? "Durdur (Esc)" : "Yenile (F5)"}
        >
          {sekme?.yukleniyor ? <Durdur /> : <Yenile />}
          <span className="gorunmez">{sekme?.yukleniyor ? "Durdur" : "Yenile"}</span>
        </button>
      </div>

      <div className={`adres${duzenleniyor ? " adres--duzenleniyor" : ""}`}>
        {/*
          Kilit **yalnız gerçek bir bağlantı varken** çiziliyor.

          Koşul `adres === ""` değil `g.ham === ""`: `gosterim` kabuğun kendi
          sayfalarında (`muiren://yeni`) boş bir ham adres döndürüyor ve
          eski koşul onu yakalamıyordu. Sonuç, yeni sekme sayfasında
          "bağlantı şifresiz" uyarısıydı — ortada bağlantı yokken. Güvenlik
          göstergesinin yanlış yerde uyarması, doğru yerde uyardığında
          inandırıcılığını götürüyor.
        */}
        <span
          className={`adres__kilit${g.guvenli ? " adres__kilit--guvenli" : " adres__kilit--acik"}`}
          title={g.guvenli ? "Bağlantı şifreli" : "Bağlantı şifresiz"}
        >
          {g.ham === "" ? null : g.guvenli ? <Kilit boyut={13} /> : <Uyari boyut={13} />}
        </span>

        {duzenleniyor ? (
          <input
            ref={girdiRef}
            className="adres__girdi"
            value={taslak}
            spellCheck={false}
            autoComplete="off"
            placeholder="Adres ya da arama"
            role="combobox"
            aria-expanded={listeAcik}
            aria-controls="adres-oneriler"
            aria-autocomplete="list"
            onChange={(e) => ayarlaTaslak(e.target.value)}
            // `onBlur` gecikmeli: öneriye tıklamak önce blur tetikliyor ve
            // liste anında kapanırsa tıklama hiç ulaşmıyor.
            onBlur={() => window.setTimeout(kapat, 120)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                git(secili >= 0 && oneriler[secili] ? oneriler[secili].url : taslak);
              } else if (e.key === "Escape") {
                e.preventDefault();
                kapat();
              } else if (e.key === "ArrowDown" && listeAcik) {
                e.preventDefault();
                ayarlaSecili((n) => (n + 1) % oneriler.length);
              } else if (e.key === "ArrowUp" && listeAcik) {
                e.preventDefault();
                ayarlaSecili((n) => (n <= 0 ? oneriler.length - 1 : n - 1));
              }
            }}
          />
        ) : (
          <button
            type="button"
            className="adres__gosterim"
            onClick={() => {
              ayarlaTaslak(g.ham);
              // Girdi bir sonraki boyamada var olacağı için odak ertelendi.
              queueMicrotask(() => {
                girdiRef.current?.focus();
                girdiRef.current?.select();
              });
            }}
          >
            {g.alan === "" ? (
              <span className="adres__soluk">{g.ham || "Adres ya da arama"}</span>
            ) : (
              <>
                {!g.guvenli && <span className="adres__soluk">{g.sema}//</span>}
                <span className="adres__soluk">{g.onEk}</span>
                <span className="adres__alan">{g.alan}</span>
                <span className="adres__soluk">{g.sonEk}</span>
              </>
            )}
          </button>
        )}

        {listeAcik && (
          <ul className="oneri" id="adres-oneriler" role="listbox">
            {oneriler.map((o, i) => (
              <li key={o.id}>
                <button
                  type="button"
                  role="option"
                  aria-selected={i === secili}
                  className={`oneri__oge${i === secili ? " oneri__oge--secili" : ""}`}
                  // `onMouseDown`: `onClick` blur'dan sonra geliyor.
                  onMouseDown={(e) => {
                    e.preventDefault();
                    git(o.url);
                  }}
                  onMouseEnter={() => ayarlaSecili(i)}
                >
                  <Saat boyut={13} />
                  {/* Başlık ve adres sayfadan geldi; backend temizledi,
                      React kaçışlıyor (CLAUDE.md #8). */}
                  <span className="oneri__ad">{o.baslik || o.alan}</span>
                  <span className="oneri__url">{o.url}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <button
        type="button"
        className={`dugme dugme--ikon${yerImiId !== null ? " dugme--acik" : ""}`}
        disabled={!sekme || adres === ""}
        onClick={onYerImi}
        aria-pressed={yerImiId !== null}
        title={yerImiId !== null ? "Yer imlerinden çıkar" : "Yer imlerine ekle"}
      >
        <Yildiz boyut={15} dolu={yerImiId !== null} />
        <span className="gorunmez">Yer imi</span>
      </button>

      {/* Engellenen şey **sessizce kaybolmuyor** (`src-tauri/src/engel/mod.rs`):
          bazı siteler ödeme ya da giriş akışını pencerede açıyor ve sessiz
          bir engelleme kullanıcıya "tarayıcı bozuk" dedirtiyor. Sayaç sıfırsa
          rozet hiç çizilmiyor — boş bir rozet gürültü. */}
      {engelSayisi > 0 && (
        <span
          className="rozet rozet--engel"
          title={engelIpucu(engelSayisi, engelSebebi)}
        >
          <Kalkan boyut={12} />
          {engelSayisi}
        </span>
      )}

      {nabiz}
    </div>
  );
}
