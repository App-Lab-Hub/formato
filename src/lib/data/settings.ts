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

// src/lib/data/settings.ts
export function applyTheme(theme: string): void {
  if (!browser) return;

  const isDark =
    theme === "dark" ||
    (theme === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  if (isDark) {
    document.documentElement.classList.remove("light");
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
    document.documentElement.classList.add("light");
  }

  // Для Monaco Editor
  // @ts-ignore
  if (window.monaco?.editor) {
    // @ts-ignore
    monaco.editor.setTheme(isDark ? "vs-dark" : "vs");
  }

  // Отправляем событие во все preview окна
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

// Получить актуальную тему (с учетом system)
export function getEffectiveTheme(settings: AppSettings): "light" | "dark" {
  if (settings.theme === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  return settings.theme as "light" | "dark";
}

// Следить за изменением системной темы
export function watchSystemTheme(
  callback: (isDark: boolean) => void,
): () => void {
  if (!browser) return () => {};

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = (e: MediaQueryListEvent) => callback(e.matches);

  mediaQuery.addEventListener("change", handler);
  return () => mediaQuery.removeEventListener("change", handler);
}
