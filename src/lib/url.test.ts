import { describe, expect, it } from "vitest";

import {
  adresMi,
  alanAdi,
  aramaAdresi,
  coz,
  gosterilecekKonak,
  gosterim,
  punycodeCoz,
  unicodeGosterilebilir,
} from "./url";

describe("adres mi arama mı", () => {
  it("nokta içeren tek parça adres", () => {
    expect(adresMi("ornek.com")).toBe(true);
    expect(adresMi("alt.ornek.com.tr/yol?x=1")).toBe(true);
  });

  it("boşluklu girdi arama", () => {
    expect(adresMi("ornek com")).toBe(false);
    expect(adresMi("en iyi anime siteleri")).toBe(false);
  });

  it("localhost ve port adres", () => {
    expect(adresMi("localhost")).toBe(true);
    expect(adresMi("localhost:3000")).toBe(true);
    expect(adresMi("127.0.0.1:8080")).toBe(true);
  });

  it("soru işareti zorla arama", () => {
    // Tek kaçış yolu: "ornek.com" diye ARAMAK isteyen kullanıcı.
    expect(adresMi("?ornek.com")).toBe(false);
    expect(coz("?ornek.com")).toEqual({ tur: "arama", sorgu: "ornek.com" });
  });

  it("javascript ve data şeması adres sayılmıyor", () => {
    // "Şunu adres çubuğuna yapıştır" saldırısının taşıyıcısı.
    expect(adresMi("javascript:alert(1)")).toBe(false);
    expect(adresMi("data:text/html,<script>x</script>")).toBe(false);
  });

  it("bilinen şemalar adres", () => {
    expect(adresMi("https://ornek.com")).toBe(true);
    expect(adresMi("http://ornek.com")).toBe(true);
    expect(adresMi("file:///C:/bir.html")).toBe(true);
    expect(adresMi("muiren://yeni")).toBe(true);
  });

  it("uzantısız tek kelime arama", () => {
    expect(adresMi("muiren")).toBe(false);
    expect(adresMi("türkçe")).toBe(false);
  });

  it("rakamla biten alan adı arama", () => {
    expect(adresMi("1.2")).toBe(false);
    expect(adresMi("sürüm.3")).toBe(false);
  });

  it("boş girdi arama", () => {
    expect(adresMi("")).toBe(false);
    expect(adresMi("   ")).toBe(false);
  });

  it("şemasız adrese https ekleniyor", () => {
    expect(coz("ornek.com")).toEqual({ tur: "adres", url: "https://ornek.com" });
  });

  it("şemalı adres olduğu gibi kalıyor", () => {
    expect(coz("http://ornek.com/a")).toEqual({ tur: "adres", url: "http://ornek.com/a" });
  });
});

describe("arama adresi", () => {
  it("şablondaki %s dolduruluyor", () => {
    expect(aramaAdresi("bir şey", "https://duckduckgo.com/?q=%s")).toBe(
      "https://duckduckgo.com/?q=bir%20%C5%9Fey",
    );
  });

  it("sorgu dizesini bölecek karakterler kaçışlanıyor", () => {
    expect(aramaAdresi("a&b=c", "https://ara.example/?q=%s")).toBe(
      "https://ara.example/?q=a%26b%3Dc",
    );
  });
});

describe("alan adı vurgusu (güvenlik)", () => {
  it("alt alan adıyla kandırma girişimi", () => {
    // Kullanıcının okuduğu ilk şey "banka.com"; vurgulanması gereken bu değil.
    expect(alanAdi("banka.com.saldirgan.net")).toBe("saldirgan.net");
  });

  it("çok parçalı son ek", () => {
    expect(alanAdi("posta.ornek.com.tr")).toBe("ornek.com.tr");
    expect(alanAdi("www.bbc.co.uk")).toBe("bbc.co.uk");
  });

  it("iki etiketli alan adı olduğu gibi", () => {
    expect(alanAdi("ornek.com")).toBe("ornek.com");
  });

  it("localhost ve IP bölünmüyor", () => {
    expect(alanAdi("localhost")).toBe("localhost");
    expect(alanAdi("192.168.1.1")).toBe("192.168.1.1");
  });

  it("büyük harf ve sondaki nokta normalleşiyor", () => {
    expect(alanAdi("WWW.Ornek.COM.")).toBe("ornek.com");
  });
});

describe("punycode", () => {
  it("bilinen örnekleri çözüyor", () => {
    expect(punycodeCoz("xn--11b5bs3a9aj6g")).toBe("परीक्षा");
    expect(punycodeCoz("xn--e1afmkfd")).toBe("пример");
    expect(punycodeCoz("xn--trke-2oa7j")).toBe("türkçe");
  });

  it("xn-- olmayan etiket olduğu gibi dönüyor", () => {
    expect(punycodeCoz("ornek")).toBe("ornek");
  });

  it("bozuk kodlamada null", () => {
    expect(punycodeCoz("xn--!!!")).toBeNull();
  });
});

describe("konak gösterimi (IDN homograf savunması)", () => {
  it("latin harfli alan adı unicode gösteriliyor", () => {
    // Türkçe alan adları kullanıcının yazdığı gibi görünmeli.
    expect(gosterilecekKonak("xn--trke-2oa7j.com")).toBe("türkçe.com");
  });

  it("kiril harfli alan adı punycode kalıyor", () => {
    // "аpple.com" (Kiril а) ekranda "apple.com"dan ayırt edilemiyor.
    expect(gosterilecekKonak("xn--pple-43d.com")).toBe("xn--pple-43d.com");
  });

  it("latin dışı yazımlar punycode kalıyor", () => {
    expect(gosterilecekKonak("xn--e1afmkfd.xn--p1ai")).toBe("xn--e1afmkfd.xn--p1ai");
  });

  it("gösterilebilirlik kuralı", () => {
    expect(unicodeGosterilebilir("türke")).toBe(true);
    expect(unicodeGosterilebilir("аpple")).toBe(false);
    expect(unicodeGosterilebilir("пример")).toBe(false);
  });
});

describe("gösterim parçaları", () => {
  it("alt alan adı soluk, kayıtlanabilir alan vurgulu", () => {
    const g = gosterim("https://banka.com.saldirgan.net/giris?x=1");
    expect(g.onEk).toBe("banka.com.");
    expect(g.alan).toBe("saldirgan.net");
    expect(g.sonEk).toBe("/giris?x=1");
    expect(g.guvenli).toBe(true);
  });

  it("http güvenli değil", () => {
    expect(gosterim("http://ornek.com/").guvenli).toBe(false);
  });

  it("port alan adına yapışıyor", () => {
    expect(gosterim("http://localhost:1420/a").alan).toBe("localhost:1420");
  });

  it("yeni sekme sayfası boş adres çubuğu", () => {
    expect(gosterim("muiren://yeni").ham).toBe("");
  });

  it("yarım girdi ham dönüyor", () => {
    // Kullanıcı yazarken adres çubuğunda her zaman yarım bir dize var.
    const g = gosterim("htt");
    expect(g.ham).toBe("htt");
    expect(g.alan).toBe("");
  });

  it("izinsiz şema ayrıştırılmıyor", () => {
    expect(gosterim("javascript:alert(1)").alan).toBe("");
  });
});
