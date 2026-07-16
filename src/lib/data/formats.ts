// src/lib/data/formats.ts

import type { Format } from "$lib/types/format";
import * as Icons from "@lucide/svelte";
import { browser } from "$app/environment";

// === Импорты из svelte-icons ===
// Font Awesome
import {
  FaFilePdf,
  FaFileWord,
  FaFileExcel,
  FaFileAudio,
  FaFileVideo,
  FaFileImage,
  FaFileAlt,
  // @ts-ignore
} from "svelte-icons/fa";

// Material Design
import {
  MdAudiotrack,
  MdVideocam,
  MdImage,
  MdDescription,
  MdTextFields,
  // @ts-ignore
} from "svelte-icons/md";

// Octicons (GitHub)
import {
  GoFile,
  // @ts-ignore
} from "svelte-icons/go";

// Typicons
import {
  TiDocument,
  TiDocumentText,
  // @ts-ignore
} from "svelte-icons/ti";

// Devicons
import {
  DiHtml5,
  // @ts-ignore
} from "svelte-icons/di";

const iconMap: Record<string, any> = {
  // ============================================
  // === ТЕКСТОВЫЕ И КОНФИГУРАЦИОННЫЕ (Lucide) ===
  // ============================================
  FileBraces: Icons.FileBraces, // JSON
  FileText: Icons.FileText, // YAML
  FileSpreadsheet: Icons.FileSpreadsheet, // CSV
  FileCode: Icons.FileCode, // XML
  AlignLeft: Icons.TextAlignStart, // TOML
  ListOrdered: Icons.ListOrdered, // INI
  Braces: Icons.Braces, // Markdown
  Globe: Icons.Globe, // HTML

  // ============================================
  // === ДОКУМЕНТЫ (svelte-icons) ===
  // ============================================
  FilePdf: FaFilePdf, // PDF
  FileWord: FaFileWord, // DOCX
  FileExcel: FaFileExcel, // XLSX
  FileAlt: FaFileAlt, // TXT
  FileRtf: MdTextFields, // RTF
  FileOdt: MdDescription, // ODT

  // ============================================
  // === ИЗОБРАЖЕНИЯ (svelte-icons) ===
  // ============================================
  FileJpg: MdImage, // JPG
  FilePng: FaFileImage, // PNG
  FileWebp: Icons.Image, // WEBP
  FileAvif: FaFileImage, // AVIF

  // ============================================
  // === АУДИО (svelte-icons) ===
  // ============================================
  FileMp3: MdAudiotrack, // MP3
  FileWav: FaFileAudio, // WAV

  // ============================================
  // === ВИДЕО (svelte-icons) ===
  // ============================================
  FileMp4: MdVideocam, // MP4
  FileMov: FaFileVideo, // MOV

  // === ЗАПАСНАЯ ===
  default: Icons.File,
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
        icon: iconMap[f.icon] || iconMap.default || Icons.File,
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
