<!-- // FileDropZone.svelte -->
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
  import { m } from '$lib/paraglide/messages';
  import { toast } from '$lib/utils/toast';
  import { showContextMenu, getDefaultActions } from '$lib/utils/context-menu';

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
  let pendingProcessed = false;

  async function hashFilePath(path: string): Promise<string> {
    return await invoke<string>('hash_file', { path });
  }

  async function processAndAddPaths(paths: string[], suppressToast: boolean = false) {
    const validPaths = paths.filter(path => {
      const ext = path.split('.').pop()?.toLowerCase();
      return ext ? sourceFormatExtensions.includes(ext) : false;
    });

    if (validPaths.length === 0) {
      if (!suppressToast) {
        toast.warning(m.file_no_suitable());
      }
      clearPendingFiles();
      return;
    }

    const knownHashes = new Set(fileHashes.values());
    const newPaths: { path: string; hash: string }[] = [];
    let duplicateCount = 0;

    for (const path of validPaths) {
      try {
        const hash = await hashFilePath(path);
        if (!knownHashes.has(hash)) {
          knownHashes.add(hash);
          newPaths.push({ path, hash });
        } else {
          duplicateCount++;
        }
      } catch (e) {
        console.warn(`Failed to hash file: ${path}`, e);
      }
    }

    if (newPaths.length === 0) {
      if (!suppressToast) {
        if (duplicateCount > 0) {
          toast.warning(m.file_duplicate_all({ count: duplicateCount }));
        } else {
          toast.warning(m.file_no_new());
        }
      }
      return;
    }

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
        
    if (!suppressToast) {
      if (newFiles.length > 0) {
        toast.success(m.file_added({ count: newFiles.length }));
      }
      if (duplicateCount > 0) {
        toast.warning(m.file_duplicate_skipped({ count: duplicateCount }));
      }
    }
    
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

  function clearPendingFiles() {
    const storageKey = `pending_files_${sourceFormatId}`;
    sessionStorage.removeItem(storageKey);
  }

  async function addPendingFiles() {
    if (pendingProcessed) return;
    pendingProcessed = true;

    const storageKey = `pending_files_${sourceFormatId}`;
    const pending = sessionStorage.getItem(storageKey);
    if (!pending) return;
    
    try {
      const paths: string[] = JSON.parse(pending);
      const existingPaths = new Set(files.map(f => f.path));
      const newPaths = paths.filter(p => !existingPaths.has(p));
      
      if (newPaths.length > 0) {
        await processAndAddPaths(newPaths, true);
      } else {
        clearPendingFiles();
      }
    } catch (e) {
      console.warn('Failed to process pending files:', e);
      clearPendingFiles();
    }
  }

  async function pickFile() {
    const result = await open({
      multiple: true,
      filters: [{ name: `${sourceFormatName} files`, extensions: sourceFormatExtensions }],
    });

    if (result) {
      const paths = Array.isArray(result) ? result : [result];
      
      const storageKey = `pending_files_${sourceFormatId}`;
      sessionStorage.setItem(storageKey, JSON.stringify(paths));
      
      await processAndAddPaths(paths);
    }
  }

  function isOverDropzone(x: number, y: number): boolean {
    if (!dropzoneEl) return false;
    const rect = dropzoneEl.getBoundingClientRect();
    return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
  }

  // Функция для обработки контекстного меню файла
  function handleFileContextMenu(e: MouseEvent, fileId: string) {
    const file = files.find(f => f.id === fileId);
    if (!file) return;

    const isConverted = convertedFiles.has(fileId);
    const isConverting = convertingFiles.has(fileId);

    // Получаем стандартные действия для файла
    const actions = getDefaultActions(fileId, {
      onConvert: (id) => {
        const index = files.findIndex(f => f.id === id);
        if (index !== -1) onconvertone(index);
      },
      onDownload: (id) => ondownload(id),
      onPreview: (id) => onpreview(id),
      onRemove: (id) => {
        const index = files.findIndex(f => f.id === id);
        if (index !== -1) removeFile(index);
      },
      isConverted,
      isConverting,
    });

    // Показываем контекстное меню
    showContextMenu(e, { items: actions });
  }

  // Функция для контекстного меню пустой области в списке файлов
  function handleEmptyAreaContextMenu(e: MouseEvent) {
    e.preventDefault();
    
    const actions = [
      {
        label: 'Добавить файлы',
        action: () => pickFile(),
      },
      {
        label: '---',
        action: () => {},
      },
      {
        label: 'Очистить всё',
        action: () => {
          if (files.length > 0) {
            clearAllWithAnimation();
          }
        },
        disabled: files.length === 0,
      },
    ];

    showContextMenu(e, { items: actions });
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
    toast.info(m.file_removed({ name: file.name }));
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

<!-- HTML с обработчиками контекстного меню -->

<button
  bind:this={dropzoneEl}
  onclick={pickFile}
  data-context-menu="ignore"
  class="group w-full max-w-4xl min-h-[180px] flex flex-col items-center justify-center gap-4 rounded-2xl border-2 border-dashed bg-card/30 duration-300 cursor-pointer transition-all {isDragOver
    ? 'border-primary bg-primary/10'
    : 'dark:border-border light:border-purple-300/40 dark:hover:border-primary/50 light:hover:border-purple-500/60 dark:hover:bg-primary/5 light:hover:bg-purple-200/40'}"
>
  <div class="rounded-full p-4 transition-all duration-500 ease-out group-hover:bg-primary/20 group-hover:text-primary {isDragOver ? 'bg-primary/30 text-primary scale-105' : 'bg-purple-200/60 text-purple-600 dark:bg-purple-500/20 dark:text-purple-300'}">
    <Upload class="h-8 w-8 transition-transform duration-500 ease-out group-hover:scale-110 {isDragOver ? 'scale-110' : ''}" />
  </div>
  <div class="text-center space-y-1">
    {#if isDragOver}
      <p class="text-base font-medium text-primary">{m.drop_files_here()}</p>
    {:else}
      <p class="text-base font-medium dark:text-foreground light:text-purple-800">
        {m.drop_files_text()} <span class="text-primary font-semibold">{sourceFormatName}</span> {m.drop_files_file()}
      </p>
      <p class="text-sm dark:text-muted-foreground light:text-purple-600/70">{m.or_click()}</p>
    {/if}
  </div>
</button>

{#if files.length > 0}
  <div 
    class="w-full max-w-4xl flex flex-col gap-4"
    oncontextmenu={handleEmptyAreaContextMenu}
  >
    <div class="flex items-center justify-between px-1">
      <div class="flex items-center gap-2">
        <span class="flex h-7 w-7 items-center justify-center rounded-full dark:bg-violet-500/20 light:bg-purple-300/60 text-xs font-bold dark:text-violet-400 light:text-purple-700">
          {files.length}
        </span>
        <span class="text-base font-medium dark:text-muted-foreground light:text-purple-700/70">
          {files.length === 1 ? m.file_queued() : m.files_queued()}
        </span>
      </div>

      <div class="flex items-center gap-2">
        {#if selectedTarget}
          <Tooltip>
            <TooltipTrigger>
              <button onclick={onconvertall} data-context-menu="ignore" class="cursor-pointer inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 dark:bg-violet-600 light:bg-purple-500 text-white hover:dark:bg-violet-700 hover:light:bg-purple-600 h-9 w-9 shadow-sm shadow-violet-500/20 group/btn">
                <Zap class="h-4 w-4 fill-current group-hover/btn:scale-110 transition-transform" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{m.convert_all()}</p></TooltipContent>
          </Tooltip>
        {/if}

        <Tooltip>
          <TooltipTrigger>
            <button onclick={clearAllWithAnimation} data-context-menu="ignore" class="cursor-pointer inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-destructive/10 hover:text-destructive h-9 w-9 dark:text-muted-foreground light:text-purple-600/60 group/btn">
              <ListX class="h-5 w-5 group-hover/btn:scale-110 transition-transform" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{m.clear_all()}</p></TooltipContent>
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
          oncontextmenu={(e) => handleFileContextMenu(e, file.id)}
          class="group relative flex items-center gap-4 rounded-xl border dark:border-border/50 light:border-purple-300/40 dark:bg-card/50 light:bg-purple-200/40 p-3.5 transition-all duration-200 dark:hover:bg-violet-500/10 light:hover:bg-purple-200/70 dark:hover:border-violet-500/20 light:hover:border-purple-400/50 hover:shadow-sm"
          class:opacity-70={isConverting}
        >
          <div class="shrink-0 flex items-center justify-center w-10 h-10 rounded-lg dark:bg-violet-500/20 light:bg-purple-300/60 dark:text-violet-400 light:text-purple-700 dark:group-hover:bg-violet-500/30 light:group-hover:bg-purple-400/60 dark:group-hover:text-violet-300 light:group-hover:text-purple-800 transition-colors">
            <FileText class="h-5 w-5" />
          </div>

          <div class="flex flex-col flex-1 min-w-0 gap-0.5">
            <div class="flex items-center gap-2">
              <span class="text-base font-medium dark:text-foreground light:text-purple-800/90 truncate" title={file.name}>
                {showExtensions ? file.name : file.name.replace(/\.[^.]+$/, '')}
              </span>
              {#if savedPath}
                <span class="shrink-0 text-[10px] font-semibold uppercase tracking-wider dark:text-emerald-400 light:text-emerald-600 dark:bg-emerald-400/10 light:bg-emerald-500/10 px-1.5 py-0.5 rounded-md">
                  {savedPath.format}
                </span>
              {/if}
            </div>
            {#if selectedTarget}
              <div class="flex items-center gap-1.5 text-xs font-medium dark:text-muted-foreground/80 light:text-purple-700/60">
                <span class="uppercase tracking-wide opacity-70">{sourceFormatId}</span>
                <ArrowRight class="h-3.5 w-3.5 dark:text-violet-500 light:text-violet-600" />
                <span class="uppercase tracking-wide dark:text-violet-400 light:text-violet-600">{selectedTarget.id}</span>
              </div>
            {/if}
          </div>

          <div class="flex items-center gap-1 shrink-0 pl-3 border-l dark:border-border/50 light:border-purple-300/40 ml-2">
            <Tooltip>
              <TooltipTrigger>
                <button onclick={() => onpreview(file.id)} disabled={!savedPath} data-context-menu="ignore" class="cursor-pointer inline-flex items-center justify-center rounded-md dark:text-muted-foreground light:text-purple-600/60 dark:hover:text-foreground light:hover:text-purple-800 h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <Eye class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{savedPath ? m.preview() : m.convert_first()}</p></TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger>
                <button onclick={() => ondownload(file.id)} disabled={!savedPath} data-context-menu="ignore" class="cursor-pointer inline-flex items-center justify-center rounded-md dark:text-muted-foreground light:text-purple-600/60 dark:hover:text-foreground light:hover:text-purple-800 h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <Download class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{savedPath ? m.download() : m.convert_first()}</p></TooltipContent>
            </Tooltip>

            {#if selectedTarget}
              <Tooltip>
                <TooltipTrigger>
                  {#if isConverting}
                    <span class="inline-flex items-center justify-center rounded-md h-8 w-8">
                      <LoaderCircle class="h-4 w-4 dark:text-violet-500 light:text-violet-600 animate-spin" />
                    </span>
                  {:else}
                    <button onclick={() => onconvertone(i)} data-context-menu="ignore" class="cursor-pointer inline-flex items-center justify-center rounded-md dark:text-muted-foreground light:text-purple-700 dark:hover:bg-violet-500 light:hover:bg-purple-500/50 dark:hover:text-white light:hover:text-white h-8 w-8 transition-all duration-200 bg-transparent hover:bg-purple-500/20">
                      <Play class="h-4 w-4" />
                    </button>
                  {/if}
                </TooltipTrigger>
                <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{isConverting ? m.converting() : m.convert()}</p></TooltipContent>
              </Tooltip>
            {/if}

            <Tooltip>
              <TooltipTrigger>
                <button onclick={() => removeFile(i)} disabled={isConverting} data-context-menu="ignore" class="cursor-pointer inline-flex items-center justify-center rounded-md dark:text-muted-foreground light:text-purple-600/60 dark:hover:bg-destructive/10 light:hover:bg-destructive/10 dark:hover:text-destructive light:hover:text-destructive h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <X class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md"><p>{isConverting ? m.cannot_remove() : m.remove()}</p></TooltipContent>
            </Tooltip>
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}