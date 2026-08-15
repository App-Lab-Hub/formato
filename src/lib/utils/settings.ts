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

  const parts = model.split("-");
  if (parts.length >= 2) {
    const langPart = parts[0];
    const lang = langPart.split("_")[0].toUpperCase();

    const name = parts[1];

    const sizeParts = parts.slice(2);
    const size = sizeParts.length > 0 ? ` (${sizeParts.join("-")})` : "";

    return `${lang} - ${name}${size}`;
  }

  return model;
}
