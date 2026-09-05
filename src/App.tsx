/**
 * Kabuk.
 *
 * İki satır: sekme şeridi + gezinme çubuğu. Ekranın geri kalanı sayfanın
 * (`docs/Frontend.md`). Alt bölge boş bir `div`; sekme webview'i o
 * dikdörtgene yerleşiyor ([`useIcerikAlani`]). Etkin sekme `muiren://yeni`
 * ise webview hiç yok ve o bölgeyi kabuk çiziyor.
 *
 * ## Kısayollar
 *
 * Buradaki `keydown` dinleyicisi **tabloyu tutmuyor**. Tuşu olduğu gibi
 * `kisayol_bas` komutuna veriyor ve tabloyu backend çalıştırıyor
 * (`src-tauri/src/kisayol.rs`). Sebep: sayfa odaktayken tuşlar kabuğa hiç
 * ulaşmıyor ve o durumda motorun hızlandırıcı kaydı devreye giriyor. İki ayrı
 * tablo yazsaydık sessizce ayrışır, sonuç "bazen çalışmıyor" diye rapor
 * edilirdi (`docs/Frontend.md`, Muiply'dan devralınan ders).
 *
 * Arayüzü ilgilendiren kısayollar (adres çubuğuna odaklan, paneli aç)
 * backend'den yapılamıyor; onlar `muiren://kisayol` olayıyla buraya dönüyor
 * ve aşağıdaki tek yerde uygulanıyor — hangi kapıdan geldiğinden bağımsız.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { AdresCubugu } from "./components/AdresCubugu";
import { SekmeArama } from "./components/SekmeArama";
import { Ayarlar } from "./components/Ayarlar";
import { TeshisPaneli } from "./components/TeshisPaneli";
import { SekmeSeridi } from "./components/SekmeSeridi";
import { YanPanel, type PanelSekmesi } from "./components/YanPanel";
import { YeniSekme } from "./components/YeniSekme";
import { Indirme } from "./components/Indirme";
import { KopruMenusu } from "./components/KopruMenusu";
import { Nabiz } from "./components/Nabiz";
import { etkinSekme, useSekmeler } from "./hooks/useSekmeler";
import { useBellek } from "./hooks/useBellek";
import { useEngel } from "./hooks/useEngel";
import { useGruplar } from "./hooks/useGruplar";
import { useIcerikAlani } from "./hooks/useIcerikAlani";
import { useIndirme } from "./hooks/useIndirme";
import { useKopruler } from "./hooks/useKopruler";
import { useTema } from "./hooks/useTema";
import { useYerImi } from "./hooks/useYerImi";
import { useYetenekler } from "./hooks/useYetenekler";
import * as ipc from "./ipc";
import {
  hataMetni,
  OLAY,
  type KisayolIsi,
  type KisayolOlayi,
  type SekmeId,
  type Settings,
} from "./ipc/tipler";
import { YENI_SEKME } from "./lib/url";

/**
 * Tuşun tarayıcı varsayılanını bastırmalı mıyız.
 *
 * Bu bir kısayol **tablosu değil** — hangi tuşun ne yaptığını bilmiyor.
 * Yalnız "bu kombinasyon büyük ihtimalle bir kısayol, sayfanın/`input`un
 * varsayılanı çalışmasın" diyor. `preventDefault` eşzamanlı olmak zorunda ve
 * komutun cevabı eşzamansız geliyor; ikisi arasındaki boşluğu bu dolduruyor.
 *
 * Ctrl+C/V/X/A ve Ctrl+Z gibi metin düzenleme kombinasyonları **dışarıda**:
 * adres çubuğunda kopyala-yapıştır çalışmak zorunda.
 */
const DUZENLEME = new Set(["c", "v", "x", "a", "z", "y"]);

function bastirilsin(e: KeyboardEvent): boolean {
  if (e.key === "F5" || e.key === "F11") return true;
  if (e.altKey && (e.key === "ArrowLeft" || e.key === "ArrowRight")) return true;
  if (!e.ctrlKey && !e.metaKey) return false;
  const t = e.key.toLowerCase();
  // Shift'li olanlar (Ctrl+Shift+A) kısayol; Shift'siz düzenleme tuşları değil.
  if (!e.shiftKey && DUZENLEME.has(t)) return false;
  return true;
}

export default function App() {
  const sekmeler = useSekmeler();
  const yetenekler = useYetenekler();
  const bellek = useBellek();
  const etkin = etkinSekme(sekmeler);
  const gruplar = useGruplar();
  const kopruler = useKopruler();
  const indirme = useIndirme();
  const engel = useEngel(etkin?.id ?? null);
  const { yerImiId, degistir: yerImiDegistir } = useYerImi(etkin?.url, etkin?.baslik);
  // Tema jetonlarını `documentElement.style` üzerine yazıyor; dönüşü yalnız
  // ayarlar ekranının tazeleme çağrısı için.
  const { tazele: temaTazele } = useTema();

  const icerikRef = useRef<HTMLDivElement>(null);
  useIcerikAlani(icerikRef);

  const [aramaAcik, ayarlaArama] = useState(false);
  const [odakSayaci, ayarlaOdak] = useState(0);
  const [duyuru, ayarlaDuyuru] = useState<string | null>(null);
  const [panel, ayarlaPanel] = useState<PanelSekmesi | null>(null);
  const [ayarlarAcik, ayarlaAyarlar] = useState(false);
  const [teshisAcik, ayarlaTeshis] = useState(false);
  /** Teşhis raporunun başlığı ayarları da yazıyor (profil, süreç politikası). */
  const [teshisAyarlari, ayarlaTeshisAyarlari] = useState<Settings | null>(null);

  /** Komut hataları burada tek yerde cümleye çevriliyor (`docs/IPC.md`). */
  const calistir = useCallback((is: Promise<unknown>) => {
    void is.catch((e) => {
      ayarlaDuyuru(hataMetni(e));
      window.setTimeout(() => ayarlaDuyuru(null), 4000);
    });
  }, []);

  const gezin = useCallback(
    (girdi: string) => {
      if (!etkin) {
        calistir(ipc.sekmeAc(girdi));
        return;
      }
      calistir(ipc.gezin(etkin.id, girdi));
    },
    [etkin, calistir],
  );

  const etkinlestir = useCallback(
    (id: SekmeId) => calistir(ipc.sekmeEtkinlestir(id)),
    [calistir],
  );

  /**
   * Arayüz işi kısayollar. Tek uygulama noktası: hem kabuk odaktayken
   * (aşağıdaki `keydown`) hem sayfa odaktayken (`muiren://kisayol` olayı)
   * buraya geliniyor.
   */
  const arayuzKisayolu = useCallback((is: KisayolIsi) => {
    switch (is) {
      case "adresOdak":
        ayarlaOdak((n) => n + 1);
        break;
      case "sekmeArama":
        ayarlaArama((a) => !a);
        break;
      case "bellekPaneli":
        ayarlaPanel((p) => (p === "bellek" ? null : "bellek"));
        break;
      case "yanPanel":
        ayarlaPanel((p) => (p === null ? "sekmeler" : null));
        break;
      case "tamEkran": {
        const pencere = getCurrentWindow();
        void pencere.isFullscreen().then((d) => pencere.setFullscreen(!d));
        break;
      }
    }
  }, []);

  // Sayfa odaktayken motorun yakaladığı arayüz işleri buradan dönüyor.
  useEffect(() => {
    const soz = listen<KisayolOlayi>(OLAY.kisayol, (e) => arayuzKisayolu(e.payload.is));
    return () => {
      void soz.then((kapat) => kapat());
    };
  }, [arayuzKisayolu]);

  /**
   * Açık örtülerden **en üsttekini** kapatır; hiçbiri açık değilse `false`.
   *
   * Sıra kapanma sırası: teşhis ayarların üstünden açılıyor, sekme araması
   * ikisinin de üstüne gelebiliyor. Kapatan tuş her zaman en son açılanı
   * bulmalı, yoksa Escape kullanıcının baktığı ekranı değil arkasındakini
   * kapatır.
   */
  const ortuKapat = useCallback((): boolean => {
    if (aramaAcik) {
      ayarlaArama(false);
      return true;
    }
    if (teshisAcik) {
      ayarlaTeshis(false);
      return true;
    }
    if (ayarlarAcik) {
      ayarlaAyarlar(false);
      return true;
    }
    return false;
  }, [aramaAcik, teshisAcik, ayarlarAcik]);

  // Kabuk odaktayken: tuşu backend'e ver, tabloyu o çalıştırsın.
  useEffect(() => {
    const dinle = (e: KeyboardEvent) => {
      // Escape'in **kabukta** bir işi var: açık örtüyü kapatmak. Backend
      // tablosunda karşılığı `Durdur` (`src-tauri/src/kisayol.rs`) ve ikisi
      // aynı tuşta çakışıyor. Ayrım burada veriliyor çünkü "örtü açık mı"
      // yalnız kabuğun bildiği bir şey: backend ayarlar ekranının açık
      // olduğunu `ortu_gorunur` ile biliyor ama hangisinin üstte olduğunu
      // bilmiyor ve bilmesinin de karşılığı yok.
      //
      // Bu satır olmadan ayarlar ekranındayken Escape sayfanın yüklenmesini
      // durduruyor ve ekran açık kalıyordu — kullanıcının kapatma düğmesini
      // aramaktan başka yolu yoktu.
      if (e.key === "Escape" && ortuKapat()) {
        e.preventDefault();
        return;
      }

      // Yalnız değiştiricili kombinasyonlar ve işlev tuşları gidiyor; her
      // harfi backend'e göndermek saniyede onlarca IPC turu demek olurdu.
      const aday =
        e.ctrlKey || e.metaKey || e.altKey || /^(F\d{1,2}|Escape)$/.test(e.key);
      if (!aday) return;

      if (bastirilsin(e)) e.preventDefault();

      const is = ipc
        .kisayolBas(e.key, e.ctrlKey || e.metaKey, e.shiftKey, e.altKey)
        .catch(() => false);
      void is;
    };

    window.addEventListener("keydown", dinle);
    return () => window.removeEventListener("keydown", dinle);
  }, [ortuKapat]);

  /**
   * Tam ekran örtü açıkken sekme webview'i **gizleniyor**.
   *
   * Sebebi CSS ile çözülemiyor: sekme webview'i ayrı bir native pencere ve
   * kabuğun **üstünde** duruyor (`lib.rs`, pencere düzeni). Kabuğun çizdiği
   * bir örtü sayfanın altında kalıyor ve kullanıcı ayarlar ekranını hiç
   * göremiyor. `z-index` iki ayrı pencerenin sırasını değiştiremiyor.
   *
   * Gizlemek uyutmak değil: sekme durumu ve sayfanın kendisi olduğu gibi
   * duruyor, örtü kapanınca aynı alana geri geliyor
   * (`tabs::surucu::Surucu::ortu_gorunur`).
   */
  // CLAUDE.md #21: yeni bir tam ekran örtü eklendiğinde buraya da eklenmesi
  // gerekiyor, yoksa sekme webview'i üstte kalıyor ve örtü sessizce görünmez
  // oluyor.
  const ortuAcik = ayarlarAcik || aramaAcik || teshisAcik;
  useEffect(() => {
    void ipc.ortuGorunur(ortuAcik).catch(() => {
      /* sürücü hazır değil; örtü yine de çiziliyor */
    });
  }, [ortuAcik]);

  const kabukSayfasi = !etkin || etkin.url === YENI_SEKME || etkin.url === "";

  return (
    <div className="kabuk">
      <SekmeSeridi
        sekmeler={sekmeler}
        gruplar={gruplar.hepsi}
        onEtkinlestir={etkinlestir}
        onKapat={(id) => calistir(ipc.sekmeKapat(id))}
        onSessizeAl={(id, sessiz) => calistir(ipc.sekmeSessizeAl(id, sessiz))}
        onYeni={() => calistir(ipc.sekmeAc())}
        onTasi={(id, hedef) => calistir(ipc.sekmeTasi(id, hedef))}
        onArama={() => ayarlaArama((a) => !a)}
        onYanPanel={() => arayuzKisayolu("yanPanel")}
        yanPanelAcik={panel !== null}
        onAyarlar={() => ayarlaAyarlar(true)}
        // Katlama durumu **backend'de** saklanıyor: sekme özetinin `katli`
        // alanı oradan türetiliyor ve arayüzde ayrı bir kopya tutmak, ikisi
        // ayrıştığında sessizce yanlış çizilen bir şerit demek olurdu.
        onGrupKatla={(id, katli) => calistir(ipc.grupGuncelle(id, { katli }))}
        // Gizli sekme boş açılıyor: adresi kullanıcı yazacak ve o ana kadar
        // hiçbir şey diske değmiyor.
        onGizliSekme={() => calistir(ipc.sekmeAc(undefined, undefined, false, true))}
        kopruMenusu={
          <KopruMenusu
            sekme={etkin}
            muiplyVar={kopruler.kurulu("Muiply")}
            muiwatchVar={kopruler.kurulu("Muiwatch")}
            onMuiply={(yol) => calistir(ipc.muiplyAc(yol))}
            onMuiwatch={(oda) => etkin && calistir(ipc.muiwatchBagla(etkin.id, oda))}
          />
        }
      />

      <AdresCubugu
        sekme={etkin}
        odakSayaci={odakSayaci}
        onGezin={gezin}
        onGeri={() => etkin && calistir(ipc.geri(etkin.id))}
        onIleri={() => etkin && calistir(ipc.ileri(etkin.id))}
        onYenile={() => etkin && calistir(ipc.yenile(etkin.id))}
        onDurdur={() => etkin && calistir(ipc.durdur(etkin.id))}
        yerImiId={yerImiId}
        onYerImi={yerImiDegistir}
        engelSayisi={engel.sayi}
        engelSebebi={engel.son?.sebep ?? null}
        // Bellek nabzı burada: projenin tek iddiası bir panelin arkasında
        // saklıydı ve kullanıcı politikanın çalıştığını hiç görmeden
        // kullanabiliyordu (`components/Nabiz.tsx`).
        // Açma değil **değiştirme**: `Ctrl+Shift+M` de aynı paneli
        // değiştiriyor (`arayuzKisayolu`) ve iki kapının aynı tuşa iki farklı
        // davranış vermesi, kullanıcının panele nasıl geldiğini hatırlamasını
        // gerektirirdi.
        nabiz={
          <Nabiz ozet={bellek} onAc={() => arayuzKisayolu("bellekPaneli")} />
        }
      />

      {/* İndirme önerisi ve sonuç bildirimi. Modal DEĞİL: indirme sayfanın
          işini durdurmuyor, kullanıcıyı da durdurmamalı (`useIndirme`). */}
      <Indirme
        durum={indirme}
        // Muiget kurulu değilse "Muiget'e gönder" düğmesi hiç çizilmiyor
        // (`docs/Kopruler.md`, genel kural). Backend zaten o durumda `sor`
        // politikasını uygulamıyor; bu ikinci kapı, kullanıcı Muiren
        // açıkken Muiget'i kaldırdığında görünen ölü düğmeyi engelliyor.
        muigetVar={kopruler.kurulu("Muiget")}
      />

      <div className="govde">
        {/* Sekme webview'i tam olarak bu dikdörtgene yerleşiyor. Kabuk
            sayfası gösterilirken webview yok; o zaman burayı kabuk
            dolduruyor. Yan panel açıldığında dikdörtgen daralıyor ve
            `useIcerikAlani` yeni ölçüyü backend'e bildiriyor. */}
        <div className="icerik" ref={icerikRef}>
          {kabukSayfasi && (
            <YeniSekme
              onGezin={gezin}
              motorVar={yetenekler.motor}
              // Özet burada da geçiyor: sayfa kendi ölçümünü yapsaydı yeni
              // sekme açmak süreç tablosunu taramak demek olurdu
              // (`bellek_ozeti` komutunun son özeti döndürmesiyle aynı
              // gerekçe).
              ozet={bellek}
              gizli={etkin?.gizli ?? false}
            />
          )}
        </div>

        {/* Kapalıyken hiç çizilmiyor: görünmeyen bir panelde 50 sekmelik
            listeyi tutmanın maliyeti tam da kaçındığımız türden. */}
        {panel !== null && (
          <YanPanel
            sekme={panel}
            onSekme={ayarlaPanel}
            onKapat={() => ayarlaPanel(null)}
            sekmeler={sekmeler}
            ozet={bellek}
            onEtkinlestir={etkinlestir}
            onKapatSekme={(id) => calistir(ipc.sekmeKapat(id))}
            onSessizeAl={(id, sessiz) => calistir(ipc.sekmeSessizeAl(id, sessiz))}
            onHepsiniUyut={() => calistir(ipc.hepsiniUyut())}
            onGezin={gezin}
            gruplar={gruplar.hepsi}
            etkinId={etkin?.id ?? null}
            onGrupAc={(ad) => calistir(ipc.grupAc(ad))}
            onGrupSil={(id) => calistir(ipc.grupSil(id))}
            onGrupGuncelle={(id, d) => calistir(ipc.grupGuncelle(id, d))}
            onGrupAta={(s, g) => calistir(ipc.grupAta(s, g))}
          />
        )}
      </div>

      {aramaAcik && (
        <SekmeArama
          sekmeler={sekmeler}
          onSec={etkinlestir}
          onKapat={() => ayarlaArama(false)}
        />
      )}

      {ayarlarAcik && (
        <Ayarlar
          onKapat={() => ayarlaAyarlar(false)}
          onTemaDegisti={temaTazele}
          onTeshis={() => {
            // Ayarlar kapanıyor: iki örtüyü üst üste açmanın karşılığı yok.
            // Ayarlar o an okunmuş hâliyle teşhise geçiyor — rapor başlığı
            // profili ve süreç politikasını yazmak zorunda ve teşhis ekranının
            // ikinci bir `ayarlar_oku` turu atmasının karşılığı yok.
            void ipc.ayarlarOku()
              .then(ayarlaTeshisAyarlari)
              .catch(() => ayarlaTeshisAyarlari(null));
            ayarlaAyarlar(false);
            ayarlaTeshis(true);
          }}
        />
      )}

      {teshisAcik && (
        <TeshisPaneli
          onKapat={() => ayarlaTeshis(false)}
          ayarlar={teshisAyarlari}
          ozet={bellek}
        />
      )}

      {duyuru && <div className="duyuru">{duyuru}</div>}
    </div>
  );
}
