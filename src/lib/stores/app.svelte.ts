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
// ЛОКАЛЬНЫЙ РЕАКТИВНЫЙ STATE (ЗАМЫКАНИЕ)
// ============================================================
// Хранилище файлов по группам (ключ = sourceFormatId)
// Инициализируем с пустым массивом для всех групп
const filesMap = new SvelteMap<string, FileItem[]>();
const counterMap = new SvelteMap<string, number>();

let sourceFormatId = $state("");

// ============================================================
// ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
// ============================================================

function getGroupFiles(groupId: string): FileItem[] {
  // Если группы нет, возвращаем пустой массив, НЕ мутируем SvelteMap
  if (!filesMap.has(groupId)) {
    return [];
  }
  return filesMap.get(groupId)!;
}

function getGroupCounter(groupId: string): number {
  if (!counterMap.has(groupId)) {
    return 0;
  }
  return counterMap.get(groupId)!;
}

// ============================================================
// ЭКСПОРТ ЕДИНОГО ОБЪЕКТА STATE
// ============================================================
export const appState = {
  get files() {
    return getGroupFiles(sourceFormatId);
  },
  get counter() {
    return getGroupCounter(sourceFormatId);
  },
  get sourceFormatId() {
    return sourceFormatId;
  },

  set sourceFormatId(value: string) {
    sourceFormatId = value;
  },
};

// ============================================================
// ФУНКЦИИ ДЛЯ ФАЙЛОВ
// ============================================================

/** Добавить один файл */
export function addFile(file: FileItem) {
  const current = getGroupFiles(sourceFormatId);
  filesMap.set(sourceFormatId, [...current, file]);
}

/** Добавить несколько файлов */
export function addFiles(newFiles: FileItem[]) {
  const current = getGroupFiles(sourceFormatId);
  filesMap.set(sourceFormatId, [...current, ...newFiles]);
}

/** Удалить файл по ID */
export function removeFile(fileId: string) {
  const current = getGroupFiles(sourceFormatId);
  const filtered = current.filter(f => f.id !== fileId);
  filesMap.set(sourceFormatId, filtered);
}

/** Очистить все файлы в текущей группе */
export function clearAllFiles() {
  filesMap.set(sourceFormatId, []);
  counterMap.set(sourceFormatId, 0);
}

/** Получить следующий ID для файла */
export function getNextId(): string {
  const current = getGroupCounter(sourceFormatId);
  counterMap.set(sourceFormatId, current + 1);
  return `file-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
}

/** Получить файл по ID */
export function getFile(fileId: string): FileItem | undefined {
  const current = getGroupFiles(sourceFormatId);
  return current.find(f => f.id === fileId);
}

/** Получить общее количество файлов в текущей группе */
export function getTotalFiles(): number {
  return getGroupFiles(sourceFormatId).length;
}

/** Получить список всех файлов в текущей группе */
export function getAllFiles(): FileItem[] {
  return getGroupFiles(sourceFormatId);
}

/** Очистить всё (все группы) */
export function resetAll() {
  filesMap.clear();
  counterMap.clear();
  sourceFormatId = "";
}
