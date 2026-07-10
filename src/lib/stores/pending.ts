// src/lib/stores/pending.ts
import { writable } from "svelte/store";
import { browser } from "$app/environment";

// Типы для pending операций
interface PendingState {
  removes: Set<string>; // ID файлов для удаления
  files: Map<string, string[]>; // sourceFormatId -> paths для добавления
}

// Создаем стор с начальным состоянием
function createPendingStore() {
  const { subscribe, set, update } = writable<PendingState>({
    removes: new Set(),
    files: new Map(),
  });

  // Загружаем из sessionStorage при инициализации
  if (browser) {
    const stored = sessionStorage.getItem("pending_operations");
    if (stored) {
      try {
        const parsed = JSON.parse(stored);
        set({
          removes: new Set(parsed.removes || []),
          files: new Map(parsed.files || []),
        });
      } catch (e) {
        console.warn("Failed to load pending operations:", e);
      }
    }
  }

  return {
    subscribe,

    // Добавить файл для удаления
    addRemove: (fileId: string) => {
      update(state => {
        state.removes.add(fileId);
        // Сохраняем в sessionStorage
        if (browser) {
          sessionStorage.setItem(
            "pending_operations",
            JSON.stringify({
              removes: Array.from(state.removes),
              files: Array.from(state.files.entries()),
            }),
          );
        }
        return state;
      });
    },

    // Удалить из pending на удаление
    removeFromRemove: (fileId: string) => {
      update(state => {
        state.removes.delete(fileId);
        if (browser) {
          sessionStorage.setItem(
            "pending_operations",
            JSON.stringify({
              removes: Array.from(state.removes),
              files: Array.from(state.files.entries()),
            }),
          );
        }
        return state;
      });
    },

    // Очистить все pending на удаление
    clearRemoves: () => {
      update(state => {
        state.removes.clear();
        if (browser) {
          sessionStorage.setItem(
            "pending_operations",
            JSON.stringify({
              removes: Array.from(state.removes),
              files: Array.from(state.files.entries()),
            }),
          );
        }
        return state;
      });
    },

    // Добавить файлы для добавления
    addFiles: (sourceFormatId: string, paths: string[]) => {
      update(state => {
        if (!state.files.has(sourceFormatId)) {
          state.files.set(sourceFormatId, []);
        }
        const existing = state.files.get(sourceFormatId)!;
        // Добавляем только новые пути
        for (const path of paths) {
          if (!existing.includes(path)) {
            existing.push(path);
          }
        }
        if (browser) {
          sessionStorage.setItem(
            "pending_operations",
            JSON.stringify({
              removes: Array.from(state.removes),
              files: Array.from(state.files.entries()),
            }),
          );
        }
        return state;
      });
    },

    // Удалить файлы из pending на добавление
    removeFiles: (sourceFormatId: string, paths: string[]) => {
      update(state => {
        if (state.files.has(sourceFormatId)) {
          const existing = state.files.get(sourceFormatId)!;
          const remaining = existing.filter(p => !paths.includes(p));
          if (remaining.length === 0) {
            state.files.delete(sourceFormatId);
          } else {
            state.files.set(sourceFormatId, remaining);
          }
        }
        if (browser) {
          sessionStorage.setItem(
            "pending_operations",
            JSON.stringify({
              removes: Array.from(state.removes),
              files: Array.from(state.files.entries()),
            }),
          );
        }
        return state;
      });
    },

    // Получить pending файлы для конкретного формата
    getPendingFiles: (sourceFormatId: string): string[] => {
      let result: string[] = [];
      update(state => {
        result = state.files.get(sourceFormatId) || [];
        return state;
      });
      return result;
    },

    // Очистить все pending
    clearAll: () => {
      set({ removes: new Set(), files: new Map() });
      if (browser) {
        sessionStorage.removeItem("pending_operations");
      }
    },
  };
}

export const pending = createPendingStore();
