// src/lib/data/settings.ts
import { invoke } from "@tauri-apps/api/core";

export interface AppSettings {
  theme: string;
  language: string;
  auto_preview: boolean;
  max_preview_size: number;
  after_convert: string;
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
}
