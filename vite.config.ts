import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  server: { port: 1427, strictPort: true, host: "127.0.0.1" },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: { target: ["es2022", "chrome105", "safari15"] },
});
