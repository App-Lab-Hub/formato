// src/lib/data/models.ts
import { invoke } from "@tauri-apps/api/core";

export interface ModelStatus {
  exists: boolean;
  path?: string;
  size?: number;
}

export interface ModelsStatus {
  synthesis: {
    [key: string]: ModelStatus; // "ru_RU-dmitri-medium": { exists: true, ... }
  };
  recognition: {
    [key: string]: ModelStatus; // "ggml-tiny-q5_1.bin": { exists: true, ... }
  };
  hasAnySynthesis: boolean;
  hasAnyRecognition: boolean;
}

export async function getModelsStatus(): Promise<ModelsStatus> {
  return await invoke<ModelsStatus>("get_models_status");
}

export async function downloadSynthesisModel(modelName: string): Promise<void> {
  await invoke("download_synthesis_model", { modelName });
}

export async function downloadRecognitionModel(
  modelName: string,
): Promise<void> {
  await invoke("download_recognition_model", { modelName });
}
