// src/lib/stores/scrollStore.ts
import { writable } from "svelte/store";

interface ScrollState {
  position: number;
  hasRestored: boolean;
  isUserInteracting: boolean;
}

// Создаем стор для главной страницы
export const mainPageScroll = writable<ScrollState>({
  position: 0,
  hasRestored: false,
  isUserInteracting: false,
});

// Создаем стор для страниц конвертера
export const convertPageScrolls = writable<Map<string, ScrollState>>(new Map());
