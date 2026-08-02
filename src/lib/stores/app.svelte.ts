import { SvelteMap, SvelteSet } from "svelte/reactivity";

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

export interface ConversionProgress {
  fileId: string;
  fileName: string;
  stage: string;
  progress: number;
  message: string;
  estimatedTime?: number;
  isComplete: boolean;
  error?: string;
}

// ============================================================
// ЛОКАЛЬНЫЙ РЕАКТИВНЫЙ STATE (ЗАМЫКАНИЕ)
// ============================================================
let files = $state([] as FileItem[]);
const convertedFiles = new SvelteMap<string, ConvertedFile>();
const fileHashes = new SvelteMap<string, string>();
let counter = $state(0);

const isConverting = new SvelteSet<string>();
let isConvertingAll = $state(false);
let conversionProgress = $state({} as Record<string, ConversionProgress>);
let totalFiles = $state(0);
let completedFiles = $state(0);

let isDeletingAll = $state(false);
let isResetting = $state(false);
let showLoaderOnList = $state(false);

let sourceFormatId = $state("");
let selectedTargetId = $state("");

// ============================================================
// ЭКСПОРТ ЕДИНОГО ОБЪЕКТА STATE И ВСЕХ СТАРЫХ ФУНКЦИЙ
// ============================================================
export const appState = {
  // Чтение состояния через геттеры (для реактивности на клиенте)
  get files() {
    return files;
  },
  get convertedFiles() {
    return convertedFiles;
  },
  get fileHashes() {
    return fileHashes;
  },
  get counter() {
    return counter;
  },
  get isConverting() {
    return isConverting;
  },
  get isConvertingAll() {
    return isConvertingAll;
  },
  get conversionProgress() {
    return conversionProgress;
  },
  get totalFiles() {
    return totalFiles;
  },
  get completedFiles() {
    return completedFiles;
  },
  get isDeletingAll() {
    return isDeletingAll;
  },
  get isResetting() {
    return isResetting;
  },
  get showLoaderOnList() {
    return showLoaderOnList;
  },
  get sourceFormatId() {
    return sourceFormatId;
  },
  get selectedTargetId() {
    return selectedTargetId;
  },

  set sourceFormatId(value: string) {
    sourceFormatId = value;
  },
  set selectedTargetId(value: string) {
    selectedTargetId = value;
  },
};

// ============================================================
// 2. ФУНКЦИИ ДЛЯ ФАЙЛОВ
// ============================================================

export function addFile(file: FileItem) {
  files = [...files, file];
}

export function addFiles(newFiles: FileItem[]) {
  files = [...files, ...newFiles];
}

export function removeFile(fileId: string) {
  files = files.filter(f => f.id !== fileId);
  convertedFiles.delete(fileId);
  fileHashes.delete(fileId);
}

export function clearAllFiles() {
  files = [];
  convertedFiles.clear();
  fileHashes.clear();
  counter = 0;
}

export function setConvertedFile(fileId: string, data: ConvertedFile) {
  convertedFiles.set(fileId, data);
}

export function setFileHash(fileId: string, hash: string) {
  fileHashes.set(fileId, hash);
}

export function getNextId(): string {
  counter++;
  return `file-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
}

export function getFile(fileId: string): FileItem | undefined {
  return files.find(f => f.id === fileId);
}

export function getConvertedFile(fileId: string): ConvertedFile | undefined {
  return convertedFiles.get(fileId);
}

export function getFileHash(fileId: string): string | undefined {
  return fileHashes.get(fileId);
}

export function getTotalFiles(): number {
  return files.length;
}

export function getConvertedCount(): number {
  return files.filter(f => convertedFiles.has(f.id)).length;
}

export function getOverallProgress(): number {
  const total = getTotalFiles();
  if (total === 0) return 0;
  return getConvertedCount() / total;
}

// ============================================================
// 3. ФУНКЦИИ ДЛЯ КОНВЕРТАЦИИ
// ============================================================

export function startConversion(
  fileId: string,
  fileName: string,
  total: number = 0,
) {
  if (total > 0) {
    totalFiles = total;
    completedFiles = 0;
  }

  isConverting.add(fileId);
  conversionProgress[fileId] = {
    fileId,
    fileName,
    stage: "starting",
    progress: 0,
    message: "Начинаем конвертацию...",
    isComplete: false,
  };
}

export function updateProgress(
  fileId: string,
  data: {
    stage?: string;
    progress: number;
    message: string;
    estimatedTime?: number;
  },
) {
  const existing = conversionProgress[fileId];
  if (!existing) return;

  conversionProgress[fileId] = {
    ...existing,
    stage: data.stage || existing.stage,
    progress: data.progress,
    message: data.message,
    estimatedTime: data.estimatedTime,
  };
}

export function completeConversion(fileId: string, error?: string) {
  const existing = conversionProgress[fileId];
  if (!existing) return;

  conversionProgress[fileId] = {
    ...existing,
    isComplete: true,
    progress: 1,
    message: error ? `❌ Ошибка: ${error}` : "✅ Готово!",
    error,
  };

  isConverting.delete(fileId);

  if (!error) {
    completedFiles++;
  }

  if (isConverting.size === 0 && isConvertingAll) {
    isConvertingAll = false;
  }
}

export function startBatch(total: number) {
  totalFiles = total;
  completedFiles = 0;
  isConvertingAll = true;
}

export function clearConversionAll() {
  conversionProgress = {};
  isConverting.clear();
  isConvertingAll = false;
  totalFiles = 0;
  completedFiles = 0;
}

export function getFileProgress(fileId: string): ConversionProgress | null {
  return conversionProgress[fileId] || null;
}

export function getOverallMessage(): string {
  if (totalFiles === 0) return "";
  if (isConvertingAll) {
    return `Конвертация ${completedFiles}/${totalFiles} файлов...`;
  }
  if (isConverting.size > 0) {
    const fileId = Array.from(isConverting)[0];
    const file = conversionProgress[fileId];
    return file ? file.message : "Конвертация...";
  }
  return "";
}

// ============================================================
// 4. ФУНКЦИИ ДЛЯ UI
// ============================================================

export function startDeletingAll() {
  isDeletingAll = true;
  showLoaderOnList = true;
  isResetting = false;
}

export function startResetting() {
  isResetting = true;
  showLoaderOnList = true;
  isDeletingAll = false;
}

export function stopDeletingAll() {
  isDeletingAll = false;
  showLoaderOnList = false;
}

export function stopResetting() {
  isResetting = false;
  showLoaderOnList = false;
}

export function clearAllLoaders() {
  isDeletingAll = false;
  isResetting = false;
  showLoaderOnList = false;
}

// ============================================================
// 5. ФУНКЦИИ ДЛЯ ФОРМАТОВ
// ============================================================

export function setSourceFormatId(id: string) {
  sourceFormatId = id;
}

export function saveSelectedTargetId(id: string) {
  selectedTargetId = id;
}

export function clearSavedTargetId() {
  selectedTargetId = "";
}

// ============================================================
// 6. ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
// ============================================================

export function isAllConverted(): boolean {
  if (files.length === 0) return false;
  return files.every(f => convertedFiles.has(f.id));
}

export function getConvertedPaths(): string[] {
  const paths: string[] = [];
  for (const file of files) {
    const converted = convertedFiles.get(file.id);
    if (converted) {
      paths.push(converted.path);
    }
  }
  return paths;
}

export function resetAll() {
  clearAllFiles();
  clearConversionAll();
  clearAllLoaders();
  sourceFormatId = "";
  selectedTargetId = "";
}
