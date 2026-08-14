// src/routes/files/+layout.ts
import type { FileInfo } from "$lib/types/files";
import type { LayoutLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";

export const load: LayoutLoad = async () => {
  try {
    const response = await invoke<{
      files: FileInfo[];
      total_files: number;
      total_size: number;
      converted_count: number;
      temp_count: number;
    }>("get_files");

    return {
      files: response.files,
      totalFiles: response.total_files,
      totalSize: response.total_size,
      convertedCount: response.converted_count,
      tempCount: response.temp_count,
    };
  } catch (error) {
    console.error("Failed to load files:", error);
    return {
      files: [],
      totalFiles: 0,
      totalSize: 0,
      convertedCount: 0,
      tempCount: 0,
    };
  }
};
