
import { paraglideVitePlugin } from "@inlang/paraglide-js";
import { defineConfig } from "vite"; // ← обычный vite, не vitest/config
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [
    tailwindcss(),
    sveltekit(),
    paraglideVitePlugin({
      project: "./project.inlang",
      outdir: "./src/lib/paraglide",
      strategy: ["globalVariable", "baseLocale"],
    }),
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: [
        "**/src-tauri/target/**",
        "**/src-tauri/**",
        "**/.svelte-kit/**",
        "**/coverage/**",
        "**/.flatpak-builder/**",
      ],
    },
  },
  resolve: {
    alias: {
      $lib: "/src/lib",
    },
  },
});
