// src/routes/__tests__/convert.spec.ts
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  getTargetFormats,
  getInputMode,
  formatFileSize,
  formatSize,
  getBaseName,
  getDefaultFileName,
  getArchiveFileName,
  getUniqueFileName,
  getArchiveName,
  fileExists,
  filterExistingFiles,
} from "$lib/utils/convert";

// Мокаем @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

describe("convert utils", () => {
  const mockFormats = [
    { id: "json", name: "JSON", format_id: "json" } as any,
    { id: "yaml", name: "YAML", format_id: "yaml" } as any,
    { id: "pdf", name: "PDF", format_id: "pdf" } as any,
  ];

  // ============================================================
  // getTargetFormats
  // ============================================================

  describe("getTargetFormats", () => {
    it("should exclude source format", () => {
      const result = getTargetFormats(mockFormats, "json");
      expect(result).toHaveLength(2);
      expect(result[0].id).toBe("yaml");
      expect(result[1].id).toBe("pdf");
    });

    it("should return all other formats", () => {
      const result = getTargetFormats(mockFormats, "pdf");
      expect(result).toHaveLength(2);
      expect(result.map(f => f.id)).toEqual(["json", "yaml"]);
    });

    it("should return empty array when formats is empty", () => {
      const result = getTargetFormats([], "json");
      expect(result).toHaveLength(0);
    });

    it("should return all formats when source not found", () => {
      const result = getTargetFormats(mockFormats, "unknown");
      expect(result).toHaveLength(3);
      expect(result.map(f => f.id)).toEqual(["json", "yaml", "pdf"]);
    });
  });

  // ============================================================
  // getInputMode
  // ============================================================

  describe("getInputMode", () => {
    it("should return 'text' when text mode enabled", () => {
      const availability = { enable_text_mode: true };
      expect(getInputMode(availability)).toBe("text");
    });

    it("should return 'file' when text mode disabled", () => {
      const availability = { enable_text_mode: false };
      expect(getInputMode(availability)).toBe("file");
    });

    it("should return default mode when availability is null", () => {
      expect(getInputMode(null)).toBe("file");
      expect(getInputMode(null, "text")).toBe("text");
    });

    it("should return 'file' when availability is undefined", () => {
      expect(getInputMode(undefined)).toBe("file");
    });

    it("should return custom default mode when provided", () => {
      expect(getInputMode(null, "text")).toBe("text");
      // Если enable_text_mode: false, возвращаем defaultMode
      expect(getInputMode({ enable_text_mode: false }, "text")).toBe("text");
      expect(getInputMode({ enable_text_mode: true }, "file")).toBe("text");
    });
  });

  // ============================================================
  // formatFileSize
  // ============================================================

  describe("formatFileSize", () => {
    it("should format bytes", () => {
      expect(formatFileSize(500)).toBe("500 B");
      expect(formatFileSize(1023)).toBe("1023 B");
    });

    it("should format kilobytes", () => {
      expect(formatFileSize(1024)).toBe("1.0 KB");
      expect(formatFileSize(1536)).toBe("1.5 KB");
      expect(formatFileSize(1024 * 1023)).toBe("1023.0 KB");
    });

    it("should format megabytes", () => {
      expect(formatFileSize(1024 * 1024)).toBe("1.0 MB");
      expect(formatFileSize(1024 * 1024 * 1.5)).toBe("1.5 MB");
    });

    it("should format gigabytes", () => {
      expect(formatFileSize(1024 * 1024 * 1024)).toBe("1.0 GB");
      expect(formatFileSize(1024 * 1024 * 1024 * 2.5)).toBe("2.5 GB");
    });

    it("should handle zero bytes", () => {
      expect(formatFileSize(0)).toBe("0 B");
    });

    it("should handle negative values", () => {
      expect(formatFileSize(-500)).toBe("-500 B");
      expect(formatFileSize(-1024)).toBe("-1.0 KB");
      expect(formatFileSize(-1536)).toBe("-1.5 KB");
    });

    it("should handle large numbers", () => {
      expect(formatFileSize(1024 * 1024 * 1024 * 1024)).toBe("1024.0 GB");
    });
  });

  // ============================================================
  // formatSize
  // ============================================================

  describe("formatSize", () => {
    it("should format megabytes", () => {
      expect(formatSize(1)).toBe("1 MB");
      expect(formatSize(10.5)).toBe("10.5 MB");
      expect(formatSize(1024)).toBe("1024 MB");
    });

    it("should handle zero", () => {
      expect(formatSize(0)).toBe("0 MB");
    });

    it("should handle negative values", () => {
      expect(formatSize(-1)).toBe("-1 MB");
    });

    it("should handle decimal numbers", () => {
      expect(formatSize(0.5)).toBe("0.5 MB");
      expect(formatSize(100.75)).toBe("100.75 MB");
    });
  });

  // ============================================================
  // getBaseName
  // ============================================================

  describe("getBaseName", () => {
    it("should remove extension", () => {
      expect(getBaseName("file.json")).toBe("file");
      expect(getBaseName("file.txt")).toBe("file");
      expect(getBaseName("file.tar.gz")).toBe("file.tar");
    });

    it("should remove @hash@ part", () => {
      expect(getBaseName("file@hash@abc123.json")).toBe("file");
      expect(getBaseName("my_file@hash@xyz.pdf")).toBe("my_file");
    });

    it("should handle names without extension", () => {
      expect(getBaseName("file")).toBe("file");
    });

    it("should handle empty string", () => {
      expect(getBaseName("")).toBe("");
    });

    it("should handle names with multiple dots", () => {
      expect(getBaseName("archive.tar.gz")).toBe("archive.tar");
      expect(getBaseName("my.file.name.json")).toBe("my.file.name");
    });

    it("should handle paths", () => {
      expect(getBaseName("/path/to/file.json")).toBe("file");
      expect(getBaseName("C:\\path\\to\\file.json")).toBe("file");
      expect(getBaseName("file.json")).toBe("file");
    });

    it("should handle @hash@ without extension", () => {
      expect(getBaseName("file@hash@abc123")).toBe("file");
    });
  });

  // ============================================================
  // getDefaultFileName
  // ============================================================

  describe("getDefaultFileName", () => {
    it("should create default file name", () => {
      expect(getDefaultFileName("file", "json")).toBe("formato_file.json");
      expect(getDefaultFileName("my_file", "pdf")).toBe("formato_my_file.pdf");
    });

    it("should handle empty name", () => {
      expect(getDefaultFileName("", "json")).toBe("formato_.json");
    });

    it("should handle special characters in name", () => {
      expect(getDefaultFileName("my file", "json")).toBe(
        "formato_my file.json",
      );
      expect(getDefaultFileName("file-name", "pdf")).toBe(
        "formato_file-name.pdf",
      );
    });
  });

  // ============================================================
  // getArchiveFileName
  // ============================================================

  describe("getArchiveFileName", () => {
    it("should create archive file name", () => {
      expect(getArchiveFileName("file", "json", "zip")).toBe(
        "formato_file.json",
      );
      expect(getArchiveFileName("my_file", "pdf", "tar.gz")).toBe(
        "formato_my_file.pdf",
      );
    });

    it("should handle empty name", () => {
      expect(getArchiveFileName("", "json", "zip")).toBe("formato_.json");
    });

    it("should handle different archive formats", () => {
      expect(getArchiveFileName("data", "csv", "zip")).toBe("formato_data.csv");
      expect(getArchiveFileName("data", "csv", "tar.gz")).toBe(
        "formato_data.csv",
      );
      expect(getArchiveFileName("data", "csv", "tar.xz")).toBe(
        "formato_data.csv",
      );
    });
  });

  // ============================================================
  // getUniqueFileName
  // ============================================================

  describe("getUniqueFileName", () => {
    it("should return original name if not used", () => {
      const usedNames = new Set<string>();
      expect(getUniqueFileName("file.json", usedNames)).toBe("file.json");
    });

    it("should add counter if name used", () => {
      const usedNames = new Set<string>(["file.json"]);
      expect(getUniqueFileName("file.json", usedNames)).toBe("file1.json");
    });

    it("should increment counter until unique", () => {
      const usedNames = new Set<string>([
        "file.json",
        "file1.json",
        "file2.json",
      ]);
      expect(getUniqueFileName("file.json", usedNames)).toBe("file3.json");
    });

    it("should handle names without extension", () => {
      const usedNames = new Set<string>(["file"]);
      expect(getUniqueFileName("file", usedNames)).toBe("file1");
    });

    it("should handle empty string", () => {
      const usedNames = new Set<string>([""]);
      expect(getUniqueFileName("", usedNames)).toBe("1");
    });

    it("should handle multiple counter digits", () => {
      const usedNames = new Set<string>([
        "file.json",
        "file1.json",
        "file2.json",
        "file3.json",
        "file4.json",
        "file5.json",
        "file6.json",
        "file7.json",
        "file8.json",
        "file9.json",
      ]);
      expect(getUniqueFileName("file.json", usedNames)).toBe("file10.json");
    });

    it("should preserve extension when adding counter", () => {
      const usedNames = new Set<string>(["file.pdf"]);
      expect(getUniqueFileName("file.pdf", usedNames)).toBe("file1.pdf");
    });

    it("should handle names with multiple dots", () => {
      const usedNames = new Set<string>(["file.tar.gz"]);
      expect(getUniqueFileName("file.tar.gz", usedNames)).toBe("file1.tar.gz");
    });
  });

  // ============================================================
  // getArchiveName
  // ============================================================

  describe("getArchiveName", () => {
    it("should create archive name with default format", () => {
      const name = getArchiveName();
      expect(name).toMatch(/^formato_\d+_[a-z0-9]+\.zip$/);
    });

    it("should create archive name with custom format", () => {
      const name = getArchiveName("tar.gz");
      expect(name).toMatch(/^formato_\d+_[a-z0-9]+\.tar\.gz$/);
    });

    it("should create archive name with tar.xz", () => {
      const name = getArchiveName("tar.xz");
      expect(name).toMatch(/^formato_\d+_[a-z0-9]+\.tar\.xz$/);
    });

    it("should create archive name with 7z", () => {
      const name = getArchiveName("7z");
      expect(name).toMatch(/^formato_\d+_[a-z0-9]+\.7z$/);
    });

    it("should generate unique names", () => {
      const name1 = getArchiveName();
      const name2 = getArchiveName();
      expect(name1).not.toBe(name2);
    });

    it("should have timestamp in name", () => {
      const name = getArchiveName();
      const timestampMatch = name.match(/^formato_(\d+)_/);
      expect(timestampMatch).toBeTruthy();
      const timestamp = parseInt(timestampMatch![1]);
      expect(timestamp).toBeGreaterThan(0);
      expect(timestamp).toBeLessThanOrEqual(Date.now());
    });
  });

  // ============================================================
  // fileExists
  // ============================================================

  describe("fileExists", () => {
    beforeEach(() => {
      vi.clearAllMocks();
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it("should return true if file exists", async () => {
      (invoke as any).mockResolvedValue(1024);
      const result = await fileExists("/path/to/file.txt");
      expect(result).toBe(true);
      expect(invoke).toHaveBeenCalledWith("get_file_size", {
        path: "/path/to/file.txt",
      });
    });

    it("should return false if file does not exist", async () => {
      (invoke as any).mockRejectedValue(new Error("Not found"));
      const result = await fileExists("/path/to/file.txt");
      expect(result).toBe(false);
      expect(invoke).toHaveBeenCalledWith("get_file_size", {
        path: "/path/to/file.txt",
      });
    });

    it("should handle invoke error and return false", async () => {
      (invoke as any).mockRejectedValue("Network error");
      const result = await fileExists("/path/to/file.txt");
      expect(result).toBe(false);
      expect(invoke).toHaveBeenCalledWith("get_file_size", {
        path: "/path/to/file.txt",
      });
    });
  });

  // ============================================================
  // filterExistingFiles
  // ============================================================

  describe("filterExistingFiles", () => {
    beforeEach(() => {
      vi.clearAllMocks();
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it("should return all existing when all files exist", async () => {
      const files = [
        { path: "/path/to/1.txt", id: "1" },
        { path: "/path/to/2.txt", id: "2" },
      ];
      (invoke as any).mockResolvedValue(1024);

      const result = await filterExistingFiles(files);
      expect(result.existing).toEqual(files);
      expect(result.missing).toEqual([]);
    });

    it("should return all missing when no files exist", async () => {
      const files = [
        { path: "/path/to/1.txt", id: "1" },
        { path: "/path/to/2.txt", id: "2" },
      ];
      (invoke as any).mockRejectedValue(new Error("Not found"));

      const result = await filterExistingFiles(files);
      expect(result.existing).toEqual([]);
      expect(result.missing).toEqual(files);
    });

    it("should return mixed results", async () => {
      const files = [
        { path: "/path/to/1.txt", id: "1" },
        { path: "/path/to/2.txt", id: "2" },
        { path: "/path/to/3.txt", id: "3" },
      ];
      (invoke as any).mockImplementation((cmd: string, args: any) => {
        if (args.path === "/path/to/2.txt") {
          return Promise.reject(new Error("Not found"));
        }
        return Promise.resolve(1024);
      });

      const result = await filterExistingFiles(files);
      expect(result.existing).toHaveLength(2);
      expect(result.missing).toHaveLength(1);
      expect(result.missing[0].path).toBe("/path/to/2.txt");
    });

    it("should handle empty array", async () => {
      const result = await filterExistingFiles([]);
      expect(result.existing).toEqual([]);
      expect(result.missing).toEqual([]);
    });

    it("should preserve file objects with extra properties", async () => {
      const files = [
        { path: "/path/to/1.txt", id: "1", name: "file1", size: 1024 },
        { path: "/path/to/2.txt", id: "2", name: "file2", size: 2048 },
      ];
      (invoke as any).mockImplementation((cmd: string, args: any) => {
        if (args.path === "/path/to/1.txt") {
          return Promise.resolve(1024);
        }
        return Promise.reject(new Error("Not found"));
      });

      const result = await filterExistingFiles(files);
      expect(result.existing).toHaveLength(1);
      expect(result.existing[0]).toEqual(files[0]);
      expect(result.missing).toHaveLength(1);
      expect(result.missing[0]).toEqual(files[1]);
    });
  });
});
