import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  normalizeIndex,
  goToConvert,
  restoreSplidePosition,
  shouldClearOnRefresh,
  SPLIDE_INDEX_KEY,
} from "$lib/utils/splide";

describe("splide utils", () => {
  // ============================================================
  // normalizeIndex
  // ============================================================

  describe("normalizeIndex", () => {
    it("should return same index for positive values within range", () => {
      expect(normalizeIndex(0, 5)).toBe(0);
      expect(normalizeIndex(2, 5)).toBe(2);
      expect(normalizeIndex(4, 5)).toBe(4);
    });

    it("should wrap index for values >= length", () => {
      expect(normalizeIndex(5, 5)).toBe(0);
      expect(normalizeIndex(6, 5)).toBe(1);
      expect(normalizeIndex(10, 5)).toBe(0);
    });

    it("should handle negative indices", () => {
      expect(normalizeIndex(-1, 5)).toBe(4);
      expect(normalizeIndex(-2, 5)).toBe(3);
      expect(normalizeIndex(-5, 5)).toBe(0);
      expect(normalizeIndex(-6, 5)).toBe(4);
    });

    it("should handle edge cases", () => {
      expect(normalizeIndex(0, 1)).toBe(0);
      expect(normalizeIndex(-1, 1)).toBe(0);
      expect(normalizeIndex(1, 1)).toBe(0);
      expect(normalizeIndex(0, 0)).toBe(0);
    });
  });

  // ============================================================
  // goToConvert
  // ============================================================

  describe("goToConvert", () => {
    let mockGoto: ReturnType<typeof vi.fn<(url: string) => void>>;
    let mockSessionStorage: Storage;

    beforeEach(() => {
      mockGoto = vi.fn<(url: string) => void>();
      mockSessionStorage = {
        getItem: vi.fn(),
        setItem: vi.fn(),
        removeItem: vi.fn(),
        clear: vi.fn(),
        length: 0,
        key: vi.fn(),
      };
    });

    it("should navigate to correct URL", () => {
      goToConvert("json", 0, true, mockSessionStorage, mockGoto);
      expect(mockGoto).toHaveBeenCalledWith("/convert/json");
    });

    it("should save index to sessionStorage when browser is true", () => {
      goToConvert("yaml", 2, true, mockSessionStorage, mockGoto);
      expect(mockSessionStorage.setItem).toHaveBeenCalledWith(
        SPLIDE_INDEX_KEY,
        "2",
      );
    });

    it("should NOT save to sessionStorage when index is negative", () => {
      goToConvert("json", -1, true, mockSessionStorage, mockGoto);
      expect(mockSessionStorage.setItem).not.toHaveBeenCalled();
    });

    it("should NOT save to sessionStorage when browser is false", () => {
      goToConvert("json", 1, false, mockSessionStorage, mockGoto);
      expect(mockSessionStorage.setItem).not.toHaveBeenCalled();
    });
  });

  // ============================================================
  // restoreSplidePosition
  // ============================================================

  describe("restoreSplidePosition", () => {
    let mockSplide: { go: ReturnType<typeof vi.fn> };
    let mockSetRestoring: ReturnType<typeof vi.fn<(value: boolean) => void>>;
    let mockSessionStorage: Storage;

    beforeEach(() => {
      mockSplide = { go: vi.fn() };
      mockSetRestoring = vi.fn<(value: boolean) => void>();
      mockSessionStorage = {
        getItem: vi.fn(),
        setItem: vi.fn(),
        removeItem: vi.fn(),
        clear: vi.fn(),
        length: 0,
        key: vi.fn(),
      };
    });

    it("should restore position from sessionStorage", () => {
      mockSessionStorage.getItem = vi.fn().mockReturnValue("2");

      const result = restoreSplidePosition(
        mockSplide,
        false,
        mockSetRestoring,
        5,
        mockSessionStorage,
      );

      expect(result).toBe(true);
      expect(mockSetRestoring).toHaveBeenCalledWith(true);
      expect(mockSetRestoring).toHaveBeenCalledWith(false);
      expect(mockSplide.go).toHaveBeenCalledWith(2, 0);
    });

    it("should not restore if splideInstance is null", () => {
      const result = restoreSplidePosition(
        null,
        false,
        mockSetRestoring,
        5,
        mockSessionStorage,
      );

      expect(result).toBe(false);
      expect(mockSessionStorage.getItem).not.toHaveBeenCalled();
    });

    it("should not restore if isRestoring is true", () => {
      const result = restoreSplidePosition(
        mockSplide,
        true,
        mockSetRestoring,
        5,
        mockSessionStorage,
      );

      expect(result).toBe(false);
      expect(mockSessionStorage.getItem).not.toHaveBeenCalled();
    });

    it("should not restore if saved index is out of range", () => {
      mockSessionStorage.getItem = vi.fn().mockReturnValue("10");

      const result = restoreSplidePosition(
        mockSplide,
        false,
        mockSetRestoring,
        5,
        mockSessionStorage,
      );

      expect(result).toBe(false);
      expect(mockSplide.go).not.toHaveBeenCalled();
    });

    it("should handle errors", () => {
      mockSessionStorage.getItem = vi.fn().mockImplementation(() => {
        throw new Error("Storage error");
      });

      const result = restoreSplidePosition(
        mockSplide,
        false,
        mockSetRestoring,
        5,
        mockSessionStorage,
      );

      expect(result).toBe(false);
      expect(mockSetRestoring).toHaveBeenCalledWith(false);
    });
  });

  // ============================================================
  // shouldClearOnRefresh
  // ============================================================

  describe("shouldClearOnRefresh", () => {
    it("should return true for navigation type 1 (refresh)", () => {
      const mockPerformance = {
        navigation: {
          type: 1,
        },
      } as Performance;

      expect(shouldClearOnRefresh(mockPerformance)).toBe(true);
    });

    it("should return false for navigation type 0 (normal navigation)", () => {
      const mockPerformance = {
        navigation: {
          type: 0,
        },
      } as Performance;

      expect(shouldClearOnRefresh(mockPerformance)).toBe(false);
    });

    it("should return false if performance is undefined", () => {
      expect(shouldClearOnRefresh(undefined as any)).toBe(false);
    });

    it("should return false if navigation is undefined", () => {
      const mockPerformance = {} as Performance;
      expect(shouldClearOnRefresh(mockPerformance)).toBe(false);
    });
  });
});
