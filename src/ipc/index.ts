/**
 * `invoke` sarmalayıcıları.
 *
 * **Bileşenler `invoke` görmüyor** (`docs/Frontend.md`): komut adı, argüman
 * şekli ve dönüş tipi tek yerde durunca sözleşme değiştiğinde derleyici bütün
 * çağrı yerlerini gösteriyor. Serpiştirilmiş `invoke("sekme_ac", ...)`
 * çağrıları bunu çalışma zamanına bırakır.
 */

import { invoke } from "@tauri-apps/api/core";

import type {
  Aralik,
  BellekOzeti,
  Dikdortgen,
  EngelOzeti,
  GecikmeOzeti,
  GecmisKaydi,
  Grup,
  GrupId,
  GrupRengi,
  KopruDurumu,
  SekmeId,
  SekmeOzeti,
  Settings,
  TemaOzeti,
  Temizlik,
  TemizlikRaporu,
  YerImi,
  Yetenekler,
} from "./tipler";

/* ----------------------------------------------------------------- sekmeler */

/**
 * Yeni sekme. `gizli` InPrivate kipinde açıyor (`Ctrl+Shift+N`).
 *
 * Gizli bayrağı **ebeveynden miras alınmıyor**: kararı çağıran veriyor.
 * Miras kuralı backend'te olsaydı kullanıcının menüden açtığı normal sekme
 * de gizli doğardı (`tabs::surucu::Surucu::sekme_ac`).
 */
export function sekmeAc(
  url?: string,
  ebeveyn?: SekmeId,
  arkaplanda = false,
  gizli = false,
): Promise<SekmeId> {
  return invoke("sekme_ac", {
    url: url ?? null,
    ebeveyn: ebeveyn ?? null,
    arkaplanda,
    gizli,
  });
}

export function sekmeKapat(id: SekmeId): Promise<SekmeId | null> {
  return invoke("sekme_kapat", { id });
}

export function sekmeEtkinlestir(id: SekmeId): Promise<void> {
  return invoke("sekme_etkinlestir", { id });
}

export function sekmeTasi(id: SekmeId, hedef: number): Promise<void> {
  return invoke("sekme_tasi", { id, hedef });
}

export function sekmeSabitle(id: SekmeId, sabit: boolean): Promise<void> {
  return invoke("sekme_sabitle", { id, sabit });
}

export function sekmeSessizeAl(id: SekmeId, sessiz: boolean): Promise<void> {
  return invoke("sekme_sessize_al", { id, sessiz });
}

export function sekmeUyutmaIstisnasi(id: SekmeId, istisna: boolean): Promise<void> {
  return invoke("sekme_uyutma_istisnasi", { id, istisna });
}

/** Uyuyan sekmeyi uyandırmıyor (CLAUDE.md #5). */
export function sekmeListesi(): Promise<SekmeOzeti[]> {
  return invoke("sekme_listesi");
}

export function sekmeGeriAl(): Promise<SekmeId | null> {
  return invoke("sekme_geri_al");
}

export function sekmeUyut(id: SekmeId): Promise<void> {
  return invoke("sekme_uyut", { id });
}

export function sekmeAt(id: SekmeId): Promise<void> {
  return invoke("sekme_at", { id });
}

/* ----------------------------------------------------------------- gezinme */

/** `girdi` ham kullanıcı metni; ayrımı `lib/url.ts` yapıyor, backend doğruluyor. */
export function gezin(id: SekmeId, girdi: string): Promise<void> {
  return invoke("gezin", { id, girdi });
}

export function geri(id: SekmeId): Promise<void> {
  return invoke("geri", { id });
}

export function ileri(id: SekmeId): Promise<void> {
  return invoke("ileri", { id });
}

export function yenile(id: SekmeId): Promise<void> {
  return invoke("yenile", { id });
}

export function durdur(id: SekmeId): Promise<void> {
  return invoke("durdur", { id });
}

/* -------------------------------------------------------------------- kabuk */

/** Kabuğun altında kalan alanı bildiriyor; sekme webview'i oraya yerleşiyor. */
export function icerikAlani(alan: Dikdortgen): Promise<void> {
  return invoke("icerik_alani", { alan });
}

export function yetenekler(): Promise<Yetenekler> {
  return invoke("yetenekler");
}

export function ayarlarOku(): Promise<Settings> {
  return invoke("ayarlar_oku");
}

/** **Düzeltilmiş** ayarları döndürüyor; arayüz onu gösteriyor (CLAUDE.md #13). */
export function ayarlarYaz(ayarlar: Settings): Promise<Settings> {
  return invoke("ayarlar_yaz", { ayarlar });
}

/* ------------------------------------------------------------------- bellek */

/**
 * Son bellek özeti. Gözcü ilk turunu koşmadıysa `null`; arayüz o zaman ilk
 * `muiren://bellek-ozeti` olayını bekliyor.
 */
export function bellekOzeti(): Promise<BellekOzeti | null> {
  return invoke("bellek_ozeti");
}

/** Korumalı olmayan her uyanık sekmeyi uyutur. Dönüş: uyutulan sayısı. */
export function hepsiniUyut(): Promise<number> {
  return invoke("hepsini_uyut");
}

export function oyunModu(acik: boolean): Promise<void> {
  return invoke("oyun_modu", { acik });
}

/**
 * Uyanma gecikmesi dağılımı.
 *
 * `bellekOzeti` ile birleşik değil: o özet gözcünün turuna bağlı ve olayla
 * yayınlanıyor, bu ise kullanıcının sekme değiştirmesiyle değişiyor. Tek
 * çağrı olsaydı panel, gecikme tazelensin diye gözcü turunu beklerdi.
 */
export function gecikmeOzeti(): Promise<GecikmeOzeti> {
  return invoke("gecikme_ozeti");
}

/** Ölçüm oturumuna temiz başlamak için defteri boşaltır. */
export function gecikmeSifirla(): Promise<void> {
  return invoke("gecikme_sifirla");
}

/* ------------------------------------------------------------------ kısayol */

/**
 * Kısayolu backend'e uygulatıyor. Dönüş: tabloda karşılığı var mıydı.
 *
 * Tablo **burada değil** (`docs/IPC.md`): `src-tauri/src/kisayol.rs` içinde ve
 * sayfa odaktayken motorun hızlandırıcı kaydı da aynı tabloya giriyor. İki
 * tablo yazsaydık sessizce ayrışırlardı.
 */
export function kisayolBas(
  tus: string,
  ctrl: boolean,
  shift: boolean,
  alt: boolean,
): Promise<boolean> {
  return invoke("kisayol_bas", { tus, ctrl, shift, alt });
}

/* ---------------------------------------------------- geçmiş ve yer imleri */

/** Geçmişte arama. Boş sorgu en son ziyaretleri veriyor. **Ağ isteği yok.** */
export function gecmisAra(sorgu: string, limit = 8): Promise<GecmisKaydi[]> {
  return invoke("gecmis_ara", { sorgu, limit });
}

export function gecmisSil(id: number): Promise<void> {
  return invoke("gecmis_sil", { id });
}

export function gecmisTemizle(aralik: Aralik): Promise<number> {
  return invoke("gecmis_temizle", { aralik });
}

export function yerImiEkle(
  url: string,
  baslik: string,
  klasor: number | null = null,
): Promise<number> {
  return invoke("yer_imi_ekle", { url, baslik, klasor });
}

export function yerImiSil(id: number): Promise<void> {
  return invoke("yer_imi_sil", { id });
}

export function yerImiListesi(klasor: number | null = null): Promise<YerImi[]> {
  return invoke("yer_imi_listesi", { klasor });
}

/** Bu adres yer imlerinde mi; adres çubuğundaki yıldız bunu okuyor. */
export function yerImiMi(url: string): Promise<number | null> {
  return invoke("yer_imi_mi", { url });
}

/* ---------------------------------------------------------------------- tema */

export function temaListesi(): Promise<TemaOzeti[]> {
  return invoke("tema_listesi");
}

/** `.muitema` dosyasını doğrulayıp kurar ve uygular. */
export function temaYukle(dosyaYolu: string): Promise<TemaOzeti> {
  return invoke("tema_yukle", { dosyaYolu });
}

export function temaUygula(ad: string): Promise<TemaOzeti> {
  return invoke("tema_uygula", { ad });
}

export function temaSil(ad: string): Promise<void> {
  return invoke("tema_sil", { ad });
}

export function temaDisaAktar(ad: string, hedef: string): Promise<void> {
  return invoke("tema_disa_aktar", { ad, hedef });
}

/** Açılışta uygulanacak tema. */
export function temaEtkin(): Promise<TemaOzeti | null> {
  return invoke("tema_etkin");
}

/* ------------------------------------------------------------------ gruplar */

export function grupListesi(): Promise<Grup[]> {
  return invoke("grup_listesi");
}

export function grupAc(ad: string): Promise<GrupId> {
  return invoke("grup_ac", { ad });
}

/** Grubu siler. **Sekmeler kalıyor**, yalnız gruptan çıkıyorlar. */
export function grupSil(id: GrupId): Promise<void> {
  return invoke("grup_sil", { id });
}

/** Sekmeyi gruba alır; `null` gruptan çıkarıyor. */
export function grupAta(sekme: SekmeId, grup: GrupId | null): Promise<void> {
  return invoke("grup_ata", { sekme, grup });
}

/**
 * Grubun alanlarını günceller. Verilmeyen alan **değişmiyor**.
 *
 * `uykuEsigiSn` iki katmanlı: verilmezse dokunulmuyor, `null` verilirse grup
 * genel ayara dönüyor, sayı verilirse eşik o oluyor.
 */
export function grupGuncelle(
  id: GrupId,
  degisiklik: {
    ad?: string;
    renk?: GrupRengi;
    katli?: boolean;
    uykuEsigiSn?: number | null;
  },
): Promise<void> {
  return invoke("grup_guncelle", {
    id,
    ad: degisiklik.ad ?? null,
    renk: degisiklik.renk ?? null,
    katli: degisiklik.katli ?? null,
    // `undefined` "dokunma", `null` "genel ayara dön". `??` ikisini
    // birleştirirdi, o yüzden açık kontrol.
    uykuEsigiSn:
      "uykuEsigiSn" in degisiklik ? degisiklik.uykuEsigiSn : undefined,
  });
}

/* ----------------------------------------------------------------- köprüler */

export function kopruDurumu(): Promise<KopruDurumu[]> {
  return invoke("kopru_durumu");
}

/** Kurulum önbelleklerini düşürüp yeniden arar (ayarlardaki "yeniden ara"). */
export function kopruTazele(): Promise<KopruDurumu[]> {
  return invoke("kopru_tazele");
}

export function muigetGonder(
  url: string,
  dosyaAdi: string | null = null,
  kaynakSayfa: string | null = null,
): Promise<void> {
  return invoke("muiget_gonder", { url, dosyaAdi, kaynakSayfa });
}

/**
 * Motorun kendi indirmesine **bir kerelik** izin verir.
 *
 * `sor` politikasında motor indirmeyi iptal etmişti; iptal edilmiş bir
 * WebView2 indirmesi sürdürülemediği için sekme aynı adrese yeniden
 * gönderiliyor (`tabs::surucu::Surucu::indirme_izin_ver`).
 */
export function indirmeIzinVer(id: SekmeId, url: string): Promise<void> {
  return invoke("indirme_izin_ver", { id, url });
}

/** Yerel medya dosyasını Muiply'a devreder. `file://` adresi de kabul. */
export function muiplyAc(yol: string): Promise<void> {
  return invoke("muiply_ac", { yol });
}

/** Sekmeyi bir Muiwatch oturumuna bağlar (koruma kuralı #7). */
export function muiwatchBagla(id: SekmeId, oda: string): Promise<void> {
  return invoke("muiwatch_bagla", { id, oda });
}

export function muiwatchBirak(id: SekmeId): Promise<void> {
  return invoke("muiwatch_birak", { id });
}

/**
 * Favicon'u `data:image/png;base64,...` olarak verir.
 *
 * Kimlik içeriğin karması, yani aynı kimlik hep aynı baytlar: `useFavicon`
 * sınırsız önbellekleyebiliyor.
 */
export function faviconOku(kimlik: string): Promise<string | null> {
  return invoke("favicon_oku", { kimlik });
}

/* ---------------------------------------------------- engelleme ve temizlik */

export function engelOzeti(): Promise<EngelOzeti> {
  return invoke("engel_ozeti");
}

/** Bu sekmede, bu sayfada kaç istek engellendi. */
export function engelSayaci(id: SekmeId): Promise<number> {
  return invoke("engel_sayaci", { id });
}

/**
 * Veri temizleme. Hem motorun profilini hem Muiren'in kendi depolarını
 * kapsıyor; ayrım kullanıcıya görünmüyor (`docs/IPC.md`).
 */
export function veriTemizle(istek: Temizlik): Promise<TemizlikRaporu> {
  return invoke("veri_temizle", { istek });
}

/**
 * Etkin temanın arka plan görseli, `data:` adresi olarak.
 *
 * `TemaOzeti` içinde **yok** ve olmamalı: görsel megabaytlarca olabiliyor ve
 * özet her tema listesinde, her `tema-degisti` olayında gidiyor.
 */
export function temaArkaplan(): Promise<string | null> {
  return invoke("tema_arkaplan");
}

/**
 * Kabuk tam ekran bir örtü açtı/kapattı (ayarlar, sekme arama).
 *
 * Sekme webview'i ayrı bir native pencere ve kabuğun **üstünde** duruyor;
 * kabuğun çizdiği bir örtü sayfanın altında kalıyor ve kullanıcı ayarlar
 * ekranını hiç göremiyor. CSS `z-index` iki ayrı pencerenin sırasını
 * değiştiremiyor — sayfa gizleniyor, uyutulmuyor.
 */
export function ortuGorunur(acik: boolean): Promise<void> {
  return invoke("ortu_gorunur", { acik });
}
