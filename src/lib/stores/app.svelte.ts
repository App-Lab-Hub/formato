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

// ============================================================
// ЛОКАЛЬНЫЙ РЕАКТИВНЫЙ STATE
// ============================================================
// Хранилище: { formatId: [FileItem, ...] }
const filesMap = new SvelteMap<string, FileItem[]>();
const counterMap = new SvelteMap<string, number>();

// Текущий формат (для отображения)
let currentFormatId = $state("");

// ============================================================
// ЭКСПОРТ
// ============================================================
export const appState = {
  // Получить файлы для конкретного формата
  getFilesForFormat(formatId: string): FileItem[] {
    if (!filesMap.has(formatId)) {
      return [];
    }
    return filesMap.get(formatId)!;
  },

  // Получить все файлы (для текущего формата)
  get files() {
    return this.getFilesForFormat(currentFormatId);
  },

  get currentFormatId() {
    return currentFormatId;
  },

  set currentFormatId(value: string) {
    currentFormatId = value;
  },

  // Добавить файл в конкретный формат
  addFileToFormat(formatId: string, file: FileItem) {
    const current = this.getFilesForFormat(formatId);
    filesMap.set(formatId, [...current, file]);
  },

  // Добавить файлы в конкретный формат
  addFilesToFormat(formatId: string, newFiles: FileItem[]) {
    const current = this.getFilesForFormat(formatId);
    filesMap.set(formatId, [...current, ...newFiles]);
  },

  // Удалить файл из конкретного формата
  removeFileFromFormat(formatId: string, fileId: string) {
    const current = this.getFilesForFormat(formatId);
    const filtered = current.filter(f => f.id !== fileId);
    filesMap.set(formatId, filtered);
  },

  // Очистить файлы в конкретном формате
  clearFilesForFormat(formatId: string) {
    filesMap.set(formatId, []);
    counterMap.set(formatId, 0);
  },

  // Получить следующий ID для формата
  getNextIdForFormat(formatId: string): string {
    const current = counterMap.get(formatId) || 0;
    counterMap.set(formatId, current + 1);
    return `file-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
  },

  // Получить количество файлов в формате
  getTotalFilesForFormat(formatId: string): number {
    return this.getFilesForFormat(formatId).length;
  },

  // Очистить всё
  resetAll() {
    filesMap.clear();
    counterMap.clear();
    currentFormatId = "";
  },
};
