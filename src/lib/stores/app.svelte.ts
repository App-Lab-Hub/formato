// src/lib/stores/app.svelte.ts
import { SvelteMap } from "svelte/reactivity";

// ============================================================
// ТИПЫ И ИНТЕРФЕЙСЫ
// ============================================================
export interface FileItem {
  path: string;
  name: string;
  id: string;
}

export interface ConvertedFile {
  path: string;
  format: string;
}

// ============================================================
// ЛОКАЛЬНЫЙ РЕАКТИВНЫЙ STATE
// ============================================================
const filesMap = new SvelteMap<string, FileItem[]>();
const counterMap = new SvelteMap<string, number>();
const convertedFilesMap = new SvelteMap<string, Map<string, ConvertedFile>>();
const selectedTargetMap = new SvelteMap<string, string>();

let currentFormatId = $state("");

// ============================================================
// ЭКСПОРТ
// ============================================================
export const appState = {
  getFilesForFormat(formatId: string): FileItem[] {
    if (!filesMap.has(formatId)) {
      return [];
    }
    return filesMap.get(formatId)!;
  },

  getConvertedFilesForFormat(formatId: string): Map<string, ConvertedFile> {
    if (!convertedFilesMap.has(formatId)) {
      return new Map();
    }
    return convertedFilesMap.get(formatId)!;
  },

  getConvertedFile(
    formatId: string,
    fileId: string,
  ): ConvertedFile | undefined {
    const converted = convertedFilesMap.get(formatId);
    if (!converted) return undefined;
    return converted.get(fileId);
  },

  getSelectedTargetForFormat(formatId: string): string | undefined {
    return selectedTargetMap.get(formatId);
  },

  setSelectedTargetForFormat(sourceFormatId: string, targetFormatId: string) {
    selectedTargetMap.set(sourceFormatId, targetFormatId);
  },

  clearSelectedTargetForFormat(formatId: string) {
    selectedTargetMap.delete(formatId);
  },

  get files() {
    return this.getFilesForFormat(currentFormatId);
  },

  get convertedFiles() {
    return this.getConvertedFilesForFormat(currentFormatId);
  },

  get currentFormatId() {
    return currentFormatId;
  },

  set currentFormatId(value: string) {
    currentFormatId = value;
  },

  addFileToFormat(formatId: string, file: FileItem) {
    const current = this.getFilesForFormat(formatId);
    filesMap.set(formatId, [...current, file]);
  },

  addFilesToFormat(formatId: string, newFiles: FileItem[]) {
    const current = this.getFilesForFormat(formatId);
    filesMap.set(formatId, [...current, ...newFiles]);
  },

  addConvertedFile(formatId: string, fileId: string, data: ConvertedFile) {
    const current = convertedFilesMap.get(formatId);
    const newMap = new Map(current || []);
    newMap.set(fileId, data);
    convertedFilesMap.set(formatId, newMap);
  },

  removeFileFromFormat(formatId: string, fileId: string) {
    const current = this.getFilesForFormat(formatId);
    const filtered = current.filter(f => f.id !== fileId);
    filesMap.set(formatId, filtered);

    const converted = convertedFilesMap.get(formatId);
    if (converted) {
      const newMap = new Map(converted);
      newMap.delete(fileId);
      convertedFilesMap.set(formatId, newMap);
    }
  },

  // [OK] НОВЫЙ МЕТОД: удаляет несколько файлов по id
  removeFilesById(formatId: string, ids: string[]) {
    const current = this.getFilesForFormat(formatId);
    const idSet = new Set(ids);
    const filtered = current.filter(f => !idSet.has(f.id));
    filesMap.set(formatId, filtered);

    const converted = convertedFilesMap.get(formatId);
    if (converted) {
      const newMap = new Map(converted);
      for (const id of ids) {
        newMap.delete(id);
      }
      convertedFilesMap.set(formatId, newMap);
    }
  },

  clearFilesForFormat(formatId: string) {
    filesMap.set(formatId, []);
    counterMap.set(formatId, 0);
    convertedFilesMap.delete(formatId);
  },

  getNextIdForFormat(formatId: string): string {
    const current = counterMap.get(formatId) || 0;
    counterMap.set(formatId, current + 1);
    return `file-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
  },

  getTotalFilesForFormat(formatId: string): number {
    return this.getFilesForFormat(formatId).length;
  },

  resetAll() {
    filesMap.clear();
    counterMap.clear();
    convertedFilesMap.clear();
    selectedTargetMap.clear();
    currentFormatId = "";
  },
};
