// src/routes/+layout.ts
// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const ssr = false;
export const prerender = true;

import {
  loadFormatsData,
  getFormats,
  isFormatsLoaded,
} from "$lib/data/formats";
import { loadSettings, getSettings, applyTheme } from "$lib/data/settings";
import { setLocale } from "$lib/paraglide/runtime";
import type { LayoutLoad } from "./$types";
import { getModelsStatus } from "$lib/data/models";

export const load: LayoutLoad = async ({ url }) => {
  if (url.pathname.startsWith("/preview")) {
    return { formats: [], settings: {} as any, modelsStatus: null };
  }

  if (!isFormatsLoaded()) await loadFormatsData();
  await loadSettings();

  const settings = getSettings();
  applyTheme(settings.theme);
  setLocale(settings.language as "en" | "ru", { reload: false });

  // ✅ Загружаем статус моделей
  let modelsStatus = null;
  try {
    modelsStatus = await getModelsStatus();
    console.log("✅ Models status loaded in root layout:", modelsStatus);
  } catch (e) {
    console.error("❌ Failed to load models status in root layout:", e);
  }

  return {
    formats: getFormats(),
    settings,
    modelsStatus,
  };
};
