<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { onMount } from 'svelte';
  import { tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Upload, Play, X, FileText, Zap, ListX, LoaderCircle, Eye, Download, ArrowRight } from 'lucide-svelte';
  import Tooltip from '$lib/components/ui/tooltip/tooltip.svelte';
  import TooltipTrigger from '$lib/components/ui/tooltip/tooltip-trigger.svelte';
  import TooltipContent from '$lib/components/ui/tooltip/tooltip-content.svelte';

 let {
    sourceFormatId,
    sourceFormatName,
    sourceFormatExtensions = [sourceFormatId],
    selectedTarget,
    files = [],
    convertingFiles = new Set(),
    convertedFiles = new Map(),
    counter = 0,
    fileHashes = new Map(),
    showExtensions = true,
    onfileschange,
    onconvertone,
    onconvertall,
    onclearall,
    onpreview,
    ondownload,
  }: {
    sourceFormatId: string;
    sourceFormatName: string;
    sourceFormatExtensions?: string[];
    selectedTarget: { id: string } | null;
    files: { path: string; name: string; id: string }[];
    convertingFiles: Set<string>;
    convertedFiles: Map<string, { path: string; format: string }>;
    counter: number;
    fileHashes: Map<string, string>;
    showExtensions?: boolean;
    onfileschange: (files: { path: string; name: string; id: string }[]) => void;
    onconvertone: (index: number) => void;
    onconvertall: () => void;
    onclearall: () => void;
    onpreview: (fileId: string) => void;
    ondownload: (fileId: string) => void;
  } = $props();

  let _counter = $state(counter);
  let isDragOver = $state(false);
  let dropzoneEl = $state<HTMLElement | null>(null);

  async function hashFilePath(path: string): Promise<string> {
    return await invoke<string>('hash_file', { path });
  }

  async function processAndAddPaths(paths: string[]) {
    const validPaths = paths.filter(path => {
      const ext = path.split('.').pop()?.toLowerCase();
      return ext ? sourceFormatExtensions.includes(ext) : false;
    });

    if (validPaths.length === 0) return;

    const knownHashes = new Set(fileHashes.values());

    const newPaths: { path: string; hash: string }[] = [];
    for (const path of validPaths) {
      try {
        const hash = await hashFilePath(path);
        if (!knownHashes.has(hash)) {
          knownHashes.add(hash);
          newPaths.push({ path, hash });
        }
      } catch (e) {
        console.warn(`Failed to hash file: ${path}`, e);
      }
    }

    if (newPaths.length === 0) return;

    const ids = newPaths.map(() => `${_counter++}-${Date.now()}`);
    const newFiles = newPaths.map(({ path, hash }, idx) => {
      const id = ids[idx];
      fileHashes.set(id, hash);
      return {
        path,
        name: path.split('/').pop() || path.split('\\').pop() || path,
        id,
      };
    });

    onfileschange([...files, ...newFiles]);
    await tick();

    for (const f of newFiles) {
      const el = document.querySelector(`[data-file-id="${f.id}"]`) as HTMLElement;
      el?.animate(
        [
          { transform: 'translateX(30px)', opacity: 0 },
          { transform: 'translateX(0)', opacity: 1 },
        ],
        { duration: 300, easing: 'ease-out', fill: 'forwards' },
      );
    }
  }

  async function addPendingFiles() {
    const storageKey = `pending_files_${sourceFormatId}`;
    const pending = sessionStorage.getItem(storageKey);
    if (!pending) return;
    
    sessionStorage.removeItem(storageKey);
    const paths: string[] = JSON.parse(pending);
    await processAndAddPaths(paths);
  }

  async function pickFile() {
    const result = await open({
      multiple: true,
      filters: [{ name: `${sourceFormatName} files`, extensions: sourceFormatExtensions }],
    });

    if (result) {
      const paths = Array.isArray(result) ? result : [result];
      
      const storageKey = `pending_files_${sourceFormatId}`;
      const pending = JSON.parse(sessionStorage.getItem(storageKey) || '[]');
      sessionStorage.setItem(storageKey, JSON.stringify([...pending, ...paths]));
      
      await processAndAddPaths(paths);
    }
  }

  function isOverDropzone(x: number, y: number): boolean {
    if (!dropzoneEl) return false;
    const rect = dropzoneEl.getBoundingClientRect();
    return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
  }

  onMount(() => {
    addPendingFiles();
    
    const webview = getCurrentWebview();

    const unlisten = webview.onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        const pos = event.payload.position;
        const x = pos.x / window.devicePixelRatio;
        const y = pos.y / window.devicePixelRatio;
        const over = isOverDropzone(x, y);
        isDragOver = over;
        document.body.style.cursor = over ? 'copy' : 'default';
      } else if (event.payload.type === 'drop') {
        isDragOver = false;
        document.body.style.cursor = '';
        const pos = event.payload.position;
        const x = pos.x / window.devicePixelRatio;
        const y = pos.y / window.devicePixelRatio;
        if (isOverDropzone(x, y) && event.payload.paths?.length) {
          processAndAddPaths(event.payload.paths);
        }
      } else if (event.payload.type === 'leave') {
        isDragOver = false;
        document.body.style.cursor = '';
      }
    });

    return () => {
      document.body.style.cursor = '';
      unlisten.then(fn => fn());
    };
  });

  async function removeFile(index: number) {
    const file = files[index];
    if (convertingFiles.has(file.id)) return;

    const el = document.querySelector(`[data-file-id="${file.id}"]`) as HTMLElement;
    if (el) {
      await el.animate(
        [
          { transform: 'translateX(0)', opacity: 1 },
          { transform: 'translateX(300px)', opacity: 0 },
        ],
        { duration: 300, easing: 'ease-in', fill: 'forwards' },
      ).finished;
    }

    onfileschange(files.filter((_, i) => i !== index));
  }

  async function clearAllWithAnimation() {
    const items = document.querySelectorAll('[data-file-item]');
    const animations = Array.from(items).map(el =>
      (el as HTMLElement).animate(
        [
          { transform: 'translateX(0)', opacity: 1 },
          { transform: 'translateX(300px)', opacity: 0 },
        ],
        { duration: 300, easing: 'ease-in', fill: 'forwards' },
      ).finished,
    );
    await Promise.all(animations);
    onclearall();
  }
</script>

<button
  bind:this={dropzoneEl}
  onclick={pickFile}
  class="group w-full max-w-4xl min-h-[180px] flex flex-col items-center justify-center gap-4 rounded-2xl border-2 border-dashed bg-card/30 duration-300 cursor-pointer transition-all {isDragOver
    ? 'border-primary bg-primary/10'
    : 'border-border hover:border-primary/50 hover:bg-primary/5'}"
>
  <div class="rounded-full bg-secondary p-4 transition-colors duration-300 {isDragOver ? 'bg-primary/20 text-primary' : 'group-hover:bg-primary/10 group-hover:text-primary'}">
    <Upload class="h-8 w-8 {isDragOver ? 'text-primary scale-110' : 'text-muted-foreground group-hover:text-primary'}" />
  </div>
  <div class="text-center space-y-1">
    {#if isDragOver}
      <p class="text-base font-medium text-primary">Drop your files here</p>
    {:else}
      <p class="text-base font-medium text-foreground">
        Drop your <span class="text-primary">{sourceFormatName}</span> files here
      </p>
      <p class="text-sm text-muted-foreground">or click to browse filesystem</p>
    {/if}
  </div>
</button>

{#if files.length > 0}
  <div class="w-full max-w-4xl flex flex-col gap-4">
    <div class="flex items-center justify-between px-1">
      <div class="flex items-center gap-2">
        <span class="flex h-7 w-7 items-center justify-center rounded-full bg-violet-500/10 text-xs font-bold text-violet-600 dark:text-violet-400">
          {files.length}
        </span>
        <span class="text-base font-medium text-muted-foreground">
          {files.length === 1 ? 'File queued' : 'Files queued'}
        </span>
      </div>

      <div class="flex items-center gap-2">
        {#if selectedTarget}
          <Tooltip>
            <TooltipTrigger>
              <button onclick={onconvertall} class="cursor-pointer inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-violet-600 text-white hover:bg-violet-700 h-9 w-9 shadow-sm shadow-violet-500/20 group/btn">
                <Zap class="h-4 w-4 fill-current group-hover/btn:scale-110 transition-transform" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>Convert all</p></TooltipContent>
          </Tooltip>
        {/if}

        <Tooltip>
          <TooltipTrigger>
            <button onclick={clearAllWithAnimation} class="cursor-pointer inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-destructive/10 hover:text-destructive h-9 w-9 text-muted-foreground group/btn">
              <ListX class="h-5 w-5 group-hover/btn:scale-110 transition-transform" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>Clear all</p></TooltipContent>
        </Tooltip>
      </div>
    </div>

    <div class="flex flex-col gap-2">
      {#each files as file, i (file.id)}
        {@const isConverting = convertingFiles.has(file.id)}
        {@const savedPath = convertedFiles.get(file.id)}
          <div
            data-file-item
            data-file-id={file.id}
            class="group relative flex items-center gap-4 rounded-xl border border-border/50 bg-card/50 p-3.5 transition-all duration-200 hover:bg-violet-500/5 hover:border-violet-500/20 hover:shadow-sm"
            class:opacity-70={isConverting}
          >
          <div class="shrink-0 flex items-center justify-center w-10 h-10 rounded-lg bg-secondary/50 text-muted-foreground group-hover:text-violet-500 transition-colors">
            <FileText class="h-5 w-5" />
          </div>

<div class="flex flex-col flex-1 min-w-0 gap-0.5">
  <div class="flex items-center gap-2">
    <span class="text-base font-medium text-foreground truncate" title={file.name}>
      {showExtensions ? file.name : file.name.replace(/\.[^.]+$/, '')}
    </span>
    {#if savedPath}
      <span class="shrink-0 text-[10px] font-semibold uppercase tracking-wider text-emerald-400 bg-emerald-400/10 px-1.5 py-0.5 rounded-md">
        {savedPath.format}
      </span>
    {/if}
  </div>
  {#if selectedTarget}
    <div class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground/80">
      <span class="uppercase tracking-wide opacity-70">{sourceFormatId}</span>
      <ArrowRight class="h-3.5 w-3.5 text-violet-500" />
      <span class="uppercase tracking-wide text-violet-600 dark:text-violet-400">{selectedTarget.id}</span>
    </div>
  {/if}
</div>

          <div class="flex items-center gap-1 shrink-0 pl-3 border-l border-border/50 ml-2">
            <Tooltip>
              <TooltipTrigger>
                <button onclick={() => onpreview(file.id)} disabled={!savedPath} class="cursor-pointer inline-flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <Eye class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{savedPath ? 'Preview' : 'Convert first'}</p></TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger>
                <button onclick={() => ondownload(file.id)} disabled={!savedPath} class="cursor-pointer inline-flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <Download class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{savedPath ? 'Download' : 'Convert first'}</p></TooltipContent>
            </Tooltip>

            {#if selectedTarget}
              <Tooltip>
                <TooltipTrigger>
                  {#if isConverting}
                    <span class="inline-flex items-center justify-center rounded-md h-8 w-8">
                      <LoaderCircle class="h-4 w-4 text-violet-500 animate-spin" />
                    </span>
                  {:else}
                    <button onclick={() => onconvertone(i)} class="cursor-pointer inline-flex items-center justify-center rounded-md text-muted-foreground hover:bg-violet-500 hover:text-white h-8 w-8 transition-all duration-200">
                      <Play class="h-4 w-4" />
                    </button>
                  {/if}
                </TooltipTrigger>
                <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{isConverting ? 'Converting...' : 'Convert'}</p></TooltipContent>
              </Tooltip>
            {/if}

            <Tooltip>
              <TooltipTrigger>
                <button onclick={() => removeFile(i)} disabled={isConverting} class="cursor-pointer inline-flex items-center justify-center rounded-md text-muted-foreground hover:bg-destructive/10 hover:text-destructive h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <X class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{isConverting ? 'Cannot remove' : 'Remove'}</p></TooltipContent>
            </Tooltip>
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}