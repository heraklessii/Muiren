/**
 * Ayarlar ekranı.
 *
 * **Bellek profili görünür ve ayarlanabilir** (`docs/Roadmap.md` Faz 3):
 * `docs/Bellek.md` şunu şart koşuyor — "Kullanıcı 'Sıkı' moddayken 3 GB
 * kazanıp bunu görebilmeli." Eşikleri gizleyen bir tarayıcıda kullanıcı
 * sekmesinin neden kaybolduğunu anlamıyor ve özelliği kapatıyor.
 *
 * İki kural burada özellikle geçerli:
 *
 * - **Düzeltilmiş değer gösteriliyor** (CLAUDE.md #13). `ayarlar_yaz` ham
 *   değil `duzelt`ten geçmiş ayarları döndürüyor; ekran o cevabı yazıyor.
 *   Kullanıcı uyku eşiğine 0 yazdığında kutuda 30 belirir — sessizce farklı
 *   bir değerle çalışmak yok.
 * - **Yeniden başlatma isteyen ayarlar işaretli.** Süreç politikası ve
 *   renderer tavanı Chromium bayrağına dönüşüyor ve bayraklar ancak WebView2
 *   ortamı yeniden kurulunca geçiyor (`docs/Setup.md`, CLAUDE.md #16).
 *   Sessizce etkisiz kalan bir ayar, kullanıcıya "bu özellik çalışmıyor"
 *   dedirtiyor.
 */

import { useEffect, useState } from "react";

import { Cop, Kapat } from "./Ikonlar";
import {
  ayarlarOku,
  ayarlarYaz,
  engelOzeti as engelOzetiOku,
  temaListesi,
  temaUygula,
  veriTemizle,
} from "../ipc";
import { useKopruler } from "../hooks/useKopruler";
import { sure } from "../lib/bicim";
import type {
  EngelOzeti,
  IndirmePolitikasi,
  Profil,
  Settings,
  SurecPolitikasi,
  TemaOzeti,
  Temizlik,
  TemizlikRaporu,
} from "../ipc/tipler";

/**
 * Profil tanımları. Sayılar burada **yok**: profil seçilince backend
 * varsayılanları getiriyor. Buradaki metinler yalnız açıklama
 * (`docs/Bellek.md` tablosu).
 */
const PROFILLER: { deger: Profil; ad: string; not: string }[] = [
  { deger: "siki", ad: "Sıkı", not: "8 GB ve altı · uyku 2 dk · atma 20 dk" },
  { deger: "dengeli", ad: "Dengeli", not: "8–24 GB · uyku 8 dk · atma 60 dk" },
  { deger: "rahat", ad: "Rahat", not: "24 GB üstü · uyku 20 dk · atma 3 sa" },
];

const SUREC: { deger: SurecPolitikasi; ad: string; not: string }[] = [
  {
    deger: "birlesik",
    ad: "Birleşik",
    not: "Aynı sitenin sekmeleri tek süreci paylaşır. Daha az RAM; o sitenin bir sekmesi çökerse hepsi gider — adresten geri geliyorlar.",
  },
  {
    deger: "varsayilan",
    ad: "Chromium varsayılanı",
    not: "Daha çok RAM, daha iyi çökme yalıtımı.",
  },
];

const INDIRME: { deger: IndirmePolitikasi; ad: string; not: string }[] = [
  {
    deger: "sor",
    ad: "Her seferinde sor",
    not: "İndirme başladığında adres çubuğunun altında bir şerit çıkıyor; Muiget'e gönder ya da Muiren indirsin.",
  },
  {
    deger: "muiget",
    ad: "Hep Muiget'e",
    not: "İndirme doğrudan devrediliyor. Muiget'in devam ettirme, sıra ve hız sınırı özellikleri Muiren'de yok.",
  },
  {
    deger: "motorda",
    ad: "Muiren indirsin",
    not: "Köprü hiç kullanılmıyor; motorun kendi indirmesi çalışıyor.",
  },
];

/**
 * Temizlenebilecek kalemler.
 *
 * `Temizlik` alanlarıyla birebir; anahtar tipi bunu derleyiciye zorlatıyor.
 * Yeni bir kalem eklendiğinde burada da görünmesi gerekiyor, yoksa arayüzde
 * ulaşılamayan bir seçenek kalırdı.
 */
const TEMIZLIK: { anahtar: keyof Temizlik; ad: string; not: string }[] = [
  {
    anahtar: "gecmis",
    ad: "Geçmiş",
    not: "Ziyaret kayıtları. Yer imleri kalıyor.",
  },
  {
    anahtar: "cerezler",
    ad: "Çerezler",
    not: "Açık oturumlar kapanıyor.",
  },
  {
    anahtar: "onbellek",
    ad: "Önbellek",
    not: "Disk önbelleği. Sayfalar bir kez daha yavaş açılıyor.",
  },
  {
    anahtar: "siteVerisi",
    ad: "Site verisi",
    not: "localStorage, IndexedDB ve Service Worker'lar. Çevrimdışı kopyalar gidiyor.",
  },
  {
    anahtar: "favicon",
    ad: "Favicon deposu",
    not: "Sekme ikonları. Yeniden ziyarette geri geliyorlar.",
  },
  {
    anahtar: "otomatikDoldurma",
    ad: "Otomatik doldurma",
    not: "Adres ve form ipuçları. Şifreler dahil değil — Muiren şifre saklamıyor.",
  },
];

const BOS_TEMIZLIK: Temizlik = {
  gecmis: false,
  favicon: false,
  cerezler: false,
  onbellek: false,
  siteVerisi: false,
  otomatikDoldurma: false,
};

/**
 * Kural kutusunun örneği.
 *
 * Ayrı bir sabit çünkü `placeholder` içinde satır sonu gerekiyor ve JSX
 * içinde `&#10;` yazmak okunmuyor.
 */
const KURAL_ORNEGI = [
  "! yorum satırı",
  "reklam.example.com",
  "||izleyici.example.net^",
  "/reklamlar/",
  "@@izin.example.com",
].join("\n");

interface Ozellik {
  onKapat: () => void;
  /** Tema değişince kabuk jetonları yeniden okuyor. */
  onTemaDegisti: () => void;
  /**
   * Teşhis ekranını açıyor.
   *
   * Ayarlar **kapanıyor**: iki tam ekran örtüyü üst üste açmak, sekme
   * webview'i zaten gizliyken ikinci bir gizleme turu ve kullanıcının
   * "hangisini kapattım" sorusu demek (`docs/Frontend.md`).
   */
  onTeshis: () => void;
}

/** Saniye girdisi — dakika olarak gösterip saniye olarak saklıyor. */
function SureAlani({
  etiket,
  not,
  deger,
  onDeger,
}: {
  etiket: string;
  not: string;
  deger: number;
  onDeger: (n: number) => void;
}) {
  return (
    <label className="ayar">
      <span className="ayar__ad">{etiket}</span>
      <span className="ayar__girdi">
        <input
          type="number"
          min={1}
          value={Math.round(deger / 60)}
          onChange={(e) => onDeger(Math.max(1, Number(e.target.value)) * 60)}
        />
        <span className="ayar__birim">dakika</span>
      </span>
      <span className="ayar__not">
        {not} Şu an: {sure(deger)}.
      </span>
    </label>
  );
}

export function Ayarlar({ onKapat, onTemaDegisti, onTeshis }: Ozellik) {
  const [ayarlar, ayarla] = useState<Settings | null>(null);
  const [yenidenBaslat, ayarlaYenidenBaslat] = useState(false);
  /**
   * Filtre açık/kapalı değişti mi.
   *
   * Yeniden **başlatma** değil yeniden **yükleme** gerekiyor ve fark gerçek:
   * istek süzgeci webview yaratılırken kaydediliyor (`Motor::istek_suzgeci_acik`),
   * dolayısıyla sonradan açılan ya da yenilenen sekmeler yeni ayarla
   * çalışıyor. Sessizce etkisiz kalan bir ayar, kullanıcıya "bu özellik
   * çalışmıyor" dedirtir.
   */
  const [yenidenYukle, ayarlaYenidenYukle] = useState(false);
  const [temalar, ayarlaTemalar] = useState<TemaOzeti[]>([]);
  const [engelOzet, ayarlaEngelOzet] = useState<EngelOzeti | null>(null);
  const [temizlik, ayarlaTemizlik] = useState<Temizlik>(BOS_TEMIZLIK);
  const [rapor, ayarlaRapor] = useState<TemizlikRaporu | null>(null);
  const kopruler = useKopruler();

  useEffect(() => {
    let canli = true;
    void ayarlarOku().then((a) => {
      if (canli) ayarla(a);
    });
    void temaListesi().then((t) => {
      if (canli) ayarlaTemalar(t);
    });
    return () => {
      canli = false;
    };
  }, []);

  // Engel özeti ayarlar her yazıldığında tazeleniyor: kural metni değiştiğinde
  // kaç kuralın derlendiği ve hangi satırların anlaşılmadığı hemen görünmeli.
  useEffect(() => {
    if (!ayarlar) return;
    let canli = true;
    void engelOzetiOku()
      .then((o) => {
        if (canli) ayarlaEngelOzet(o);
      })
      .catch(() => {});
    return () => {
      canli = false;
    };
  }, [ayarlar?.filtreKurallari, ayarlar?.filtreAcik]);

  if (!ayarlar) return null;

  /**
   * Yazıp **düzeltilmiş** cevabı ekrana koyuyor (CLAUDE.md #13).
   *
   * `bayrak` true ise bu değişiklik Chromium bayrağına dönüşüyor ve yeniden
   * başlatma rozeti çıkıyor.
   */
  const yaz = (yeni: Settings, bayrak = false) => {
    ayarla(yeni); // iyimser: kutu anında tepki versin
    void ayarlarYaz(yeni).then((duzeltilmis) => {
      ayarla(duzeltilmis);
      if (bayrak) ayarlaYenidenBaslat(true);
    });
  };

  return (
    <div className="ortu" onClick={onKapat}>
      <div
        className="ayarlar"
        role="dialog"
        aria-label="Ayarlar"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="ayarlar__baslik">
          <h2>Ayarlar</h2>
          <button
            type="button"
            className="dugme dugme--ikon"
            onClick={onKapat}
            aria-label="Kapat"
          >
            <Kapat boyut={14} />
          </button>
        </div>

        {yenidenBaslat && (
          <p className="ayarlar__rozet">
            Süreç ayarları Chromium bayrağına dönüşüyor ve ancak Muiren yeniden
            başlatılınca etkili oluyor.
          </p>
        )}

        {yenidenYukle && (
          <p className="ayarlar__rozet">
            İstek filtresi webview yaratılırken kuruluyor: açık sekmeler
            yenilenene kadar eski ayarla çalışıyor.
          </p>
        )}

        <div className="ayarlar__govde">
          <section className="ayarlar__bolum">
            <h3>Bellek profili</h3>
            <p className="ayar__not">
              Profil seçmek eşikleri makinenin RAM'ine göre ayarlıyor. Aşağıdan
              tek tek değiştirebilirsiniz.
            </p>
            <div className="ayarlar__secenekler">
              {PROFILLER.map((p) => (
                <button
                  key={p.deger}
                  type="button"
                  className={`secenek${ayarlar.profil === p.deger ? " secenek--etkin" : ""}`}
                  onClick={() => yaz({ ...ayarlar, profil: p.deger })}
                >
                  <b>{p.ad}</b>
                  <span>{p.not}</span>
                </button>
              ))}
            </div>
          </section>

          <section className="ayarlar__bolum">
            <h3>Eşikler</h3>
            <SureAlani
              etiket="Uyku eşiği"
              not="Bu süre dokunulmayan sekme uyutuluyor; sayfanın durumu korunuyor."
              deger={ayarlar.uykuEsigiSn}
              onDeger={(n) => yaz({ ...ayarlar, uykuEsigiSn: n })}
            />
            <SureAlani
              etiket="Atma eşiği"
              not="Bu süre sonunda webview tamamen bırakılıyor; sekme adresten geri geliyor. Uyku eşiğinin altına inemiyor."
              deger={ayarlar.atmaEsigiSn}
              onDeger={(n) => yaz({ ...ayarlar, atmaEsigiSn: n })}
            />
            <label className="ayar">
              <span className="ayar__ad">Aynı anda uyanık üst sınır</span>
              <span className="ayar__girdi">
                <input
                  type="number"
                  min={1}
                  max={200}
                  value={ayarlar.uyanikUstSinir}
                  onChange={(e) =>
                    yaz({ ...ayarlar, uyanikUstSinir: Number(e.target.value) })
                  }
                />
                <span className="ayar__birim">sekme</span>
              </span>
              <span className="ayar__not">
                Sınır aşıldığında en uzun süredir dokunulmayanlar uyutuluyor.
              </span>
            </label>
          </section>

          <section className="ayarlar__bolum">
            <h3>Uyutulmayacak siteler</h3>
            <label className="ayar">
              <span className="ayar__ad">Alan adları</span>
              <textarea
                className="ayar__metin"
                rows={3}
                value={ayarlar.istisnaAlanlari.join("\n")}
                placeholder="posta.sirket.com&#10;panel.ornek.net"
                onChange={(e) =>
                  ayarla({
                    ...ayarlar,
                    istisnaAlanlari: e.target.value.split("\n"),
                  })
                }
                onBlur={() => yaz(ayarlar)}
              />
              <span className="ayar__not">
                Her satıra bir alan adı. Alt alan adları da kapsanıyor:
                <code>sirket.com</code> yazmak <code>posta.sirket.com</code>
                sekmesini de koruyor.
              </span>
            </label>
          </section>

          <section className="ayarlar__bolum">
            <h3>
              Süreç modeli
              <span className="ayarlar__uyari">yeniden başlatma gerekiyor</span>
            </h3>
            <div className="ayarlar__secenekler">
              {SUREC.map((s) => (
                <button
                  key={s.deger}
                  type="button"
                  className={`secenek${
                    ayarlar.surecPolitikasi === s.deger ? " secenek--etkin" : ""
                  }`}
                  onClick={() => yaz({ ...ayarlar, surecPolitikasi: s.deger }, true)}
                >
                  <b>{s.ad}</b>
                  <span>{s.not}</span>
                </button>
              ))}
            </div>
            <label className="ayar">
              <span className="ayar__ad">Renderer süreç tavanı</span>
              <span className="ayar__girdi">
                <input
                  type="number"
                  min={0}
                  max={64}
                  value={ayarlar.rendererTavani}
                  onChange={(e) =>
                    yaz({ ...ayarlar, rendererTavani: Number(e.target.value) }, true)
                  }
                />
                <span className="ayar__birim">0 = sınırsız</span>
              </span>
              <span className="ayar__not">
                Düşük tavan (4–6) belleği daha çok düşürüyor ama sekmeler
                CPU'da sıraya giriyor. 8 GB'lık makinede makul, 32 GB'da
                gereksiz.
              </span>
            </label>
          </section>

          <section className="ayarlar__bolum">
            <h3>Tema</h3>
            <p className="ayar__not">
              Tema bir <b>veri</b> paketi: jeton kümesi ve arka plan. Kod
              çalıştırmıyor — tarayıcı arayüzünde çalışan kod, açılan her
              sayfayı görebilirdi.
            </p>
            <div className="ayarlar__secenekler">
              {temalar.map((t) => (
                <button
                  key={t.ad}
                  type="button"
                  className={`secenek${ayarlar.tema === t.ad ? " secenek--etkin" : ""}`}
                  onClick={() => {
                    void temaUygula(t.ad).then(() => {
                      ayarla({ ...ayarlar, tema: t.ad });
                      onTemaDegisti();
                    });
                  }}
                >
                  <b>
                    {t.ad}
                    <span className="tema__ornek">
                      {t.jetonlar
                        .filter(([a]) => a === "--bg" || a === "--accent" || a === "--text")
                        .map(([a, d]) => (
                          <i key={a} style={{ background: d }} aria-hidden />
                        ))}
                    </span>
                  </b>
                  <span>
                    {t.yerlesik ? "yerleşik" : `${t.yazar || "bilinmeyen"} · ${t.surum}`}
                    {t.uyarilar.length > 0 && ` · ${t.uyarilar.length} uyarı`}
                  </span>
                  {/* Uyarı tema reddedilmediği için gösteriliyor: kullanıcı
                      okunmaz bir arayüzle baş başa kalmasın. */}
                  {ayarlar.tema === t.ad &&
                    t.uyarilar.map((u) => (
                      <span key={u} className="tema__uyari">
                        {u}
                      </span>
                    ))}
                </button>
              ))}
            </div>
          </section>

          <section className="ayarlar__bolum">
            <h3>Arama ve geçmiş</h3>
            <label className="ayar">
              <span className="ayar__ad">Arama şablonu</span>
              <span className="ayar__girdi">
                <input
                  type="text"
                  className="ayar__uzun"
                  value={ayarlar.aramaUrl}
                  onChange={(e) => ayarla({ ...ayarlar, aramaUrl: e.target.value })}
                  onBlur={() => yaz(ayarlar)}
                  spellCheck={false}
                />
              </span>
              <span className="ayar__not">
                <code>%s</code> sorgunun yerini tutuyor ve <code>https://</code>
                zorunlu. Geçersiz şablon varsayılana dönüyor. Canlı öneri API'si
                kullanılmıyor — öneriler yalnız yerel geçmişten.
              </span>
            </label>
            <label className="ayar">
              <span className="ayar__ad">Geçmiş saklama</span>
              <span className="ayar__girdi">
                <input
                  type="number"
                  min={-1}
                  max={3650}
                  value={ayarlar.gecmisSaklamaGun}
                  onChange={(e) =>
                    yaz({ ...ayarlar, gecmisSaklamaGun: Number(e.target.value) })
                  }
                />
                <span className="ayar__birim">gün</span>
              </span>
              <span className="ayar__not">
                <code>0</code> sınırsız, <code>-1</code> geçmiş tutma. Budama
                açılışta arka planda koşuyor.
              </span>
            </label>
          </section>

          <section className="ayarlar__bolum">
            <h3>Köprüler</h3>
            <p className="ayar__not">
              Kardeş uygulama kurulu değilse ilgili menü <b>hiç çizilmiyor</b> —
              hata verilmiyor, indirme önerisi zorlanmıyor
              (<code>docs/Kopruler.md</code>). Hiçbir köprü çerez, oturum
              başlığı ya da sayfa içeriği taşımıyor; gönderilen şey her zaman
              bir adres ya da bir dosya yolu.
            </p>
            <div className="ayarlar__secenekler">
              {kopruler.hepsi.map((k) => (
                <div
                  key={k.ad}
                  className={`secenek${k.kurulu ? " secenek--etkin" : ""}`}
                  title={k.yol ?? "bulunamadı"}
                >
                  <b>{k.ad}</b>
                  <span>{k.kurulu ? "kurulu" : "kurulu değil"}</span>
                </div>
              ))}
            </div>
            <button type="button" className="dugme" onClick={kopruler.tazele}>
              Yeniden ara
            </button>
            <p className="ayar__not">
              Muiren açıkken bir kardeş uygulama kurduysanız arama sonucu
              önbellekte kalmış olabilir.
            </p>
          </section>

          {/* Muiget kurulu değilse bu bölüm hiç çizilmiyor: üç seçenek de aynı
              sonucu verirdi (motorun kendi indirmesi) ve seçenek sunmak
              kullanıcıyı yanıltırdı. Karar #4'ün "kullanıcı indirme yapamaz
              duruma düşmüyor" yarısı zaten geçerli. */}
          {kopruler.kurulu("Muiget") && (
            <section className="ayarlar__bolum">
              <h3>İndirme</h3>
              <div className="ayarlar__secenekler">
                {INDIRME.map((i) => (
                  <button
                    key={i.deger}
                    type="button"
                    className={`secenek${
                      ayarlar.indirmePolitikasi === i.deger ? " secenek--etkin" : ""
                    }`}
                    onClick={() => yaz({ ...ayarlar, indirmePolitikasi: i.deger })}
                  >
                    <b>{i.ad}</b>
                    <span>{i.not}</span>
                  </button>
                ))}
              </div>
              <p className="ayar__not">
                Muiget'e giden istek yalnız adresi, dosya adını ve sayfanın
                adresini taşıyor. <b>Çerez ve oturum başlığı gönderilmiyor</b>;
                bunun bedeli, giriş gerektiren bir indirmenin Muiget tarafında
                başarısız olabilmesi.
              </p>
            </section>
          )}

          <section className="ayarlar__bolum">
            <h3>Engelleme</h3>
            <label className="ayar ayar--satir">
              <input
                type="checkbox"
                checked={ayarlar.engellemeAcik}
                onChange={(e) => yaz({ ...ayarlar, engellemeAcik: e.target.checked })}
              />
              <span className="ayar__ad">
                Kendiliğinden açılan pencereleri ve yönlendirmeleri engelle
              </span>
              <span className="ayar__not">
                Kullanıcının <b>tıkladığı</b> bağlantı her zaman açılıyor;
                engellenen yalnız sayfanın kendi kendine açtığı pencere. Bir şey
                engellendiğinde adres çubuğunda rozet çıkıyor — sessiz engelleme
                yok.
              </span>
            </label>

            <label className="ayar ayar--satir">
              <input
                type="checkbox"
                checked={ayarlar.filtreAcik}
                onChange={(e) => {
                  yaz({ ...ayarlar, filtreAcik: e.target.checked });
                  ayarlaYenidenYukle(true);
                }}
              />
              <span className="ayar__ad">
                İstek filtresini kullan
                <span className="ayarlar__uyari">
                  sekmelerin yeniden yüklenmesi gerekiyor
                </span>
              </span>
              <span className="ayar__not">
                Kutudan çıkan liste <b>yok</b>: filtre listeleri kullanıcı
                tarafından konuyor (karar #5). Filtre kapalıyken sayfa istekleri
                kabuk sürecinden hiç geçmiyor — bedeli sıfır.
              </span>
            </label>

            <label className="ayar">
              <span className="ayar__ad">Filtre kuralları</span>
              <textarea
                className="ayar__metin"
                rows={5}
                value={ayarlar.filtreKurallari.join("\n")}
                placeholder={KURAL_ORNEGI}
                spellCheck={false}
                onChange={(e) =>
                  ayarla({ ...ayarlar, filtreKurallari: e.target.value.split("\n") })
                }
                onBlur={() => yaz(ayarlar)}
              />
              <span className="ayar__not">
                Her satıra bir kural. Alan adı yazmak alt alan adlarını da
                kapsıyor; <code>@@</code> öneki istisna. Seçenekli
                (<code>$third-party</code>) ve öğe gizleyen (<code>##</code>)
                kurallar <b>uygulanmıyor</b> ve aşağıda anlaşılmayan olarak
                listeleniyor.
              </span>
            </label>

            {engelOzet && (
              <p className="ayar__not">
                {engelOzet.kuralSayisi} kural derlendi.
                {engelOzet.anlasilmayanlar.length > 0 && (
                  <>
                    {" "}
                    <b>{engelOzet.anlasilmayanlar.length} satır anlaşılmadı:</b>{" "}
                    <code>{engelOzet.anlasilmayanlar.slice(0, 5).join(" · ")}</code>
                    {engelOzet.anlasilmayanlar.length > 5 && " …"}
                  </>
                )}
              </p>
            )}
          </section>

          <section className="ayarlar__bolum">
            <h3>Oyun modu</h3>
            <label className="ayar ayar--satir">
              <input
                type="checkbox"
                checked={ayarlar.oyunAlgilama}
                onChange={(e) => yaz({ ...ayarlar, oyunAlgilama: e.target.checked })}
              />
              <span className="ayar__ad">Oyun algılandığında sekmeleri uyut</span>
              <span className="ayar__not">
                Tam ekran ve kenarlıksız (ya da yüksek öncelikli) bir pencere
                oyun sayılıyor. Bu <b>kaba bir sezgi</b> ve yanlış pozitif
                ihtimali var; o yüzden varsayılan kapalı. Açıkken oyun
                başlayınca eşikler dörde bölünüyor ve uyanık sınır 2'ye iniyor.
                Oyun bitince sekmeler <b>kendiliğinden uyandırılmıyor</b>: 30
                sekmenin birden yüklenmesi tam da kaçındığımız sıçrama olurdu.
              </span>
            </label>

            <label className="ayar ayar--satir">
              <input
                type="checkbox"
                checked={ayarlar.oyunPencereGizle}
                onChange={(e) =>
                  yaz({ ...ayarlar, oyunPencereGizle: e.target.checked })
                }
              />
              <span className="ayar__ad">
                Oyun modunda Muiren penceresini simge durumuna al
              </span>
              <span className="ayar__not">
                Pencere <b>gizlenmiyor, küçültülüyor</b>: görev çubuğundan her
                an geri geliyor. Oyun bitince Muiren onu kendisi geri getiriyor
                — kendi küçülttüğünüz pencereye dokunulmuyor. Elle açtığınız
                oyun modunda da geçerli.
              </span>
            </label>
          </section>

          <section className="ayarlar__bolum">
            <h3>Teşhis</h3>
            <p className="ayar__not">
              GPU hızlandırma, codec ve DRM durumu — video takılmasının dört
              sebebinden hangisinin bu makinede geçerli olduğunu söyleyen ekran
              (`docs/Medya.md`). Ölçüm raporunun başlık tablosu da orada.
            </p>
            <button type="button" className="dugme" onClick={onTeshis}>
              Teşhis ekranını aç
            </button>
          </section>

          <section className="ayarlar__bolum">
            <h3>Veri temizleme</h3>
            <p className="ayar__not">
              Seçilenler hem motorun profilinden hem Muiren'in kendi
              depolarından siliniyor. <b>Yer imleri hiçbir kalemde
              silinmiyor</b> — kasten sakladığınız şeyler.
            </p>
            {TEMIZLIK.map((t) => (
              <label key={t.anahtar} className="ayar ayar--satir">
                <input
                  type="checkbox"
                  checked={temizlik[t.anahtar]}
                  onChange={(e) =>
                    ayarlaTemizlik({ ...temizlik, [t.anahtar]: e.target.checked })
                  }
                />
                <span className="ayar__ad">{t.ad}</span>
                <span className="ayar__not">{t.not}</span>
              </label>
            ))}
            <button
              type="button"
              className="dugme dugme--tehlike"
              disabled={!Object.values(temizlik).some(Boolean)}
              onClick={() => {
                void veriTemizle(temizlik).then((r) => {
                  // "Temizlendi" demek hiçbir şey silinmediğinde de doğru
                  // görünürdü; sayı veriliyor.
                  ayarlaRapor(r);
                  ayarlaTemizlik(BOS_TEMIZLIK);
                });
              }}
            >
              <Cop boyut={14} /> Seçilenleri sil
            </button>
            {rapor && (
              <p className="ayar__not">
                {rapor.gecmisKaydi} geçmiş kaydı, {rapor.faviconDosyasi} favicon
                silindi.
                {rapor.motorBaslatildi &&
                  " Motor tarafındaki silme başlatıldı (eşzamansız)."}
              </p>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}
