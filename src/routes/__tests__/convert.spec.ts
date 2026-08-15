import { describe, it, expect } from "vitest";
import {
  getTargetFormats,
  getTargetFormatsWithAvailability,
  getInputMode,
  formatFileSize,
  formatSize,
  getBaseName,
  getDefaultFileName,
  getArchiveFileName,
  getUniqueFileName,
  getArchiveName,
} from "$lib/utils/convert";

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
  });

  // ============================================================
  // getTargetFormatsWithAvailability
  // ============================================================

  describe("getTargetFormatsWithAvailability", () => {
    it("should mark formats as available", () => {
      const availability = { available_formats: ["yaml", "pdf"] };
      const result = getTargetFormatsWithAvailability(
        mockFormats,
        "json",
        availability,
      );
      expect(result).toHaveLength(2);
      expect(result[0].available).toBe(true);
      expect(result[1].available).toBe(true);
    });

    it("should mark formats as unavailable", () => {
      const availability = { available_formats: ["pdf"] };
      const result = getTargetFormatsWithAvailability(
        mockFormats,
        "json",
        availability,
      );
      expect(result).toHaveLength(2);
      expect(result[0].available).toBe(false);
      expect(result[1].available).toBe(true);
    });

    it("should handle null availability", () => {
      const result = getTargetFormatsWithAvailability(
        mockFormats,
        "json",
        null,
      );
      expect(result).toHaveLength(2);
      expect(result[0].available).toBe(false);
      expect(result[1].available).toBe(false);
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
  });

  // ============================================================
  // getDefaultFileName
  // ============================================================

  describe("getDefaultFileName", () => {
    it("should create default file name", () => {
      expect(getDefaultFileName("file", "json")).toBe("formato_file.json");
      expect(getDefaultFileName("my_file", "pdf")).toBe("formato_my_file.pdf");
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
  });
});
