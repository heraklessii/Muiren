import { describe, expect, it } from "vitest";

import { codecDurumu, raporBasligi, webview2Surumu, yazilimOlusturucu } from "./teshis";

describe("webview2Surumu", () => {
  it("Edg etiketinden sürümü alıyor", () => {
    const ua =
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.3537.71";
    expect(webview2Surumu(ua)).toBe("141.0.3537.71");
  });

  it("Chrome sürümünü WebView2 sürümü sanmıyor", () => {
    // İkisi ayrı numaralar; raporun istediği Edge'inki.
    const ua = "Mozilla/5.0 Chrome/141.0.0.0 Safari/537.36 Edg/141.0.3537.71";
    expect(webview2Surumu(ua)).not.toBe("141.0.0.0");
  });

  it("Edge olmayan tarayıcıda null", () => {
    // `npm run dev` ile Chrome'da açıldığında: uydurulmuş bir sürüm rapora
    // girmemeli.
    expect(webview2Surumu("Mozilla/5.0 Chrome/141.0.0.0 Safari/537.36")).toBeNull();
  });
});

describe("yazilimOlusturucu", () => {
  it("SwiftShader yazılım sayılıyor", () => {
    expect(
      yazilimOlusturucu("ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device))"),
    ).toBe(true);
  });

  it("llvmpipe ve temel görüntü sürücüsü de yazılım", () => {
    expect(yazilimOlusturucu("Mesa/X.org, llvmpipe (LLVM 15.0.7, 256 bits)")).toBe(true);
    expect(yazilimOlusturucu("Microsoft Basic Render Driver")).toBe(true);
  });

  it("gerçek GPU donanım sayılıyor", () => {
    expect(
      yazilimOlusturucu("ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0)"),
    ).toBe(false);
  });

  it("tanımadığı dizeye yanlış alarm vermiyor", () => {
    // Liste kara: donanım adları sonsuz, yazılım oluşturucular sayılı.
    expect(yazilimOlusturucu("Apple M3 Pro")).toBe(false);
  });

  it("büyük harfli sürücü dizesi de tanınıyor", () => {
    // CLAUDE.md #10 istisnası: Türkçe küçültme "SWIFTSHADER" dizesini
    // "swıftshader" yapıp eşleşmeyi düşürüyordu; sürücü dizesi ASCII.
    expect(yazilimOlusturucu("ANGLE (SWIFTSHADER DEVICE)")).toBe(true);
  });
});

describe("codecDurumu", () => {
  it("verimli çözüm donanım", () => {
    expect(codecDurumu({ supported: true, powerEfficient: true })).toBe("donanim");
  });

  it("desteklenen ama verimsiz çözüm yazılım", () => {
    // Açılıyor ama CPU'dan yiyor: kullanıcının "kasıyor" dediği hâl.
    expect(codecDurumu({ supported: true, powerEfficient: false })).toBe("yazilim");
  });

  it("desteklenmeyen codec yok", () => {
    expect(codecDurumu({ supported: false, powerEfficient: false })).toBe("yok");
  });

  it("cevap alınamadıysa yok denmiyor", () => {
    // Bilinmeyeni olumsuz saymak, olmayan bir arızayı rapor etmek olurdu.
    expect(codecDurumu(null)).toBeNull();
  });
});

describe("raporBasligi", () => {
  const girdi = {
    tarih: "2026-09-05",
    cekirdek: 16,
    sistemToplamMb: 32768,
    platform: "Windows",
    gpu: "ANGLE (NVIDIA GeForce RTX 3060)",
    webview2: "141.0.3537.71",
    muiren: "0.1.0",
    profil: "Dengeli",
    surecPolitikasi: "process-per-site açık",
  };

  it("protokolün istediği satırların hepsi var", () => {
    const metin = raporBasligi(girdi);
    for (const alan of [
      "Tarih",
      "Makine",
      "WebView2 Runtime",
      "Muiren",
      "Bellek profili",
      "Süreç politikası",
      "Ağ",
      "Karşılaştırılan",
    ]) {
      expect(metin).toContain(`| ${alan} |`);
    }
  });

  it("ölçülen RAM gigabayta çevriliyor", () => {
    expect(raporBasligi(girdi)).toContain("32 GB RAM");
  });

  it("bilinmeyen alan boş değil, işaretli", () => {
    // `docs/olcumler/README.md`: sürüm alanları boş bırakılmıyor. Boş bırakmak
    // ile "?" yazmak arasındaki fark, raporu okuyanın eksiği görmesi.
    const metin = raporBasligi({ ...girdi, webview2: null, muiren: null });
    expect(metin).toContain("| WebView2 Runtime | ? — elle doldurun |");
    expect(metin).toContain("| Muiren | ? — elle doldurun |");
  });

  it("işlemci modeli her zaman elle isteniyor", () => {
    // Tarayıcı işlemci modelini bilmiyor; bildiğini sanmak rapora uydurma
    // veri sokardı.
    expect(raporBasligi(girdi)).toContain("işlemci modeli: ?");
  });
});
