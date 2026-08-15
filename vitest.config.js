import { defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
    globals: true,
    environment: "jsdom",
    setupFiles: ["./vitest-setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "html"],
      exclude: [
        "node_modules/",
        "src/**/*.test.ts",
        "src/**/*.spec.ts",
        "src/lib/paraglide/**",
        "src-tauri/**",
      ],
    },
  },
  resolve: {
    alias: {
      $lib: "/src/lib",
    },
  },
});
