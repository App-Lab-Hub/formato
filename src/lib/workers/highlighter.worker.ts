// src/lib/workers/highlighter.worker.ts
import { getSingletonHighlighter } from "shiki";

let highlighter: Awaited<ReturnType<typeof getSingletonHighlighter>> | null =
  null;

async function getH() {
  if (!highlighter) {
    highlighter = await getSingletonHighlighter({
      themes: ["github-dark"],
      langs: ["json", "yaml", "xml", "toml", "html", "markdown", "ini", "txt"],
    });
  }
  return highlighter;
}

self.onmessage = async (
  e: MessageEvent<{
    id: number;
    code: string;
    lang: string;
    startLine: number;
  }>,
) => {
  const { id, code, lang, startLine } = e.data;
  const h = await getH();
  const html = await h.codeToHtml(code, { lang, theme: "github-dark" });
  self.postMessage({ id, html, startLine });
};
