// src/lib/data/formats.ts

import type { Format } from "$lib/types/format";
import * as Icons from "lucide-svelte";
import { browser } from "$app/environment";

const iconMap: Record<string, any> = {
  FileBraces: Icons.FileBraces,
  FileText: Icons.FileText,
  FileSpreadsheet: Icons.FileSpreadsheet,
  FileCode: Icons.FileCode,
  FileJson: Icons.FileJson,
  Table: Icons.Table,
  AlignLeft: Icons.AlignLeft,
  Grid3x3: Icons.Grid3x3,
  ListOrdered: Icons.ListOrdered,
  Braces: Icons.Braces,
  Globe: Icons.Globe,
};

let _formats: Format[] = [];
let _loaded = false;
let _loadingPromise: Promise<void> | null = null;

export function getFormats(): Format[] {
  return _formats;
}

export function getFormatById(id: string): Format | undefined {
  return _formats.find(f => f.id === id);
}

export function isFormatsLoaded(): boolean {
  return _loaded;
}

export async function loadFormatsData(): Promise<void> {
  if (_loaded) {
    console.log("ℹ️ Formats already loaded, skipping");
    return;
  }

  if (_loadingPromise) {
    console.log("ℹ️ Formats loading in progress, waiting...");
    return _loadingPromise;
  }

  _loadingPromise = (async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const data = await invoke<any[]>("get_formats");

      _formats = data.map(f => ({
        id: f.format_id,
        name: f.name,
        extensions: f.extensions,
        description: f.description,
        icon: iconMap[f.icon] || Icons.File,
        color: f.color,
        glow: f.glow,
        textColor: f.text_color,
        borderHover: f.border_hover,
      }));

      _loaded = true;
      console.log("✅ Formats loaded from DB:", _formats.length);
    } catch (error) {
      console.error("❌ Failed to load formats:", error);
      _formats = [];
      _loaded = false;
    } finally {
      _loadingPromise = null;
    }
  })();

  return _loadingPromise;
}
