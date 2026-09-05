//! Tauri komutları — **ince sarmalayıcılar**.
//!
//! **Buraya iş mantığı yazılmıyor** (CLAUDE.md #1). Gerekçe soyut değil: aynı
//! karara bellek gözcüsünün döngüsünden de gelinebiliyor ve iki kopya
//! birbirinden sessizce ayrışır. Bir komutun gövdesi tek satırdan uzunsa
//! büyük ihtimalle yanlış yerde.
//!
//! Sözleşme `docs/IPC.md` içinde ve **önce orası** yazılıyor (CLAUDE.md #3).
//!
//! ## Webview yaratabilen komut `async` olmak ZORUNDA
//!
//! Eşzamanlı (`async` olmayan) bir Tauri komutu **ana iş parçacığında ve
//! kabuk webview'inin `WebMessageReceived` geri çağrısının içinde** koşuyor.
//! Webview yaratma (`Window::add_child`) o noktada kilitleniyor: wry,
//! WebView2 ortamı/denetleyicisi hazır olana kadar iç içe bir mesaj döngüsü
//! (`webview2_com::wait_with_pump`) çeviriyor, ama WebView2 zaten bir geri
//! çağrının içindeyken ikinci bir geri çağrıyı teslim etmiyor. Beklenen cevap
//! hiç gelmiyor; ana iş parçacığı sonsuza kadar bekliyor.
//!
//! Belirtisi tam olarak kullanıcının gördüğü şey: pencere hâlâ boyanıyor ve
//! Windows onu "yanıt veriyor" sayıyor, ama arayüz ölü — ne adres çubuğu
//! yazıyor ne sekmeye tıklanıyor. Tauri bunu kendi belgelerinde de yazıyor
//! (`Window::add_child`: "deadlocks when used in a synchronous command or
//! event handlers").
//!
//! Bu yüzden çağrı ağacı `Motor::sekme_ac`e ulaşabilen her komut `async`:
//! `async` komut Tauri'nin çalışma zamanında, ayrı bir iş parçacığında
//! koşuyor ve oradan yapılan yaratma isteği olay döngüsüne **kuyruklanıyor**.
//! Motorun kendi olay geri çağrıları da aynı tuzağa düşüyor; onlar
//! `tabs::surucu` içinde `std::thread::spawn` ile çıkıyor
//! (`Dinleyici::yeni_pencere`, `Dinleyici::kisayol`).
//!
//! Sıra garantisi gereken komutlar (`icerik_alani`, `ortu_gorunur`) bilerek
//! eşzamanlı kalıyor: ikisi de webview yaratmıyor ve `async` olsalardı iki
//! ardışık çağrı birbirini geçebilirdi.

pub mod ayarlar;
pub mod bellek;
pub mod gecmis;
pub mod gezinme;
pub mod kopru;
pub mod tabs;
pub mod tema;
