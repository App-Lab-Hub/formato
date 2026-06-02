// src/lib/highlighter.ts
type ChunkCallback = (startLine: number, html: string) => void;

let worker: Worker | null = null;
let callbacks = new Map<number, ChunkCallback>();
let idCounter = 0;

function getWorker(): Worker {
  if (!worker) {
    worker = new Worker(
      new URL("../workers/highlighter.worker.ts", import.meta.url),
      { type: "module" },
    );
    worker.onmessage = (
      e: MessageEvent<{ id: number; html: string; startLine: number }>,
    ) => {
      const { id, html, startLine } = e.data;
      const cb = callbacks.get(id);
      if (cb) {
        cb(startLine, html);
        callbacks.delete(id);
      }
    };
  }
  return worker;
}

const CHUNK_SIZE = 20;

export function highlightChunkAsync(
  lines: string[],
  startLine: number,
  lang: string,
  onDone: ChunkCallback,
): void {
  const w = getWorker();
  const id = ++idCounter;
  callbacks.set(id, onDone);

  const chunk = lines.slice(startLine, startLine + CHUNK_SIZE).join("\n");
  w.postMessage({ id, code: chunk, lang, startLine });
}
