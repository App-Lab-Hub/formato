// src/routes/__tests__/settings.spec.ts
import { describe, it, expect } from "vitest";
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

    it("should format sizes less than 1 as MB", () => {
      expect(getMaxPreviewLabel(0.25)).toBe("250 MB");
      expect(getMaxPreviewLabel(0.5)).toBe("500 MB");
      expect(getMaxPreviewLabel(0.999)).toBe("999 MB");
    });

    it("should format sizes 1 and above as MB", () => {
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
        synthesis: { "ru_RU-dmitri-medium": { exists: true } },
        recognition: {},
      };
      const result = hasAnyModels(status);
      expect(result.hasSynthesis).toBe(true);
      expect(result.hasRecognition).toBe(false);
    });

    it("should return true if any recognition model exists", () => {
      const status = {
        synthesis: {},
        recognition: { "ggml-tiny-q5_1.bin": { exists: true } },
      };
      const result = hasAnyModels(status);
      expect(result.hasSynthesis).toBe(false);
      expect(result.hasRecognition).toBe(true);
    });

    it("should return false if no models exist", () => {
      const status = {
        synthesis: { "ru_RU-dmitri-medium": { exists: false } },
        recognition: { "ggml-tiny-q5_1.bin": { exists: false } },
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
  // getSynthesisModelDisplay — 100% ПОКРЫТИЕ
  // ============================================================

  describe("getSynthesisModelDisplay", () => {
    // Строка 57: if (!model) return model;
    it("should return empty string for empty model", () => {
      expect(getSynthesisModelDisplay("")).toBe("");
    });

    // Строка 58: if (parts.length >= 2) … else return model;
    it("should return original when less than 2 parts", () => {
      expect(getSynthesisModelDisplay("ru")).toBe("ru");
      expect(getSynthesisModelDisplay("ru_RU")).toBe("ru_RU");
      expect(getSynthesisModelDisplay("something")).toBe("something");
    });

    // Строка 59: const langPart = parts[0];
    // Строка 60: langPart с "_" → берём первую часть до "_" и переводим в верхний регистр
    it("should extract language from langPart with underscore", () => {
      expect(getSynthesisModelDisplay("ru_RU-dmitri-medium")).toBe(
        "RU - dmitri (medium)",
      );
      expect(getSynthesisModelDisplay("en_US-lessac-medium")).toBe(
        "EN - lessac (medium)",
      );
    });

    // Строка 60: langPart без "_" → используется langPart (с .toUpperCase())
    it("should use langPart as language when no underscore", () => {
      expect(getSynthesisModelDisplay("ru-dmitri-medium")).toBe(
        "RU - dmitri (medium)",
      );
      expect(getSynthesisModelDisplay("en-lessac-medium")).toBe(
        "EN - lessac (medium)",
      );
    });

    // Строка 61: const name = parts[1];
    it("should extract name from parts[1]", () => {
      expect(getSynthesisModelDisplay("ru_RU-dmitri")).toBe("RU - dmitri");
      expect(getSynthesisModelDisplay("en_US-amy")).toBe("EN - amy");
    });

    // Строка 62: const sizeParts = parts.slice(2);
    // Строка 63: const size = sizeParts.length > 0 ? ` (${sizeParts.join("-")})` : "";
    it("should include size when present", () => {
      expect(getSynthesisModelDisplay("ru_RU-dmitri-medium")).toBe(
        "RU - dmitri (medium)",
      );
      expect(getSynthesisModelDisplay("ru-dmitri-medium")).toBe(
        "RU - dmitri (medium)",
      );
    });

    // Строка 63: size = "" когда sizeParts.length === 0
    it("should omit size when absent", () => {
      expect(getSynthesisModelDisplay("ru_RU-dmitri")).toBe("RU - dmitri");
      expect(getSynthesisModelDisplay("en_US-amy")).toBe("EN - amy");
    });

    // Строки 60 + 63 вместе: нет подчеркивания, нет размера
    it("should handle model without underscore and without size", () => {
      expect(getSynthesisModelDisplay("ru-dmitri")).toBe("RU - dmitri");
      expect(getSynthesisModelDisplay("en-amy")).toBe("EN - amy");
    });

    // Строка 63: sizeParts.join("-") объединяет все части
    it("should join all remaining parts with -", () => {
      expect(getSynthesisModelDisplay("en_US-amy-medium-v2")).toBe(
        "EN - amy (medium-v2)",
      );
      expect(getSynthesisModelDisplay("en_US-amy-medium-v2-beta")).toBe(
        "EN - amy (medium-v2-beta)",
      );
    });

    // Строка 64: return `${lang} - ${name}${size}`;
    it("should return final formatted string", () => {
      expect(getSynthesisModelDisplay("ru_RU-dmitri-medium")).toBe(
        "RU - dmitri (medium)",
      );
      expect(getSynthesisModelDisplay("en_US-amy")).toBe("EN - amy");
      expect(getSynthesisModelDisplay("ru-dmitri")).toBe("RU - dmitri");
    });
  });
});
