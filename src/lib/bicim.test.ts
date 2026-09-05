import { describe, expect, it } from "vitest";

import { bayt, bellek, durumOzeti, sure } from "./bicim";

describe("bellek", () => {
  it("1 GB altı MB kalıyor", () => {
    expect(bellek(512)).toBe("512 MB");
    expect(bellek(1023.4)).toBe("1023 MB");
  });

  it("1 GB üstü tek ondalık", () => {
    expect(bellek(3277)).toBe("3.2 GB");
  });

  it("10 GB üstü ondalıksız", () => {
    expect(bellek(12288)).toBe("12 GB");
  });

  it("yaklaşık değerin başında tilde var", () => {
    // Yaklaşık değeri kesin gibi göstermek yok (docs/Bellek.md).
    expect(bellek(3277, true)).toBe("~3.2 GB");
  });

  it("saçma girdi çökmüyor", () => {
    expect(bellek(Number.NaN)).toBe("0 MB");
    expect(bellek(-5)).toBe("0 MB");
  });
});

describe("süre", () => {
  it("basamakları geçiyor", () => {
    expect(sure(45)).toBe("45 sn");
    expect(sure(60)).toBe("1 dk");
    expect(sure(42 * 60)).toBe("42 dk");
    expect(sure(3 * 3600)).toBe("3 sa");
    expect(sure(50 * 3600)).toBe("2 gün");
  });

  it("ek üretmiyor", () => {
    // "Sekme {ad}'de" çalışmıyor (Ayşe'de ama Onur'da) — eksiz kalıp
    // kullanılıyor (CLAUDE.md #11).
    expect(sure(120)).not.toContain("'");
  });
});

describe("durum özeti", () => {
  it("üç kalemi noktayla ayırıyor", () => {
    expect(durumOzeti(14, 22, 9)).toBe("14 uyanık · 22 uyuyan · 9 atılmış");
  });

  it("sıfır kalem yazılmıyor", () => {
    expect(durumOzeti(3, 0, 0)).toBe("3 uyanık");
    expect(durumOzeti(0, 0, 0)).toBe("");
  });
});

describe("bayt", () => {
  it("bayt altında ondalık göstermiyor", () => {
    expect(bayt(0)).toBe("0 B");
    expect(bayt(512)).toBe("512 B");
  });

  it("1024'lük basamak kullanıyor", () => {
    // 1000'lik olsaydı Windows'un gösterdiğinden farklı çıkar ve kullanıcı
    // iki ayrı sayı gördüğünü sanardı.
    expect(bayt(1024)).toBe("1.0 KB");
    expect(bayt(1024 * 1024)).toBe("1.0 MB");
    expect(bayt(1024 * 1024 * 1024)).toBe("1.0 GB");
  });

  it("10'un üstünde tam sayı", () => {
    expect(bayt(15 * 1024 * 1024)).toBe("15 MB");
  });

  it("geçersiz değerde patlamıyor", () => {
    expect(bayt(Number.NaN)).toBe("0 B");
    expect(bayt(-5)).toBe("0 B");
  });
});
