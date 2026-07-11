// src/routes/files/+layout.ts
import type { FileInfo } from "$lib/types/files";
import type { LayoutLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";

export const load: LayoutLoad = async () => {
  try {
    const files = await invoke<FileInfo[]>("get_files");
    return {
      files,
    };
  } catch (error) {
    console.error("Failed to load files:", error);
    return {
      files: [],
    };
  }
};
