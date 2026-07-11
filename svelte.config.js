import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      // Tauri uses a custom protocol, so we don't need fallback
      fallback: "index.html",
    }),
    // Tauri supports SPA mode with custom protocol
    csrf: {
      checkOrigin: false,
    },
  },
};

export default config;
