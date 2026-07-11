import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

// @tauri-apps/cli handles the dev server for Tauri
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [sveltekit()],

  // Tauri expects a fixed port for dev
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Don't watch the Rust source
      ignored: ["**/src-tauri/**"],
    },
  },
});
