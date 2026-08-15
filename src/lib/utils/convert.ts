// src/lib/utils/convert.ts
import type { Format } from "$lib/types/format";

export function getTargetFormats(
  formats: Format[],
  sourceFormatId: string,
): Format[] {
  return formats.filter(f => f.id !== sourceFormatId);
}

export function getTargetFormatsWithAvailability(
  formats: Format[],
  sourceFormatId: string,
  availability: { available_formats?: string[] } | null,
): { format: Format; available: boolean }[] {
  const availableIds = availability?.available_formats || [];
  return formats
    .filter(f => f.id !== sourceFormatId)
    .map(f => ({
      format: f,
      available: availableIds.includes(f.id),
    }));
}

export function getInputMode(
  availability: { enable_text_mode?: boolean } | null,
  defaultMode: "file" | "text" = "file",
): "file" | "text" {
  return availability?.enable_text_mode ? "text" : defaultMode;
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  if (bytes < 1024 * 1024 * 1024)
    return (bytes / 1024 / 1024).toFixed(1) + " MB";
  return (bytes / 1024 / 1024 / 1024).toFixed(1) + " GB";
}

export function formatSize(mb: number): string {
  return mb + " MB";
}

export function getBaseName(fileName: string): string {
  let baseName = fileName.replace(/\.[^.]+$/, "");
  if (baseName.includes("@hash@")) {
    baseName = baseName.split("@hash@")[0];
  }
  return baseName;
}

export function getDefaultFileName(
  baseName: string,
  extension: string,
): string {
  return `formato_${baseName}.${extension}`;
}

export function getArchiveFileName(
  baseName: string,
  extension: string,
  _archiveFormat: string,
): string {
  return `formato_${baseName}.${extension}`;
}

export function getUniqueFileName(
  fileName: string,
  usedNames: Set<string>,
): string {
  if (!usedNames.has(fileName)) {
    return fileName;
  }

  // Проверяем, есть ли расширение
  const lastDotIndex = fileName.lastIndexOf(".");
  const hasExtension = lastDotIndex > 0 && lastDotIndex < fileName.length - 1;

  let nameWithoutExt: string;
  let ext: string;

  if (hasExtension) {
    nameWithoutExt = fileName.substring(0, lastDotIndex);
    ext = fileName.substring(lastDotIndex + 1);
  } else {
    nameWithoutExt = fileName;
    ext = "";
  }

  let counter = 1;
  let newName: string;

  do {
    if (ext) {
      newName = `${nameWithoutExt}${counter}.${ext}`;
    } else {
      newName = `${nameWithoutExt}${counter}`;
    }
    counter++;
  } while (usedNames.has(newName));

  return newName;
}

export function getArchiveName(format: string = "zip"): string {
  const timestamp = Date.now();
  const randomId = Math.random().toString(36).slice(2, 8);
  return `formato_${timestamp}_${randomId}.${format}`;
}

// ============================================================
// НОВЫЕ ФУНКЦИИ ДЛЯ ПРОВЕРКИ СУЩЕСТВОВАНИЯ ФАЙЛОВ
// ============================================================

/**
 * Проверяет, существует ли файл по указанному пути
 */
export async function fileExists(path: string): Promise<boolean> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("get_file_size", { path });
    return true;
  } catch {
    return false;
  }
}

/**
 * Разделяет файлы на существующие и отсутствующие
 */
export async function filterExistingFiles<
  T extends { path: string; id: string },
>(files: T[]): Promise<{ existing: T[]; missing: T[] }> {
  const existing: T[] = [];
  const missing: T[] = [];

  for (const file of files) {
    const exists = await fileExists(file.path);
    if (exists) {
      existing.push(file);
    } else {
      missing.push(file);
    }
  }

  return { existing, missing };
}

/**
 * Проверяет файлы в store и удаляет отсутствующие
 * Возвращает количество удаленных файлов
 */
export async function cleanupMissingFiles(
  formatId: string,
  getFiles: () => { path: string; id: string }[],
  removeFilesById: (formatId: string, ids: string[]) => void,
): Promise<number> {
  const files = getFiles();
  if (files.length === 0) return 0;

  const { missing } = await filterExistingFiles(files);

  if (missing.length > 0) {
    const missingIds = missing.map(f => f.id);
    removeFilesById(formatId, missingIds);
  }

  return missing.length;
}
