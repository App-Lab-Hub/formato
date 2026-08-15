// src/lib/utils/settings.ts
export function getMaxPreviewLabel(size: number): string {
  if (size === 0) return "∞";
  if (size < 1) return `${Math.round(size * 1000)} MB`;
  return `${size} MB`;
}

export function isModelDownloaded(
  modelName: string,
  modelsStatus: {
    synthesis: Record<string, { exists: boolean }>;
    recognition: Record<string, { exists: boolean }>;
  } | null,
): boolean {
  if (!modelsStatus) return false;
  return modelsStatus.synthesis[modelName]?.exists || false;
}

export function isRecognitionModelDownloaded(
  modelName: string,
  modelsStatus: {
    synthesis: Record<string, { exists: boolean }>;
    recognition: Record<string, { exists: boolean }>;
  } | null,
): boolean {
  if (!modelsStatus) return false;
  return modelsStatus.recognition[modelName]?.exists || false;
}

export function hasAnyModels(
  modelsStatus: {
    synthesis: Record<string, { exists: boolean }>;
    recognition: Record<string, { exists: boolean }>;
  } | null,
): { hasSynthesis: boolean; hasRecognition: boolean } {
  if (!modelsStatus) {
    return { hasSynthesis: false, hasRecognition: false };
  }

  const hasSynthesis = Object.values(modelsStatus.synthesis).some(
    m => m.exists,
  );
  const hasRecognition = Object.values(modelsStatus.recognition).some(
    m => m.exists,
  );

  return { hasSynthesis, hasRecognition };
}

export function getSynthesisModelDisplay(model: string): string {
  if (!model) return model;

  // Формат: ru_RU-dmitri-medium
  // или en_US-amy-medium-v2
  const parts = model.split("-");
  if (parts.length >= 2) {
    // Берем первую часть (ru_RU) и извлекаем код языка (ru → RU)
    const langPart = parts[0];
    const lang = langPart.split("_")[0]?.toUpperCase() || langPart;

    // Вторая часть — имя модели
    const name = parts[1] || "";

    // Третья часть — размер (если есть)
    const size = parts[2] || "";

    return `${lang} - ${name}${size ? ` (${size})` : ""}`;
  }
  return model;
}
