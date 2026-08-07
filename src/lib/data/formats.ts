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
  // === ТЕКСТОВЫЕ И КОНФИГУРАЦИОННЫЕ ===
  // ============================================
  FileBraces: Icons.Braces, // JSON
  FileText: Icons.FileText, // YAML
  FileSpreadsheet: Icons.Table, // CSV
  FileCode: Icons.CodeXml, // XML
  AlignLeft: Icons.AlignLeft, // TOML
  ListOrdered: Icons.ListOrdered, // INI
  Braces: Icons.Brackets, // Markdown
  Globe: Icons.Globe, // HTML
  FileAlt: FaFileAlt, // TXT
  FileRtf: Icons.Type, // RTF

  // ============================================
  // === ДОКУМЕНТЫ ===
  // ============================================
  FilePdf: FaFilePdf, // PDF
  FileWord: FaFileWord, // DOCX
  FileExcel: FaFileExcel, // XLSX
  FileOdt: MdDescription, // ODT

  // ============================================
  // === ИЗОБРАЖЕНИЯ ===
  // ============================================
  FileJpg: Icons.Image, // JPG
  FilePng: Icons.ImageDown, // PNG
  FileWebp: Icons.ImageUp, // WEBP
  FileAvif: Icons.ImageOff, // AVIF
  FileGif: Icons.ImagePlay, // GIF
  FileBmp: Icons.ImageMinus, // BMP
  FileTiff: Icons.ImagePlus, // TIFF
  FileIco: Icons.Square, // ICO
  FileQoi: Icons.Circle, // QOI
  FileTga: Icons.Diamond, // TGA
  FileExr: Icons.Hexagon, // EXR
  FileHdr: Icons.Octagon, // HDR
  FileDds: Icons.Pentagon, // DDS
  FilePnm: Icons.Triangle, // PNM
  FileFarbfeld: Icons.Circle, // Farbfeld
  FileImage: MdImage, // Общая иконка для изображений

  // ============================================
  // === АУДИО ===
  // ============================================
  FileMp3: Icons.Music, // MP3
  FileWav: Icons.Waves, // WAV
  FileAac: Icons.Music2, // AAC
  FileFlac: Icons.Music3, // FLAC
  FileOgg: Icons.Music4, // OGG
  FileOpus: Icons.Speaker, // OPUS
  FileWma: Icons.Volume2, // WMA
  FileM4a: Icons.Headphones, // M4A
  FileAiff: Icons.Mic, // AIFF
  FileAc3: Icons.Volume, // AC3
  FileVoc: Icons.MicVocal, // Creative Voice
  FileWv: Icons.Volume1, // WavPack
  FileAdx: Icons.Mic, // ADX
  FileDts: Icons.VolumeX, // DTS
  FileEac3: Icons.Volume, // E-AC-3
  FileTta: Icons.Mic, // True Audio
  FileAptx: Icons.Bluetooth, // aptX
  FileSbc: Icons.BluetoothConnected, // SBC
  FileMlp: Icons.Album, // MLP
  FileCaf: Icons.Headphones, // CAF
  FileW64: Icons.Volume, // W64
  FileAudio: MdAudiotrack, // Общая иконка для аудио

  // ============================================
  // === ВИДЕО ===
  // ============================================
  FileMp4: Icons.Video, // MP4
  FileMov: Icons.Film, // MOV
  FileAvi: Icons.Clapperboard, // AVI
  FileMkv: Icons.Play, // MKV
  FileWebm: Icons.Monitor, // WEBM
  FileWmv: Icons.Tv, // WMV
  FileFlv: Icons.Cast, // FLV
  File3gp: Icons.Smartphone, // 3GP
  FileM4v: Icons.Tablet, // M4V
  FileTs: Icons.Monitor, // MPEG-TS
  FileVob: Icons.Disc, // VOB
  FileMpg: Icons.Disc2, // MPEG
  FileHevc: Icons.Disc3, // HEVC
  FileMjpeg: Icons.Camera, // MJPEG
  FileNut: Icons.CameraOff, // NUT
  FileVideo: MdVideocam, // Общая иконка для видео

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
        formatType: f.format_type,
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
