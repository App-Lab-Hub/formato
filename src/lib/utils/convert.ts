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
