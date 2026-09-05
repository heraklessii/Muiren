import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import "./styles.css";

// Sahte backend — yalnız geliştirme sunucusunda ve yalnız `?sahte` varsa
// (`src/dev/sahte.ts`). `import.meta.env.DEV` derlemede `false` oluyor ve
// dinamik içe aktarım elendiği için üretim paketine tek bayt girmiyor.
if (import.meta.env.DEV && new URLSearchParams(location.search).has("sahte")) {
  const { kur } = await import("./dev/sahte");
  kur();
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
