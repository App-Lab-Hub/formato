// src/lib/services/formats.ts

import { invoke } from "@tauri-apps/api/core";
import * as Icons from "lucide-svelte";
import {
  formats,
  formatsLoading,
  formatsError,
  formatsLoaded,
} from "$lib/stores/formats";
import type { FormatDB, Format } from "$lib/types/format";
import { get } from "svelte/store";

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

function mapFormat(data: FormatDB): Format {
  return {
    id: data.format_id,
    name: data.name,
    extensions: data.extensions,
    description: data.description,
    icon: iconMap[data.icon] || Icons.File,
    color: data.color,
    glow: data.glow,
    textColor: data.text_color,
    borderHover: data.border_hover,
  };
}

export async function loadFormats(): Promise<void> {
  // 1. Проверяем store
  if (get(formatsLoaded)) {
    console.log("ℹ️ Formats already loaded in store, skipping");
    return;
  }

  // 2. Проверяем, не идёт ли загрузка
  if (get(formatsLoading)) {
    console.log("ℹ️ Formats loading in progress, skipping");
    return;
  }

  formatsLoading.set(true);
  formatsError.set(null);

  try {
    const data = await invoke<FormatDB[]>("get_formats");
    const formatted = data.map(mapFormat);
    formats.set(formatted);
    formatsLoaded.set(true);
    console.log("✅ Formats loaded from DB:", formatted.length);
  } catch (error) {
    console.error("❌ Failed to load formats:", error);
    formatsError.set(String(error));
  } finally {
    formatsLoading.set(false);
  }
}

export function getFormatFromStore(id: string): Format | undefined {
  return get(formats).find(format => format.id === id);
}

export async function getFormatById(id: string): Promise<Format | null> {
  try {
    const data = await invoke<FormatDB>("get_format_by_id", { formatId: id });
    return mapFormat(data);
  } catch (error) {
    console.error(`❌ Failed to load format ${id}:`, error);
    return null;
  }
}
