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
import type { FormatDB } from "$lib/types/format";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async ({ url }) => {
  if (url.pathname.startsWith("/preview")) {
    return { formats: [], settings: {} as any };
  }
  if (!isFormatsLoaded()) await loadFormatsData();
  await loadSettings();

  const settings = getSettings();
  applyTheme(settings.theme);
  setLocale(settings.language as "en" | "ru", { reload: false });

  return {
    formats: getFormats(),
    // formats: [] as FormatDB[],

    settings,
  };
};
