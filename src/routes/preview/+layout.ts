export const ssr = false;

import { invoke } from "@tauri-apps/api/core";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async ({ url }) => {
  const path = url.searchParams.get("path") ?? "";
  const lang = url.searchParams.get("lang") ?? "";
  const title = url.searchParams.get("title") ?? "Preview";
  const size = parseInt(url.searchParams.get("size") ?? "0");
  const maxSize = parseFloat(url.searchParams.get("maxSize") ?? "5");

  if (!path) {
    return {
      content: "",
      lang: "",
      title: "Preview",
      size: 0,
      maxSize: 5,
      blocked: false,
    };
  }

  // Проверяем лимит ДО загрузки контента
  const maxSizeBytes = maxSize === 0 ? Infinity : maxSize * 1024 * 1024;
  const blocked = size > maxSizeBytes;

  if (blocked) {
    return {
      content: "",
      lang: decodeURIComponent(lang),
      title: decodeURIComponent(title),
      size,
      maxSize,
      blocked: true,
    };
  }

  const content = await invoke<string>("read_file_content", {
    path: decodeURIComponent(path),
  });

  return {
    content,
    lang: decodeURIComponent(lang),
    title: decodeURIComponent(title),
    size,
    maxSize,
    blocked: false,
  };
};
