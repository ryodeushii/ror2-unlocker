import { defineConfig } from "vite";

export default defineConfig(({ mode }) => ({
  base: mode === "web" ? "/ror2-unlocker/" : "/",
  clearScreen: false,
  server: {
    port: mode === "web" ? 1428 : 1427,
    strictPort: true,
    host: "127.0.0.1",
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    outDir: mode === "web" ? "dist-web" : "dist",
    target: ["es2022", "chrome105", "safari15"],
  },
  plugins:
    mode === "web"
      ? [
          {
            name: "web-content-security-policy",
            apply: "build",
            transformIndexHtml() {
              return [
                {
                  tag: "meta",
                  attrs: {
                    "http-equiv": "Content-Security-Policy",
                    content:
                      "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; font-src 'self'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; form-action 'none'",
                  },
                  injectTo: "head-prepend",
                },
              ];
            },
          },
        ]
      : [],
}));
