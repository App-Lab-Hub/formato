import { paraglideVitePlugin } from "@inlang/paraglide-js";
import { defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { playwright } from "@vitest/browser-playwright";

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
  test: {
    include: ["src/**/*.test.ts", "src/**/*.spec.ts"],
    globals: true,
    browser: {
      enabled: true,
      headless: true,
      provider: playwright(), // <-- 2. ВЫЗЫВАЕМ ФУНКЦИЮ (Типы теперь совпадут!)
      instances: [
        { browser: "chromium" }, // <-- 3. ЗАДАЕМ БРАУЗЕР ЧЕРЕЗ INSTANCES
      ],
    },
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
    conditions: ["browser"],
  },
});
