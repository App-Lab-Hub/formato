// src/lib/stores/convertStore.ts
import { writable, type Writable } from "svelte/store";
import type { Format } from "$lib/types/format";

// Экспортируем тип
export interface ConvertState {
  selectedTarget: Format | null;
  files: { path: string; name: string; id: string }[];
  convertedFiles: Map<string, string>;
  convertingFiles: Set<string>;
  counter: number;
}

// Хранилище для всех страниц конвертера
const convertStores = new Map<string, Writable<ConvertState>>();

// Создаем начальное состояние
function createInitialState(): ConvertState {
  return {
    selectedTarget: null,
    files: [],
    convertedFiles: new Map(),
    convertingFiles: new Set(),
    counter: 0,
  };
}

// Получаем стор для конкретного формата
export function getConvertStore(formatId: string): Writable<ConvertState> {
  if (!convertStores.has(formatId)) {
    const store = writable<ConvertState>(createInitialState());
    convertStores.set(formatId, store);
  }
  return convertStores.get(formatId)!;
}

// Очищаем стор при выходе
export function clearConvertStore(formatId: string) {
  if (convertStores.has(formatId)) {
    const store = convertStores.get(formatId)!;
    store.set(createInitialState());
    convertStores.delete(formatId);
  }
}
