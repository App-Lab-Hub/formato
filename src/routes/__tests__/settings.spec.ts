import { describe, it, expect, vi } from "vitest";
import {
  getMaxPreviewLabel,
  isModelDownloaded,
  isRecognitionModelDownloaded,
  hasAnyModels,
  getSynthesisModelDisplay,
} from "$lib/utils/settings";

describe("settings utils", () => {
  const mockModelsStatus = {
    synthesis: {
      "ru_RU-dmitri-medium": { exists: true },
      "ru_RU-irina-medium": { exists: false },
      "en_US-lessac-medium": { exists: true },
    },
    recognition: {
      "ggml-tiny-q5_1.bin": { exists: true },
      "ggml-base-q5_1.bin": { exists: false },
    },
  };

  // ============================================================
  // getMaxPreviewLabel
  // ============================================================

  describe("getMaxPreviewLabel", () => {
    it("should return ∞ for size 0", () => {
      expect(getMaxPreviewLabel(0)).toBe("∞");
    });

    it("should format sizes less than 1 as MB with 3 decimals", () => {
      expect(getMaxPreviewLabel(0.25)).toBe("250 MB");
      expect(getMaxPreviewLabel(0.5)).toBe("500 MB");
      expect(getMaxPreviewLabel(0.999)).toBe("999 MB");
    });

    it("should format sizes 1 and above as MB without decimals", () => {
      expect(getMaxPreviewLabel(1)).toBe("1 MB");
      expect(getMaxPreviewLabel(10)).toBe("10 MB");
      expect(getMaxPreviewLabel(100)).toBe("100 MB");
      expect(getMaxPreviewLabel(1024)).toBe("1024 MB");
    });

    it("should handle integer values correctly", () => {
      expect(getMaxPreviewLabel(0)).toBe("∞");
      expect(getMaxPreviewLabel(1)).toBe("1 MB");
      expect(getMaxPreviewLabel(50)).toBe("50 MB");
    });
  });

  // ============================================================
  // isModelDownloaded
  // ============================================================

  describe("isModelDownloaded", () => {
    it("should return true if synthesis model exists", () => {
      expect(isModelDownloaded("ru_RU-dmitri-medium", mockModelsStatus)).toBe(
        true,
      );
    });

    it("should return false if synthesis model does not exist", () => {
      expect(isModelDownloaded("ru_RU-irina-medium", mockModelsStatus)).toBe(
        false,
      );
    });

    it("should return false if model not in list", () => {
      expect(isModelDownloaded("non-existent-model", mockModelsStatus)).toBe(
        false,
      );
    });

    it("should return false if modelsStatus is null", () => {
      expect(isModelDownloaded("ru_RU-dmitri-medium", null)).toBe(false);
    });
  });

  // ============================================================
  // isRecognitionModelDownloaded
  // ============================================================

  describe("isRecognitionModelDownloaded", () => {
    it("should return true if recognition model exists", () => {
      expect(
        isRecognitionModelDownloaded("ggml-tiny-q5_1.bin", mockModelsStatus),
      ).toBe(true);
    });

    it("should return false if recognition model does not exist", () => {
      expect(
        isRecognitionModelDownloaded("ggml-base-q5_1.bin", mockModelsStatus),
      ).toBe(false);
    });

    it("should return false if model not in list", () => {
      expect(
        isRecognitionModelDownloaded("non-existent-model", mockModelsStatus),
      ).toBe(false);
    });

    it("should return false if modelsStatus is null", () => {
      expect(isRecognitionModelDownloaded("ggml-tiny-q5_1.bin", null)).toBe(
        false,
      );
    });
  });

  // ============================================================
  // hasAnyModels
  // ============================================================

  describe("hasAnyModels", () => {
    it("should return true if any synthesis model exists", () => {
      const status = {
        synthesis: {
          "ru_RU-dmitri-medium": { exists: true },
          "ru_RU-irina-medium": { exists: false },
        },
        recognition: {
          "ggml-tiny-q5_1.bin": { exists: false },
        },
      };
      const result = hasAnyModels(status);
      expect(result.hasSynthesis).toBe(true);
      expect(result.hasRecognition).toBe(false);
    });

    it("should return true if any recognition model exists", () => {
      const status = {
        synthesis: {
          "ru_RU-dmitri-medium": { exists: false },
        },
        recognition: {
          "ggml-tiny-q5_1.bin": { exists: true },
        },
      };
      const result = hasAnyModels(status);
      expect(result.hasSynthesis).toBe(false);
      expect(result.hasRecognition).toBe(true);
    });

    it("should return false if no models exist", () => {
      const status = {
        synthesis: {
          "ru_RU-dmitri-medium": { exists: false },
        },
        recognition: {
          "ggml-tiny-q5_1.bin": { exists: false },
        },
      };
      const result = hasAnyModels(status);
      expect(result.hasSynthesis).toBe(false);
      expect(result.hasRecognition).toBe(false);
    });

    it("should return false if modelsStatus is null", () => {
      const result = hasAnyModels(null);
      expect(result.hasSynthesis).toBe(false);
      expect(result.hasRecognition).toBe(false);
    });

    it("should handle empty objects", () => {
      const status = {
        synthesis: {},
        recognition: {},
      };
      const result = hasAnyModels(status);
      expect(result.hasSynthesis).toBe(false);
      expect(result.hasRecognition).toBe(false);
    });
  });

  // ============================================================
  // getSynthesisModelDisplay
  // ============================================================

  describe("getSynthesisModelDisplay", () => {
    it("should format model name with language, name and size", () => {
      expect(getSynthesisModelDisplay("ru_RU-dmitri-medium")).toBe(
        "RU - dmitri (medium)",
      );
      expect(getSynthesisModelDisplay("en_US-lessac-medium")).toBe(
        "EN - lessac (medium)",
      );
      expect(getSynthesisModelDisplay("ru_RU-irina-medium")).toBe(
        "RU - irina (medium)",
      );
    });

    it("should handle model names without size", () => {
      expect(getSynthesisModelDisplay("ru_RU-dmitri")).toBe("RU - dmitri");
    });

    it("should handle model names with extra parts", () => {
      expect(getSynthesisModelDisplay("en_US-amy-medium-v2")).toBe(
        "EN - amy (medium)",
      );
    });

    it("should return original string if format is unexpected", () => {
      expect(getSynthesisModelDisplay("unknown")).toBe("unknown");
      expect(getSynthesisModelDisplay("")).toBe("");
    });
  });
});
