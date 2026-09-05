// `vitest/config` uzerinden: `vite`in kendi `defineConfig`i `test` alanini
// tanimiyor. Ikisi ayni yapilandirmayi uretiyor (Muiply ile ayni gerekce).
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],

  // Tauri CLI'in urettigi hatalari gizlememek icin.
  clearScreen: false,

  server: {
    // Tauri sabit portu bekliyor; port doluysa fail etmeli ki pencere yanlis
    // bir adrese baglanmasin.
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  build: {
    sourcemap: true,
  },

  test: {
    include: ["src/**/*.test.ts"],
    // Testlenen her sey saf: adres/arama ayrimi, suzme, bicimleme. Karar
    // testleri Rust tarafinda (docs/Frontend.md "Test").
    environment: "node",
  },
});
