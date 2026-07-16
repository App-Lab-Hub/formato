// src/lib/data/formats.ts

import type { Format } from "$lib/types/format";
import * as Icons from "@lucide/svelte";
import { browser } from "$app/environment";

// === Импорты из svelte-icons (только то, что есть в твоем package) ===

// Font Awesome (основные файловые иконки)
import {
  FaFilePdf,
  FaFileWord,
  FaFileExcel,
  FaFileAudio,
  FaFileVideo,
  FaFileImage,
  FaFileAlt,
  FaFileCode as FaFileCodeIcon,
  FaFileArchive,
  FaFile,
  // @ts-ignore
} from "svelte-icons/fa";

// Material Design (специфичные иконки для форматов)
import {
  MdAudiotrack, // MP3
  MdVideocam, // MP4
  MdImage, // JPG
  MdDescription, // ODT
  MdTableChart, // XLSX (альтернатива)
  MdCode, // XML (альтернатива)
  MdDataObject, // JSON (альтернатива)
  MdSettings, // INI (альтернатива)
  MdTextFields, // RTF
  MdPictureAsPdf, // PDF (альтернатива)
  MdMusicNote, // MP3 (альтернатива)
  MdMovie, // MOV (альтернатива)
  // @ts-ignore
} from "svelte-icons/md";

// Octicons (GitHub) - технические иконки
import {
  GoFileCode,
  GoFileMedia,
  GoFileMusic,
  GoFileVideo,
  GoFilePdf,
  GoFile,
  // @ts-ignore
} from "svelte-icons/go";

// Typicons
import {
  TiDocumentText,
  TiDocument,
  TiImage,
  TiVideo,
  TiMusic,
  TiCode,
  TiChartBar,
  TiClipboard,
  TiBook,
  TiFolder,
  // @ts-ignore
} from "svelte-icons/ti";

// Devicons (для технических форматов)
import {
  DiHtml5, // HTML
  DiCss3, // CSS (если добавишь)
  DiJavascript, // JS (если добавишь)
  DiPython, // Python (если добавишь)
  DiRuby, // Ruby (если добавишь)
  DiRust, // Rust (если добавишь)
  DiPhp, // PHP (если добавишь)
  DiJava, // Java (если добавишь)
  DiGo, // Go (если добавишь)
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

  FilePdf: FaFilePdf, // PDF (Font Awesome)
  FileWord: FaFileWord, // DOCX (Font Awesome)
  FileExcel: FaFileExcel, // XLSX (Font Awesome)
  FileAlt: FaFileAlt, // TXT (Font Awesome)
  FileRtf: MdTextFields, // RTF (Material Design)
  FileOdt: MdDescription, // ODT (Material Design)

  // ============================================
  // === ИЗОБРАЖЕНИЯ (svelte-icons) ===
  // ============================================

  FileJpg: MdImage, // JPG (Material Design)
  FilePng: FaFileImage, // PNG (Font Awesome)
  FileWebp: Icons.Image, // WEBP (Lucide)
  FileAvif: FaFileImage, // AVIF (Font Awesome)

  // ============================================
  // === АУДИО (svelte-icons) ===
  // ============================================

  FileMp3: MdAudiotrack, // MP3 (Material Design)
  FileWav: FaFileAudio, // WAV (Font Awesome)

  // ============================================
  // === ВИДЕО (svelte-icons) ===
  // ============================================

  FileMp4: MdVideocam, // MP4 (Material Design)
  FileMov: FaFileVideo, // MOV (Font Awesome)

  // ============================================
  // === ДОПОЛНИТЕЛЬНЫЕ (запасные) ===
  // ============================================
  FileArchive: FaFileArchive,
  FileCodeFa: FaFileCodeIcon,
  FileMusic: MdMusicNote,
  FileMovie: MdMovie,

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
      console.log(
        "📋 First 3 formats:",
        _formats
          .slice(0, 3)
          .map(f => f.name)
          .join(", "),
      );
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
