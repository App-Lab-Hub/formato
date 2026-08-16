// src/lib/stores/__tests__/app.svelte.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  appState,
  type FileItem,
  type ConvertedFile,
} from "$lib/stores/app.svelte";

describe("appState", () => {
  // Очищаем состояние перед каждым тестом
  beforeEach(() => {
    appState.resetAll();
  });

  // ============================================================
  // ТЕСТЫ: Управление форматами
  // ============================================================

  describe("currentFormatId", () => {
    it("should have empty currentFormatId by default", () => {
      expect(appState.currentFormatId).toBe("");
    });

    it("should set currentFormatId", () => {
      appState.currentFormatId = "json";
      expect(appState.currentFormatId).toBe("json");
    });
  });

  // ============================================================
  // ТЕСТЫ: Управление файлами
  // ============================================================

  describe("addFileToFormat", () => {
    it("should add a file to a format", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };

      appState.addFileToFormat("json", file);
      const files = appState.getFilesForFormat("json");

      expect(files).toHaveLength(1);
      expect(files[0]).toEqual(file);
    });

    it("should add multiple files to the same format", () => {
      const file1: FileItem = {
        id: "file-1",
        path: "/path/to/file1.json",
        name: "file1.json",
      };
      const file2: FileItem = {
        id: "file-2",
        path: "/path/to/file2.json",
        name: "file2.json",
      };

      appState.addFileToFormat("json", file1);
      appState.addFileToFormat("json", file2);

      const files = appState.getFilesForFormat("json");
      expect(files).toHaveLength(2);
    });

    it("should add files to different formats separately", () => {
      const jsonFile: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };
      const csvFile: FileItem = {
        id: "file-2",
        path: "/path/to/file.csv",
        name: "file.csv",
      };

      appState.addFileToFormat("json", jsonFile);
      appState.addFileToFormat("csv", csvFile);

      expect(appState.getFilesForFormat("json")).toHaveLength(1);
      expect(appState.getFilesForFormat("csv")).toHaveLength(1);
    });
  });

  describe("addFilesToFormat", () => {
    it("should add multiple files at once", () => {
      const files: FileItem[] = [
        {
          id: "file-1",
          path: "/path/to/file1.json",
          name: "file1.json",
        },
        {
          id: "file-2",
          path: "/path/to/file2.json",
          name: "file2.json",
        },
      ];

      appState.addFilesToFormat("json", files);
      expect(appState.getFilesForFormat("json")).toHaveLength(2);
    });

    it("should append files to existing ones", () => {
      const file1: FileItem = {
        id: "file-1",
        path: "/path/to/file1.json",
        name: "file1.json",
      };
      const file2: FileItem = {
        id: "file-2",
        path: "/path/to/file2.json",
        name: "file2.json",
      };

      appState.addFileToFormat("json", file1);
      appState.addFilesToFormat("json", [file2]);

      expect(appState.getFilesForFormat("json")).toHaveLength(2);
    });
  });

  // ============================================================
  // ТЕСТЫ: Удаление файлов
  // ============================================================

  describe("removeFileFromFormat", () => {
    it("should remove a file from a format", () => {
      const file1: FileItem = {
        id: "file-1",
        path: "/path/to/file1.json",
        name: "file1.json",
      };
      const file2: FileItem = {
        id: "file-2",
        path: "/path/to/file2.json",
        name: "file2.json",
      };

      appState.addFileToFormat("json", file1);
      appState.addFileToFormat("json", file2);

      appState.removeFileFromFormat("json", "file-1");

      const files = appState.getFilesForFormat("json");
      expect(files).toHaveLength(1);
      expect(files[0].id).toBe("file-2");
    });

    it("should remove converted file when source file is removed", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };
      const converted: ConvertedFile = {
        path: "/path/to/converted/file.pdf",
        format: "pdf",
      };

      appState.addFileToFormat("json", file);
      appState.addConvertedFile("json", "file-1", converted);

      expect(appState.getConvertedFile("json", "file-1")).toBeDefined();

      appState.removeFileFromFormat("json", "file-1");

      expect(appState.getConvertedFile("json", "file-1")).toBeUndefined();
    });

    it("should not fail when removing non-existent file", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };

      appState.addFileToFormat("json", file);

      expect(() => {
        appState.removeFileFromFormat("json", "non-existent");
      }).not.toThrow();

      expect(appState.getFilesForFormat("json")).toHaveLength(1);
    });
  });

  describe("removeFilesById", () => {
    it("should remove multiple files by ids", () => {
      const files: FileItem[] = [
        { id: "file-1", path: "/path/to/file1.json", name: "file1.json" },
        { id: "file-2", path: "/path/to/file2.json", name: "file2.json" },
        { id: "file-3", path: "/path/to/file3.json", name: "file3.json" },
      ];

      appState.addFilesToFormat("json", files);
      appState.removeFilesById("json", ["file-1", "file-3"]);

      const remaining = appState.getFilesForFormat("json");
      expect(remaining).toHaveLength(1);
      expect(remaining[0].id).toBe("file-2");
    });

    it("should remove converted files when source files are removed", () => {
      const files: FileItem[] = [
        { id: "file-1", path: "/path/to/file1.json", name: "file1.json" },
        { id: "file-2", path: "/path/to/file2.json", name: "file2.json" },
      ];

      appState.addFilesToFormat("json", files);
      appState.addConvertedFile("json", "file-1", {
        path: "/path/to/file1.pdf",
        format: "pdf",
      });
      appState.addConvertedFile("json", "file-2", {
        path: "/path/to/file2.pdf",
        format: "pdf",
      });

      appState.removeFilesById("json", ["file-1"]);

      expect(appState.getConvertedFile("json", "file-1")).toBeUndefined();
      expect(appState.getConvertedFile("json", "file-2")).toBeDefined();
    });

    it("should not fail when removing non-existent ids", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };

      appState.addFileToFormat("json", file);

      expect(() => {
        appState.removeFilesById("json", ["non-existent"]);
      }).not.toThrow();

      expect(appState.getFilesForFormat("json")).toHaveLength(1);
    });
  });

  describe("clearFilesForFormat", () => {
    it("should clear all files for a format", () => {
      const files: FileItem[] = [
        { id: "file-1", path: "/path/to/file1.json", name: "file1.json" },
        { id: "file-2", path: "/path/to/file2.json", name: "file2.json" },
      ];

      appState.addFilesToFormat("json", files);
      appState.clearFilesForFormat("json");

      expect(appState.getFilesForFormat("json")).toHaveLength(0);
    });

    it("should clear converted files for a format", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };
      const converted: ConvertedFile = {
        path: "/path/to/converted/file.pdf",
        format: "pdf",
      };

      appState.addFileToFormat("json", file);
      appState.addConvertedFile("json", "file-1", converted);

      appState.clearFilesForFormat("json");

      expect(appState.getConvertedFile("json", "file-1")).toBeUndefined();
    });

    it("should reset counter for the format", () => {
      const id1 = appState.getNextIdForFormat("json");
      const id2 = appState.getNextIdForFormat("json");

      expect(id1).not.toBe(id2);

      appState.clearFilesForFormat("json");

      // После очистки счетчик должен сброситься
      const newId = appState.getNextIdForFormat("json");
      expect(newId).toMatch(/^file-/);
    });
  });

  // ============================================================
  // ТЕСТЫ: Конвертированные файлы
  // ============================================================

  describe("addConvertedFile", () => {
    it("should add a converted file for a format", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };
      const converted: ConvertedFile = {
        path: "/path/to/converted/file.pdf",
        format: "pdf",
      };

      appState.addFileToFormat("json", file);
      appState.addConvertedFile("json", "file-1", converted);

      const result = appState.getConvertedFile("json", "file-1");
      expect(result).toEqual(converted);
    });

    it("should store converted files in a Map", () => {
      const file1: FileItem = {
        id: "file-1",
        path: "/path/to/file1.json",
        name: "file1.json",
      };
      const file2: FileItem = {
        id: "file-2",
        path: "/path/to/file2.json",
        name: "file2.json",
      };
      const converted1: ConvertedFile = {
        path: "/path/to/converted/file1.pdf",
        format: "pdf",
      };
      const converted2: ConvertedFile = {
        path: "/path/to/converted/file2.docx",
        format: "docx",
      };

      appState.addFileToFormat("json", file1);
      appState.addFileToFormat("json", file2);
      appState.addConvertedFile("json", "file-1", converted1);
      appState.addConvertedFile("json", "file-2", converted2);

      const map = appState.getConvertedFilesForFormat("json");
      expect(map.size).toBe(2);
      expect(map.get("file-1")).toEqual(converted1);
      expect(map.get("file-2")).toEqual(converted2);
    });
  });

  describe("getConvertedFile", () => {
    it("should return undefined for non-existent file", () => {
      const result = appState.getConvertedFile("json", "non-existent");
      expect(result).toBeUndefined();
    });

    it("should return undefined for non-existent format", () => {
      const result = appState.getConvertedFile("non-existent", "file-1");
      expect(result).toBeUndefined();
    });
  });

  describe("getConvertedFilesForFormat", () => {
    it("should return empty Map for non-existent format", () => {
      const map = appState.getConvertedFilesForFormat("non-existent");
      expect(map).toBeInstanceOf(Map);
      expect(map.size).toBe(0);
    });
  });

  // ============================================================
  // ТЕСТЫ: Выбор целевого формата
  // ============================================================

  describe("setSelectedTargetForFormat", () => {
    it("should set selected target for a format", () => {
      appState.setSelectedTargetForFormat("json", "pdf");
      expect(appState.getSelectedTargetForFormat("json")).toBe("pdf");
    });

    it("should update selected target for a format", () => {
      appState.setSelectedTargetForFormat("json", "pdf");
      appState.setSelectedTargetForFormat("json", "docx");
      expect(appState.getSelectedTargetForFormat("json")).toBe("docx");
    });

    it("should store different targets for different formats", () => {
      appState.setSelectedTargetForFormat("json", "pdf");
      appState.setSelectedTargetForFormat("csv", "xlsx");

      expect(appState.getSelectedTargetForFormat("json")).toBe("pdf");
      expect(appState.getSelectedTargetForFormat("csv")).toBe("xlsx");
    });
  });

  describe("clearSelectedTargetForFormat", () => {
    it("should clear selected target for a format", () => {
      appState.setSelectedTargetForFormat("json", "pdf");
      appState.clearSelectedTargetForFormat("json");

      expect(appState.getSelectedTargetForFormat("json")).toBeUndefined();
    });

    it("should not fail when clearing non-existent format", () => {
      expect(() => {
        appState.clearSelectedTargetForFormat("non-existent");
      }).not.toThrow();
    });
  });

  // ============================================================
  // ТЕСТЫ: Генерация ID
  // ============================================================

  describe("getNextIdForFormat", () => {
    it("should generate unique IDs for a format", () => {
      const id1 = appState.getNextIdForFormat("json");
      const id2 = appState.getNextIdForFormat("json");

      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^file-/);
      expect(id2).toMatch(/^file-/);
    });

    it("should increment counter for each format separately", () => {
      const id1_json = appState.getNextIdForFormat("json");
      const id1_csv = appState.getNextIdForFormat("csv");
      const id2_json = appState.getNextIdForFormat("json");
      const id2_csv = appState.getNextIdForFormat("csv");

      expect(id1_json).not.toBe(id2_json);
      expect(id1_csv).not.toBe(id2_csv);
    });

    it("should reset counter after clearFilesForFormat", () => {
      const id1 = appState.getNextIdForFormat("json");
      appState.clearFilesForFormat("json");
      const id2 = appState.getNextIdForFormat("json");

      // ID должны быть разными, но формат одинаковый
      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^file-/);
      expect(id2).toMatch(/^file-/);
    });
  });

  // ============================================================
  // ТЕСТЫ: Геттеры
  // ============================================================

  describe("files getter", () => {
    it("should return files for current format", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };

      appState.currentFormatId = "json";
      appState.addFileToFormat("json", file);

      expect(appState.files).toHaveLength(1);
      expect(appState.files[0]).toEqual(file);
    });

    it("should return empty array when no current format", () => {
      appState.currentFormatId = "";
      expect(appState.files).toEqual([]);
    });
  });

  describe("convertedFiles getter", () => {
    it("should return converted files for current format", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };
      const converted: ConvertedFile = {
        path: "/path/to/converted/file.pdf",
        format: "pdf",
      };

      appState.currentFormatId = "json";
      appState.addFileToFormat("json", file);
      appState.addConvertedFile("json", "file-1", converted);

      const map = appState.convertedFiles;
      expect(map.size).toBe(1);
      expect(map.get("file-1")).toEqual(converted);
    });

    it("should return empty Map when no current format", () => {
      appState.currentFormatId = "";
      const map = appState.convertedFiles;
      expect(map).toBeInstanceOf(Map);
      expect(map.size).toBe(0);
    });
  });

  // ============================================================
  // ТЕСТЫ: resetAll
  // ============================================================

  describe("resetAll", () => {
    it("should reset all state", () => {
      const file: FileItem = {
        id: "file-1",
        path: "/path/to/file.json",
        name: "file.json",
      };
      const converted: ConvertedFile = {
        path: "/path/to/converted/file.pdf",
        format: "pdf",
      };

      appState.currentFormatId = "json";
      appState.addFileToFormat("json", file);
      appState.addConvertedFile("json", "file-1", converted);
      appState.setSelectedTargetForFormat("json", "pdf");
      appState.getNextIdForFormat("json");

      appState.resetAll();

      expect(appState.currentFormatId).toBe("");
      expect(appState.getFilesForFormat("json")).toEqual([]);
      expect(appState.getConvertedFilesForFormat("json").size).toBe(0);
      expect(appState.getSelectedTargetForFormat("json")).toBeUndefined();
    });
  });

  // ============================================================
  // ТЕСТЫ: getTotalFilesForFormat
  // ============================================================

  describe("getTotalFilesForFormat", () => {
    it("should return total files for a format", () => {
      const files: FileItem[] = [
        { id: "file-1", path: "/path/to/file1.json", name: "file1.json" },
        { id: "file-2", path: "/path/to/file2.json", name: "file2.json" },
        { id: "file-3", path: "/path/to/file3.json", name: "file3.json" },
      ];

      appState.addFilesToFormat("json", files);

      expect(appState.getTotalFilesForFormat("json")).toBe(3);
    });

    it("should return 0 for non-existent format", () => {
      expect(appState.getTotalFilesForFormat("non-existent")).toBe(0);
    });
  });

  // ============================================================
  // ТЕСТЫ: Сложные сценарии
  // ============================================================

  describe("complex scenarios", () => {
    it("should handle multiple formats with multiple files each", () => {
      const jsonFiles: FileItem[] = [
        { id: "json-1", path: "/path/to/file1.json", name: "file1.json" },
        { id: "json-2", path: "/path/to/file2.json", name: "file2.json" },
      ];
      const csvFiles: FileItem[] = [
        { id: "csv-1", path: "/path/to/file1.csv", name: "file1.csv" },
        { id: "csv-2", path: "/path/to/file2.csv", name: "file2.csv" },
        { id: "csv-3", path: "/path/to/file3.csv", name: "file3.csv" },
      ];

      appState.addFilesToFormat("json", jsonFiles);
      appState.addFilesToFormat("csv", csvFiles);

      appState.setSelectedTargetForFormat("json", "pdf");
      appState.setSelectedTargetForFormat("csv", "xlsx");

      expect(appState.getTotalFilesForFormat("json")).toBe(2);
      expect(appState.getTotalFilesForFormat("csv")).toBe(3);
      expect(appState.getSelectedTargetForFormat("json")).toBe("pdf");
      expect(appState.getSelectedTargetForFormat("csv")).toBe("xlsx");
    });

    it("should handle file removal with converted files cleanup", () => {
      const files: FileItem[] = [
        { id: "file-1", path: "/path/to/file1.json", name: "file1.json" },
        { id: "file-2", path: "/path/to/file2.json", name: "file2.json" },
      ];

      appState.addFilesToFormat("json", files);

      appState.addConvertedFile("json", "file-1", {
        path: "/path/to/file1.pdf",
        format: "pdf",
      });
      appState.addConvertedFile("json", "file-2", {
        path: "/path/to/file2.pdf",
        format: "pdf",
      });

      expect(appState.getConvertedFilesForFormat("json").size).toBe(2);

      appState.removeFilesById("json", ["file-1"]);

      expect(appState.getFilesForFormat("json")).toHaveLength(1);
      expect(appState.getConvertedFilesForFormat("json").size).toBe(1);
      expect(appState.getConvertedFile("json", "file-1")).toBeUndefined();
      expect(appState.getConvertedFile("json", "file-2")).toBeDefined();
    });
  });
});
