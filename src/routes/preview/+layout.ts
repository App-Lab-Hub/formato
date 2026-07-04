export const ssr = false;

import { invoke } from "@tauri-apps/api/core";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async ({ url }) => {
  const path = url.searchParams.get("path") ?? "";
  const lang = url.searchParams.get("lang") ?? "";
  const title = url.searchParams.get("title") ?? "Preview";
  const size = url.searchParams.get("size") ?? "0";
  const maxSize = url.searchParams.get("maxSize") ?? "5";
  console.log("path: ", path);
  if (!path) {
    return { content: "", lang: "", title: "Preview", size: 0, maxSize: 5 };
  }

  const content = await invoke<string>("read_file_content", {
    path: decodeURIComponent(path),
  });

  return {
    content,
    lang: decodeURIComponent(lang),
    title: decodeURIComponent(title),
    size: parseInt(size),
    maxSize: parseFloat(maxSize),
  };
};
