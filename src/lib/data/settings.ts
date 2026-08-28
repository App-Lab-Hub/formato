// src/lib/data/settings.ts
import { invalidateAll } from "$app/navigation";
import { invoke } from "@tauri-apps/api/core";
import { browser } from "$app/environment";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export interface AppSettings {
  theme: string;
  language: string;
  auto_preview: boolean;
  max_preview_size: number;
  show_extensions: boolean;
  enable_cache: boolean;
  enable_archive: boolean;
  archive_format: string;
  synthesis_model: Record<string, string>; // { "ru": "ru_RU-dmitri-medium", "en": "en_US-lessac-medium" }
  recognition_model: string;
}

let settings: AppSettings | null = null;
let isLoaded = false;

export async function loadSettings(): Promise<AppSettings> {
  if (isLoaded && settings) return settings;

  settings = await invoke<AppSettings>("get_settings");
  isLoaded = true;
  return settings;
}

export function getSettings(): AppSettings {
  if (!settings) {
    throw new Error("Settings not loaded. Call loadSettings() first.");
  }
  return settings;
}

export function isSettingsLoaded(): boolean {
  return isLoaded;
}

export async function saveSettings(newSettings: AppSettings): Promise<void> {
  settings = newSettings;
  await invoke("save_settings", { settings: newSettings });
  invalidateAll();
}

// Получить системную тему с проверкой
function getSystemTheme(): "dark" | "light" {
  if (!browser) return "dark";

  try {
    const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const isLight = window.matchMedia("(prefers-color-scheme: light)").matches;

    if (isDark) return "dark";
    if (isLight) return "light";

    // console.log("⚠️ Unknown system theme, falling back to dark");
    return "dark";
  } catch (error) {
    console.warn("Failed to detect system theme, falling back to dark:", error);
    return "dark";
  }
}

// Получить цвета для фона окна в зависимости от темы
function getWindowBackgroundColors(theme: string): {
  r: number;
  g: number;
  b: number;
  a: number;
} {
  let isDark: boolean;

  if (theme === "system") {
    isDark = getSystemTheme() === "dark";
  } else {
    isDark = theme === "dark";
  }

  if (isDark) {
    return { r: 20, g: 10, b: 41, a: 255 };
  } else {
    return { r: 239, g: 231, b: 255, a: 255 };
  }
}

async function setWindowBackground(theme: string) {
  try {
    const colors = getWindowBackgroundColors(theme);
    await invoke("set_window_background", {
      r: colors.r,
      g: colors.g,
      b: colors.b,
      a: colors.a,
    });
  } catch (e) {
    console.warn("Failed to set window background:", e);
  }
}

export function applyTheme(theme: string, emit: boolean = true): void {
  if (!browser) return;

  let isDark: boolean;

  if (theme === "system") {
    isDark = getSystemTheme() === "dark";
  } else {
    isDark = theme === "dark";
  }

  if (isDark) {
    document.documentElement.classList.remove("light");
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
    document.documentElement.classList.add("light");
  }

  setWindowBackground(theme);

  if (emit) {
    WebviewWindow.getAll()
      .then(windows => {
        for (const win of windows) {
          if (win.label.startsWith("preview-")) {
            win.emit("theme-changed", theme);
          }
        }
      })
      .catch(() => {});
  }
}

export function getEffectiveTheme(settings: AppSettings): "light" | "dark" {
  if (settings.theme === "system") {
    return getSystemTheme();
  }
  return settings.theme as "light" | "dark";
}

export function watchSystemTheme(
  callback: (isDark: boolean) => void,
): () => void {
  if (!browser) return () => {};

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = (e: MediaQueryListEvent) => {
    const isDark = getSystemTheme() === "dark";
    // console.log("🔄 System theme changed:", isDark ? "dark" : "light");
    callback(isDark);
  };

  mediaQuery.addEventListener("change", handler);

  const initialIsDark = getSystemTheme() === "dark";
  // console.log(
  //   "👀 Watching system theme, initial:",
  //   initialIsDark ? "dark" : "light",
  // );

  return () => {
    // console.log("👋 Stopped watching system theme");
    mediaQuery.removeEventListener("change", handler);
  };
}

// Получить модели для синтеза речи
export function getSynthesisModel(lang: string): string {
  const settings = getSettings();
  return settings.synthesis_model[lang] || settings.synthesis_model["en"];
}
