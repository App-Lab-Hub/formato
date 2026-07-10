<!-- src/lib/components/convert/FileList.svelte -->
<script lang="ts">
  import { m } from '$lib/paraglide/messages';
  import { Tooltip, TooltipContent, TooltipTrigger } from '$lib/components/ui/tooltip';
  import { FileText, ArrowRight, Eye, Download, Play, X, LoaderCircle, Zap, ListX, FolderArchive } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import { toast } from '$lib/utils/toast';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';

  type FileItem = { path: string; name: string; id: string };
  type TargetFormat = { id: string; name: string };
  type ConvertedFile = { path: string; format: string };

  let {
    files,
    sourceFormatId,
    selectedTarget = null,
    convertedFiles,
    convertingFiles,
    showExtensions = true,
    onconvertone,
    onconvertall,
    onclearall,
    onpreview,
    ondownload,
    onremove,
    settings,
  } = $props<{
    files: FileItem[];
    sourceFormatId: string;
    selectedTarget?: TargetFormat | null;
    convertedFiles: Map<string, ConvertedFile>;
    convertingFiles: Set<string>;
    showExtensions?: boolean;
    onconvertone: (index: number) => void;
    onconvertall: () => void;
    onclearall: () => void;
    onpreview: (fileId: string) => void;
    ondownload: (fileId: string) => void;
    onremove: (index: number) => void;
    settings: { enable_archive: boolean; archive_format: string };
  }>();
  
  let isClearing = $state(false);
  let itemsAnimated = $state<Set<string>>(new Set());
  
  async function clearAllWithAnimation() {
    if (isClearing) return;
    isClearing = true;
    
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
    await onclearall();
    isClearing = false;
  }
  
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

    onremove(index);
  }

  async function downloadAllAsArchive() {
    if (files.length === 0) {
      toast.warning(m.no_files_to_archive());
      return;
    }

    // Проверяем, что все файлы сконвертированы
    const allConverted = files.every((f: FileItem) => convertedFiles.has(f.id));
    if (!allConverted) {
      toast.warning(m.convert_all_first());
      return;
    }

    try {
      // Получаем пути всех сконвертированных файлов
      const convertedPaths: string[] = [];
      for (const f of files) {
        const converted = convertedFiles.get(f.id);
        if (converted) {
          convertedPaths.push(converted.path);
        }
      }

      if (convertedPaths.length === 0) {
        toast.warning(m.no_converted_files());
        return;
      }

      // Формируем имя архива с уникальным значением
      const archiveFormat = settings?.archive_format || 'zip';
      const timestamp = Date.now();
      const randomId = Math.random().toString(36).slice(2, 8);
      const defaultName = `formato_${timestamp}_${randomId}.${archiveFormat}`;

      const filePath = await save({
        defaultPath: defaultName,
        title: m.save_archive(),
        filters: [
          {
            name: `${archiveFormat.toUpperCase()} Archive`,
            extensions: [archiveFormat],
          },
        ],
      });

      if (!filePath) {
        toast.info(m.save_cancelled());
        return;
      }

      // 🔥 ИСПРАВЛЕНО: используем archive_multiple_files вместо create_archive
      await invoke('archive_multiple_files', {
        files: convertedPaths,
        outputPath: filePath,
        format: archiveFormat,
      });

      const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'archive';
      toast.success(m.archive_saved({ name: fileName }));
    } catch (e) {
      console.error('[Download All Archive] Failed:', e);
      toast.error(m.archive_error());
    }
  }

  function animateNewItems() {
    const items = document.querySelectorAll('[data-file-item]');
    items.forEach((el, i) => {
      const id = (el as HTMLElement).dataset.fileId;
      if (id && !itemsAnimated.has(id)) {
        itemsAnimated.add(id);
        
        // Начальное состояние: на 300px вправо и прозрачный
        (el as HTMLElement).style.transform = 'translateX(300px)';
        (el as HTMLElement).style.opacity = '0';
        
        // Запускаем анимацию к нулю с задержкой
        setTimeout(() => {
          (el as HTMLElement).animate(
            [
              { transform: 'translateX(300px)', opacity: 0 },
              { transform: 'translateX(0)', opacity: 1 },
            ],
            { duration: 300, easing: 'ease-out', fill: 'forwards' },
          );
        }, i * 50);
      }
    });
  }

  // Проверяем, все ли файлы сконвертированы
  const allConverted = $derived(
    files.length > 0 && files.every((f: FileItem) => convertedFiles.has(f.id))
  );

  // Проверяем, включена ли архивация в настройках
  const isArchiveEnabled = $derived(settings?.enable_archive ?? false);

  // Анимируем новые элементы после монтирования
  onMount(() => {
    animateNewItems();
  });

  // Следим за изменениями файлов
  $effect(() => {
    // Сбрасываем анимацию для удалённых файлов
    const currentIds = new Set(files.map((f: FileItem) => f.id));
    for (const id of itemsAnimated) {
      if (!currentIds.has(id)) {
        itemsAnimated.delete(id);
      }
    }
    
    // Анимируем новые файлы
    animateNewItems();
  });
</script>

{#if files.length > 0}
  <div class="w-full max-w-4xl flex flex-col gap-4">
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
              <button onclick={onconvertall} class="cursor-pointer inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 dark:bg-violet-600 light:bg-purple-500 text-white hover:dark:bg-violet-700 hover:light:bg-purple-600 h-9 w-9 shadow-sm shadow-violet-500/20 group/btn">
                <Zap class="h-4 w-4 fill-current group-hover/btn:scale-110 transition-transform" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
              <p>{m.convert_all()}</p>
            </TooltipContent>
          </Tooltip>
        {/if}

        <!-- Кнопка "Скачать все как архив" -->
        {#if isArchiveEnabled && allConverted}
        <Tooltip>
            <TooltipTrigger>
            <button 
                onclick={downloadAllAsArchive} 
                class="cursor-pointer inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 dark:bg-violet-600 light:bg-purple-500 text-white hover:dark:bg-violet-700 hover:light:bg-purple-600 h-9 w-9 shadow-sm shadow-violet-500/20"
            >
                <FolderArchive class="h-4 w-4 group-hover:scale-110 transition-transform" />
            </button>
            </TooltipTrigger>
            <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
            <p>{m.download_all_archive()}</p>
            </TooltipContent>
        </Tooltip>
        {/if}

        <Tooltip>
          <TooltipTrigger>
            <button onclick={clearAllWithAnimation} class="cursor-pointer inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-destructive/10 hover:text-destructive h-9 w-9 dark:text-muted-foreground light:text-purple-600/60 group/btn">
              <ListX class="h-5 w-5 group-hover/btn:scale-110 transition-transform" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
            <p>{m.clear_all()}</p>
          </TooltipContent>
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
                <button onclick={() => onpreview(file.id)} disabled={!savedPath} class="cursor-pointer inline-flex items-center justify-center rounded-md dark:text-muted-foreground light:text-purple-600/60 dark:hover:text-foreground light:hover:text-purple-800 h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <Eye class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
                <p>{savedPath ? m.preview() : m.convert_first()}</p>
              </TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger>
                <button onclick={() => ondownload(file.id)} disabled={!savedPath} class="cursor-pointer inline-flex items-center justify-center rounded-md dark:text-muted-foreground light:text-purple-600/60 dark:hover:text-foreground light:hover:text-purple-800 h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <Download class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
                <p>{savedPath ? m.download() : m.convert_first()}</p>
              </TooltipContent>
            </Tooltip>

            {#if selectedTarget}
              <Tooltip>
                <TooltipTrigger>
                  {#if isConverting}
                    <span class="inline-flex items-center justify-center rounded-md h-8 w-8">
                      <LoaderCircle class="h-4 w-4 dark:text-violet-500 light:text-violet-600 animate-spin" />
                    </span>
                  {:else}
                    <button onclick={() => onconvertone(i)} class="cursor-pointer inline-flex items-center justify-center rounded-md dark:text-muted-foreground light:text-purple-700 dark:hover:bg-violet-500 light:hover:bg-purple-500/50 dark:hover:text-white light:hover:text-white h-8 w-8 transition-all duration-200 bg-transparent hover:bg-purple-500/20">
                      <Play class="h-4 w-4" />
                    </button>
                  {/if}
                </TooltipTrigger>
                <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
                  <p>{isConverting ? m.converting() : m.convert()}</p>
                </TooltipContent>
              </Tooltip>
            {/if}

            <Tooltip>
              <TooltipTrigger>
                <button onclick={() => removeFile(i)} disabled={isConverting} class="cursor-pointer inline-flex items-center justify-center rounded-md dark:text-muted-foreground light:text-purple-600/60 dark:hover:bg-destructive/10 light:hover:bg-destructive/10 dark:hover:text-destructive light:hover:text-destructive h-8 w-8 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
                  <X class="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
                <p>{isConverting ? m.cannot_remove() : m.remove()}</p>
              </TooltipContent>
            </Tooltip>
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}