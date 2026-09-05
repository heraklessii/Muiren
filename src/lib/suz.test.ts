import { describe, expect, it } from "vitest";

import type { SekmeOzeti } from "../ipc/tipler";
import { kucult, sirala, suz } from "./suz";

function sekme(id: number, gorunenAd: string, url: string, baslik = ""): SekmeOzeti {
  return {
    id,
    url,
    baslik,
    gorunenAd,
    favicon: null,
    durum: "atilmis",
    ebeveyn: null,
    derinlik: 0,
    sabit: false,
    uyutmaIstisnasi: false,
    sesCaliyor: false,
    sessiz: false,
    formDolu: false,
    gizli: false,
    grup: null,
    katli: false,
    grupUykuEsigiSn: null,
    tamEkran: false,
    yukleniyor: false,
    geriVar: false,
    ileriVar: false,
    etkin: false,
    bostaSn: 0,
  };
}

const liste = [
  sekme(1, "İSTANBUL Haber", "https://haber.example/istanbul"),
  sekme(2, "YouTube", "https://youtube.example/izle", "Anime listesi"),
  sekme(3, "Işık Üniversitesi", "https://isik.example"),
  sekme(4, "Dokümantasyon", "https://docs.example"),
];

describe("türkçe küçültme", () => {
  it("büyük İ küçük i oluyor", () => {
    // Düz toLowerCase() burada "i̇stanbul" üretiyor ve eşleşme kaçıyor.
    expect(kucult("İSTANBUL")).toBe("istanbul");
  });

  it("büyük I küçük ı oluyor", () => {
    expect(kucult("IŞIK")).toBe("ışık");
  });
});

describe("süzme", () => {
  it("türkçe büyük harfli başlık eşleşiyor", () => {
    expect(suz(liste, "istanbul").map((s) => s.id)).toEqual([1]);
  });

  it("ışık araması ı ile eşleşiyor", () => {
    expect(suz(liste, "ışık").map((s) => s.id)).toEqual([3]);
  });

  it("adreste de arıyor", () => {
    expect(suz(liste, "youtube.example").map((s) => s.id)).toEqual([2]);
  });

  it("başlıkta da arıyor", () => {
    expect(suz(liste, "anime").map((s) => s.id)).toEqual([2]);
  });

  it("çok parçalı sorguda hepsi eşleşmeli", () => {
    // "yt anime" yazan biri iki parçayı da geçen sekmeyi arıyor.
    expect(suz(liste, "youtube anime").map((s) => s.id)).toEqual([2]);
    expect(suz(liste, "youtube istanbul")).toEqual([]);
  });

  it("boş sorgu her şeyi döndürüyor", () => {
    expect(suz(liste, "   ")).toHaveLength(4);
  });
});

describe("sıralama", () => {
  it("başlıkla başlayanlar öne geçiyor", () => {
    const sonuc = sirala([liste[3], liste[1]], "you");
    expect(sonuc.map((s) => s.id)).toEqual([2, 4]);
  });

  it("eşitlikte sekme çubuğu sırası korunuyor", () => {
    expect(sirala(liste, "e").map((s) => s.id)).toEqual([1, 2, 3, 4]);
  });
});
