// src/routes/settings/+layout.ts
import type { LayoutLoad } from "./$types";
import { getModelsStatus, type ModelsStatus } from "$lib/data/models";

export const load: LayoutLoad = async () => {
  try {
    const modelsStatus = await getModelsStatus();
    console.log("✅ Models status loaded in layout:", modelsStatus);
    return {
      modelsStatus,
    };
  } catch (e) {
    console.error("❌ Failed to load models status in layout:", e);
    return {
      modelsStatus: null,
    };
  }
};
