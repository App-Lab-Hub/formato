// preview/+layout.ts
export const ssr = false;
export const prerender = false;

import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async ({ url }) => {
  const path = url.searchParams.get("path") ?? "";
  const lang = url.searchParams.get("lang") ?? "";
  const title = url.searchParams.get("title") ?? "Preview";
  const size = parseInt(url.searchParams.get("size") ?? "0");
  const maxSize = parseFloat(url.searchParams.get("maxSize") ?? "5");

  return {
    path,
    lang: decodeURIComponent(lang),
    title: decodeURIComponent(title),
    size,
    maxSize,
  };
};
