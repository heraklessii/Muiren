//! Sekme sırası ve ağacı — saf.
//!
//! **Kural (`docs/Sekmeler.md`):** bu dosyada `motor` çağrısı, zaman okuma
//! veya IO yok. Girdi bir düğüm listesi, çıktı yeni sıra ya da bir id.
//!
//! Sekmeler düz bir liste ama `ebeveyn` alanı bir ağaç kuruyor: bir sekmeden
//! link açtığında yeni sekme onun çocuğu oluyor ve **hemen sağına** giriyor,
//! listenin sonuna değil. 40 sekme arasında çalışan biri için tek gerçek fark
//! bu — açtığın şey açtığın yerin yanında duruyor.
//!
//! Alt ağacın **bitişik** olması bir değişmez (invariant): çocuk her zaman
//! ebeveynin alt ağacının hemen ardına ekleniyor ve [`tasi`] bir sekmeyi
//! taşırken çocuklarını da taşıyor. [`alt_agac`] bu değişmeze dayanıyor.

use std::collections::HashSet;

use super::SekmeId;

/// Ağaç kararları için gereken en küçük sekme bilgisi.
///
/// Bilinçli olarak `Sekme`nin kendisi değil: bu modülün başlığa, URL'e, duruma
/// ya da zamana ihtiyacı yok ve olmamalı. Testler bu yüzden üç satır.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dugum {
    pub id: SekmeId,
    pub ebeveyn: Option<SekmeId>,
    pub sabit: bool,
}

impl Dugum {
    pub fn yeni(id: SekmeId, ebeveyn: Option<SekmeId>) -> Self {
        Dugum {
            id,
            ebeveyn,
            sabit: false,
        }
    }
}

fn indeks(dugumler: &[Dugum], id: SekmeId) -> Option<usize> {
    dugumler.iter().position(|d| d.id == id)
}

/// Sabitlenmiş sekmeler bloğunun bittiği indeks.
///
/// Sabitlenmişler her zaman solda; sabitlenmemiş bir sekme aralarına giremez.
pub fn sabit_siniri(dugumler: &[Dugum]) -> usize {
    dugumler.iter().take_while(|d| d.sabit).count()
}

/// `kok` ve bütün torunları, listedeki sırayla.
pub fn alt_agac(dugumler: &[Dugum], kok: SekmeId) -> Vec<SekmeId> {
    let Some(bas) = indeks(dugumler, kok) else {
        return Vec::new();
    };

    let mut kume: HashSet<SekmeId> = HashSet::new();
    kume.insert(kok);
    let mut sonuc = vec![kok];

    for d in &dugumler[bas + 1..] {
        match d.ebeveyn {
            Some(e) if kume.contains(&e) => {
                kume.insert(d.id);
                sonuc.push(d.id);
            }
            // Alt ağaç bitişik: aileye ait olmayan ilk düğümde bitiyor.
            _ => break,
        }
    }

    sonuc
}

/// Yeni sekme hangi indekse girer.
///
/// - Ebeveyn yoksa: listenin sonuna.
/// - Ebeveyn varsa: ebeveynin alt ağacının hemen ardına. Yani kardeşleri varsa
///   en son kardeşin (ve onun torunlarının) sağına, yoksa ebeveynin hemen
///   sağına.
pub fn ekle_konumu(dugumler: &[Dugum], ebeveyn: Option<SekmeId>) -> usize {
    match ebeveyn.and_then(|e| indeks(dugumler, e)) {
        Some(i) => i + alt_agac(dugumler, dugumler[i].id).len(),
        None => dugumler.len(),
    }
}

/// `id` kapandıktan sonra hangi sekme etkin olur.
///
/// Sırayla: sağdaki kardeş → soldaki kardeş → ebeveyn → sağdaki sekme →
/// soldaki sekme. Chrome'un davranışı doğrudan "sağdaki sekme"; ebeveyne
/// dönmek ağaç modelinde çok daha az şaşırtıyor, çünkü bir linki yeni sekmede
/// açıp kapattığında geldiğin yere dönüyorsun.
///
/// Liste kapanmadan **önceki** hâli. Dönüş `None` ise kapanacak son sekme.
pub fn kapatinca_etkin(dugumler: &[Dugum], id: SekmeId) -> Option<SekmeId> {
    let i = indeks(dugumler, id)?;
    if dugumler.len() == 1 {
        return None;
    }
    let kapanan = dugumler[i];

    let kardes_mi = |d: &Dugum| d.ebeveyn == kapanan.ebeveyn && d.id != id;

    if let Some(d) = dugumler[i + 1..].iter().find(|d| kardes_mi(d)) {
        return Some(d.id);
    }
    if let Some(d) = dugumler[..i].iter().rev().find(|d| kardes_mi(d)) {
        return Some(d.id);
    }
    if let Some(e) = kapanan.ebeveyn {
        if indeks(dugumler, e).is_some() {
            return Some(e);
        }
    }
    dugumler
        .get(i + 1)
        .or_else(|| i.checked_sub(1).and_then(|j| dugumler.get(j)))
        .map(|d| d.id)
}

/// Kapanan sekmenin çocukları büyükanne/babaya bağlanıyor.
///
/// Alternatif "çocukları da kapat" olurdu ve bir linki yanlışlıkla kapatan
/// kullanıcının açtığı her şeyi silerdi. Toplu kapama ayrı bir iş
/// ([`alt_agac`] ile).
pub fn kapatinca_ebeveyn_devri(dugumler: &mut [Dugum], id: SekmeId) {
    let Some(i) = indeks(dugumler, id) else {
        return;
    };
    let yeni_ebeveyn = dugumler[i].ebeveyn;
    for d in dugumler.iter_mut() {
        if d.ebeveyn == Some(id) {
            d.ebeveyn = yeni_ebeveyn;
        }
    }
}

/// Sürükle-bırak. Bir sekme taşınırken **çocukları da** taşınıyor.
///
/// `hedef`, taşımadan önceki listedeki indeks. Sonuç yeni sıra.
pub fn tasi(dugumler: &[Dugum], id: SekmeId, hedef: usize) -> Vec<Dugum> {
    let Some(bas) = indeks(dugumler, id) else {
        return dugumler.to_vec();
    };
    let tasinan: Vec<SekmeId> = alt_agac(dugumler, id);
    let kume: HashSet<SekmeId> = tasinan.iter().copied().collect();

    let tasinan_dugumler: Vec<Dugum> = dugumler
        .iter()
        .filter(|d| kume.contains(&d.id))
        .copied()
        .collect();
    let mut kalan: Vec<Dugum> = dugumler
        .iter()
        .filter(|d| !kume.contains(&d.id))
        .copied()
        .collect();

    // Hedef indeks, taşınan blok listeden çıkarıldıktan sonra kayıyor.
    let duzeltilmis = if hedef > bas {
        hedef.saturating_sub(tasinan.len() - 1)
    } else {
        hedef
    };

    // Sabitlenmişlerin arasına girilmiyor, aralarından da çıkılmıyor.
    let sinir = sabit_siniri(&kalan);
    let konum = if tasinan_dugumler[0].sabit {
        duzeltilmis.min(sinir)
    } else {
        duzeltilmis.max(sinir).min(kalan.len())
    };

    for (n, d) in tasinan_dugumler.into_iter().enumerate() {
        kalan.insert(konum + n, d);
    }
    kalan
}

#[cfg(test)]
mod testler {
    use super::*;

    fn liste(ciftler: &[(u64, Option<u64>)]) -> Vec<Dugum> {
        ciftler.iter().map(|(id, e)| Dugum::yeni(*id, *e)).collect()
    }

    fn idler(dugumler: &[Dugum]) -> Vec<u64> {
        dugumler.iter().map(|d| d.id).collect()
    }

    #[test]
    fn ebeveynsiz_sekme_sona_giriyor() {
        let d = liste(&[(1, None), (2, None)]);
        assert_eq!(ekle_konumu(&d, None), 2);
    }

    #[test]
    fn cocuk_ebeveynin_hemen_sagina_giriyor() {
        let d = liste(&[(1, None), (2, None), (3, None)]);
        assert_eq!(ekle_konumu(&d, Some(1)), 1);
    }

    #[test]
    fn ikinci_cocuk_ilk_kardesin_sagina_giriyor() {
        // 1'in çocuğu 4 var; ikinci çocuk 4'ün sağına gitmeli, 1'in değil.
        let d = liste(&[(1, None), (4, Some(1)), (2, None)]);
        assert_eq!(ekle_konumu(&d, Some(1)), 2);
    }

    #[test]
    fn torunlar_atlaniyor() {
        // 1 → 4 → 5 zinciri. 1'in yeni çocuğu 5'in de sağına gitmeli, yoksa
        // yeni sekme torunun ortasına düşer ve alt ağaç bitişikliği bozulur.
        let d = liste(&[(1, None), (4, Some(1)), (5, Some(4)), (2, None)]);
        assert_eq!(ekle_konumu(&d, Some(1)), 3);
    }

    #[test]
    fn alt_agac_yabanci_dugumde_duruyor() {
        let d = liste(&[
            (1, None),
            (4, Some(1)),
            (5, Some(4)),
            (2, None),
            (6, Some(2)),
        ]);
        assert_eq!(alt_agac(&d, 1), vec![1, 4, 5]);
        assert_eq!(alt_agac(&d, 2), vec![2, 6]);
        assert_eq!(alt_agac(&d, 5), vec![5]);
    }

    #[test]
    fn kapaninca_sagdaki_kardes_etkin_oluyor() {
        let d = liste(&[(1, None), (2, None), (3, None)]);
        assert_eq!(kapatinca_etkin(&d, 2), Some(3));
    }

    #[test]
    fn sagda_kardes_yoksa_soldaki() {
        let d = liste(&[(1, None), (2, None)]);
        assert_eq!(kapatinca_etkin(&d, 2), Some(1));
    }

    #[test]
    fn kardes_yoksa_ebeveyne_donuyor() {
        // Bir linki yeni sekmede açıp kapattığında geldiğin yere dönüyorsun.
        let d = liste(&[(1, None), (4, Some(1)), (2, None)]);
        assert_eq!(kapatinca_etkin(&d, 4), Some(1));
    }

    #[test]
    fn ebeveyn_de_yoksa_sagdaki_sekme() {
        let d = liste(&[(9, Some(99)), (2, None)]);
        assert_eq!(kapatinca_etkin(&d, 9), Some(2));
    }

    #[test]
    fn son_sekme_kapaninca_none() {
        let d = liste(&[(1, None)]);
        assert_eq!(kapatinca_etkin(&d, 1), None);
    }

    #[test]
    fn cocuklar_buyukanneye_devrediliyor() {
        let mut d = liste(&[(1, None), (4, Some(1)), (5, Some(4))]);
        kapatinca_ebeveyn_devri(&mut d, 4);
        assert_eq!(d[2].ebeveyn, Some(1));
    }

    #[test]
    fn tasima_cocuklari_da_goturuyor() {
        let d = liste(&[(1, None), (4, Some(1)), (2, None), (3, None)]);
        assert_eq!(idler(&tasi(&d, 1, 3)), vec![2, 3, 1, 4]);
    }

    #[test]
    fn sola_tasima() {
        let d = liste(&[(1, None), (2, None), (3, None), (7, Some(3))]);
        assert_eq!(idler(&tasi(&d, 3, 0)), vec![3, 7, 1, 2]);
    }

    #[test]
    fn sabitlenmemis_sekme_sabit_blogun_arasina_giremiyor() {
        let mut d = liste(&[(1, None), (2, None), (3, None)]);
        d[0].sabit = true;
        d[1].sabit = true;
        assert_eq!(idler(&tasi(&d, 3, 0)), vec![1, 2, 3]);
    }

    #[test]
    fn sabit_sekme_blogun_disina_cikamiyor() {
        let mut d = liste(&[(1, None), (2, None), (3, None)]);
        d[0].sabit = true;
        assert_eq!(idler(&tasi(&d, 1, 2)), vec![1, 2, 3]);
    }

    #[test]
    fn olmayan_sekme_sirayi_bozmuyor() {
        let d = liste(&[(1, None), (2, None)]);
        assert_eq!(idler(&tasi(&d, 42, 0)), vec![1, 2]);
        assert_eq!(kapatinca_etkin(&d, 42), None);
        assert_eq!(alt_agac(&d, 42), Vec::<SekmeId>::new());
    }
}
