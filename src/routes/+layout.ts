// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const ssr = false;
export const prerender = false; // Важно для динамических приложений в Tauri

// src/routes/+layout.ts
import {
  loadFormatsData,
  getFormats,
  isFormatsLoaded,
} from "$lib/data/formats";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async () => {
  // Загружаем форматы ДО рендера
  if (!isFormatsLoaded()) {
    await loadFormatsData();
  }

  return {
    formats: getFormats(),
  };
};
