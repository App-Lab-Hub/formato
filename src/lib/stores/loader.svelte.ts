// src/lib/stores/loader.svelte.ts

// Создаем локальные реактивные переменные
let isDeletingAll = $state(false);
let isResetting = $state(false);
let showLoaderOnList = $state(false);

// Экспортируем объект, который их читает и меняет (без использования this)
export const loader = {
  get isDeletingAll() {
    return isDeletingAll;
  },
  get isResetting() {
    return isResetting;
  },
  get showLoaderOnList() {
    return showLoaderOnList;
  },

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

  clearAll() {
    isDeletingAll = false;
    isResetting = false;
    showLoaderOnList = false;
  },
};
