// src/lib/stores/loader.svelte.ts

// Создаем локальные реактивные переменные
let isDeletingAll = $state(false);
let isResetting = $state(false);
let showLoaderOnList = $state(false);
let downloadingSynthesis = $state(false);
let downloadingRecognition = $state(false);

// Новые переменные для конвертации файлов
let convertingFileIds = $state<Set<string>>(new Set());

// Экспортируем объект, который их читает и меняет (без использования this)
export const loader = {
  // Существующие геттеры
  get isDeletingAll() {
    return isDeletingAll;
  },
  get isResetting() {
    return isResetting;
  },
  get showLoaderOnList() {
    return showLoaderOnList;
  },
  get downloadingSynthesis() {
    return downloadingSynthesis;
  },
  get downloadingRecognition() {
    return downloadingRecognition;
  },

  // Новый геттер для конвертации
  get convertingFiles() {
    return convertingFileIds;
  },

  // Существующие методы
  startDeletingAll() {
    isDeletingAll = true;
    showLoaderOnList = true;
    isResetting = false;
  },

  startResetting() {
    isResetting = true;
    showLoaderOnList = true;
    isDeletingAll = false;
  },

  stopDeletingAll() {
    isDeletingAll = false;
    showLoaderOnList = false;
  },

  stopResetting() {
    isResetting = false;
    showLoaderOnList = false;
  },

  startDownloadingSynthesis() {
    downloadingSynthesis = true;
  },

  stopDownloadingSynthesis() {
    downloadingSynthesis = false;
  },

  startDownloadingRecognition() {
    downloadingRecognition = true;
  },

  stopDownloadingRecognition() {
    downloadingRecognition = false;
  },

  // Новые методы для управления конвертацией
  startConverting(fileId: string) {
    convertingFileIds = new Set(convertingFileIds).add(fileId);
  },

  stopConverting(fileId: string) {
    const newSet = new Set(convertingFileIds);
    newSet.delete(fileId);
    convertingFileIds = newSet;
  },

  isConverting(fileId: string): boolean {
    return convertingFileIds.has(fileId);
  },

  clearConverting() {
    convertingFileIds = new Set();
  },

  clearAll() {
    isDeletingAll = false;
    isResetting = false;
    showLoaderOnList = false;
    downloadingSynthesis = false;
    downloadingRecognition = false;
    convertingFileIds = new Set();
  },
};
