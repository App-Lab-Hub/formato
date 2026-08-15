import { describe, it, expect } from "vitest";
import {
  getTypeLabel,
  getTypeColor,
  formatDate,
  getEmptyMessage,
  filterFiles,
  getCurrentPageFiles,
  getTotalPages,
  goToPage,
  getPaginationInfo,
  getPageNumbers,
} from "$lib/utils/files";

describe("files utils", () => {
  const mockFiles = [
    { name: "file1.json", file_type: "converted" },
    { name: "file2.txt", file_type: "temp" },
    { name: "file3.pdf", file_type: "converted" },
    { name: "file4.tmp", file_type: "temp" },
    { name: "file5.xml", file_type: "converted" },
  ];

  const mockMessages = {
    noSearchResults: "No files matching your search",
    emptyAll: "No files in folders",
    emptyConverted: "No converted files",
    emptyTemp: "No temporary files",
    empty: "No files",
  };

  // ============================================================
  // getTypeLabel
  // ============================================================

  describe("getTypeLabel", () => {
    it("should return 'Converted' for converted type", () => {
      expect(getTypeLabel("converted")).toBe("Converted");
    });

    it("should return 'Temporary' for temp type", () => {
      expect(getTypeLabel("temp")).toBe("Temporary");
    });
  });

  // ============================================================
  // getTypeColor
  // ============================================================

  describe("getTypeColor", () => {
    it("should return emerald colors for converted type", () => {
      expect(getTypeColor("converted")).toBe(
        "text-emerald-400 bg-emerald-400/10",
      );
    });

    it("should return amber colors for temp type", () => {
      expect(getTypeColor("temp")).toBe("text-amber-400 bg-amber-400/10");
    });
  });

  // ============================================================
  // formatDate
  // ============================================================

  describe("formatDate", () => {
    it("should format date with default locale", () => {
      const dateStr = "2024-01-15T14:30:00";
      const result = formatDate(dateStr);
      // en-US формат: MM/DD/YYYY
      expect(result).toMatch(/01\/15\/2024/);
    });

    it("should format date with Russian locale", () => {
      const dateStr = "2024-01-15T14:30:00";
      const result = formatDate(dateStr, "ru");
      // ru-RU формат: DD.MM.YYYY
      expect(result).toMatch(/15\.01\.2024/);
    });

    it("should return 'Unknown' for invalid date", () => {
      expect(formatDate("invalid")).toBe("Unknown");
      expect(formatDate("")).toBe("Unknown");
    });

    it("should handle date objects", () => {
      const date = new Date("2024-01-15T14:30:00");
      const result = formatDate(date.toString());
      expect(result).toBeTruthy();
    });
  });

  // ============================================================
  // getEmptyMessage
  // ============================================================

  describe("getEmptyMessage", () => {
    it("should return no search results message when searchQuery is not empty", () => {
      expect(getEmptyMessage("test", "all", mockMessages)).toBe(
        "No files matching your search",
      );
    });

    it("should return empty all message for 'all' filter", () => {
      expect(getEmptyMessage("", "all", mockMessages)).toBe(
        "No files in folders",
      );
    });

    it("should return empty converted message for 'converted' filter", () => {
      expect(getEmptyMessage("", "converted", mockMessages)).toBe(
        "No converted files",
      );
    });

    it("should return empty temp message for 'temp' filter", () => {
      expect(getEmptyMessage("", "temp", mockMessages)).toBe(
        "No temporary files",
      );
    });
  });

  // ============================================================
  // filterFiles
  // ============================================================

  describe("filterFiles", () => {
    it("should return all files when filterType is 'all'", () => {
      const result = filterFiles(mockFiles, "", "all");
      expect(result).toHaveLength(5);
    });

    it("should filter by search query", () => {
      const result = filterFiles(mockFiles, "file1", "all");
      expect(result).toHaveLength(1);
      expect(result[0].name).toBe("file1.json");
    });

    it("should filter by type", () => {
      const result = filterFiles(mockFiles, "", "converted");
      expect(result).toHaveLength(3);
      expect(result.every(f => f.file_type === "converted")).toBe(true);
    });

    it("should filter by type and search", () => {
      const result = filterFiles(mockFiles, "file", "converted");
      expect(result).toHaveLength(3);
    });

    it("should be case insensitive", () => {
      const result = filterFiles(mockFiles, "FILE1", "all");
      expect(result).toHaveLength(1);
    });
  });

  // ============================================================
  // getCurrentPageFiles
  // ============================================================

  describe("getCurrentPageFiles", () => {
    const items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    it("should return first page items", () => {
      const result = getCurrentPageFiles(items, 0, 3);
      expect(result).toEqual([1, 2, 3]);
    });

    it("should return second page items", () => {
      const result = getCurrentPageFiles(items, 1, 3);
      expect(result).toEqual([4, 5, 6]);
    });

    it("should handle last page with fewer items", () => {
      const result = getCurrentPageFiles(items, 3, 3);
      expect(result).toEqual([10]);
    });

    it("should return empty array for page out of range", () => {
      const result = getCurrentPageFiles(items, 10, 3);
      expect(result).toEqual([]);
    });
  });

  // ============================================================
  // getTotalPages
  // ============================================================

  describe("getTotalPages", () => {
    it("should calculate total pages correctly", () => {
      expect(getTotalPages(10, 3)).toBe(4);
      expect(getTotalPages(9, 3)).toBe(3);
      expect(getTotalPages(0, 3)).toBe(0);
      expect(getTotalPages(5, 5)).toBe(1);
    });
  });

  // ============================================================
  // goToPage
  // ============================================================

  describe("goToPage", () => {
    it("should return the page if within range", () => {
      expect(goToPage(2, 5)).toBe(2);
    });

    it("should return 0 if page is negative", () => {
      expect(goToPage(-1, 5)).toBe(0);
    });

    it("should return last page if page exceeds total", () => {
      expect(goToPage(10, 5)).toBe(4);
    });
  });

  // ============================================================
  // getPaginationInfo
  // ============================================================

  describe("getPaginationInfo", () => {
    it("should calculate from and to correctly", () => {
      const result = getPaginationInfo(0, 20, 45);
      expect(result.from).toBe(1);
      expect(result.to).toBe(20);
    });

    it("should handle last page", () => {
      const result = getPaginationInfo(2, 20, 45);
      expect(result.from).toBe(41);
      expect(result.to).toBe(45);
    });

    it("should handle empty list", () => {
      const result = getPaginationInfo(0, 20, 0);
      expect(result.from).toBe(1);
      expect(result.to).toBe(0);
    });
  });

  // ============================================================
  // getPageNumbers
  // ============================================================

  describe("getPageNumbers", () => {
    it("should return all pages when totalPages <= 7", () => {
      const result = getPageNumbers(0, 5);
      expect(result).toEqual([0, 1, 2, 3, 4]);
    });

    it("should show first, last, current and neighbors", () => {
      const result = getPageNumbers(3, 10);
      expect(result).toContain(0);
      expect(result).toContain(9);
      expect(result).toContain(3);
      expect(result).toContain(2);
      expect(result).toContain(4);
    });

    it("should add ellipsis for gaps", () => {
      const result = getPageNumbers(5, 20);
      expect(result).toContain("...");
    });
  });
});
