<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { ArrowLeft, LoaderCircle } from 'lucide-svelte';
  import { getFormatById, getFormats, isFormatsLoaded } from '$lib/data/formats';
  import { invoke } from '@tauri-apps/api/core';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import 'overlayscrollbars/overlayscrollbars.css';
  import TooltipProvider from '$lib/components/ui/tooltip/tooltip-provider.svelte';
  import SourceFormatHeader from '$lib/components/convert/SourceFormatHeader.svelte';
  import TargetFormatGrid from '$lib/components/convert/TargetFormatGrid.svelte';
  import FileDropZone from '$lib/components/convert/FileDropZone.svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { onMount } from 'svelte';
  import type { Format } from '$lib/types/format';
  import { browser } from '$app/environment';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { m } from '$lib/paraglide/messages';

  const sourceFormatId: string = page.params.format!;
  let settings = $derived(page.data.settings);
  function getStorageKey(base: string): string {
    return `convert_${sourceFormatId}_${base}`;
  }
  
  let sourceFormat = $state<Format | undefined>(getFormatById(sourceFormatId));
  let isLoading = $state(!isFormatsLoaded() && !sourceFormat);
  let loadError = $state<string | null>(null);
  let targetFormats = $state<Format[]>([]);

  let savedTargetId = browser ? sessionStorage.getItem(getStorageKey('selectedTargetId')) : null;
  let selectedTarget = $state<Format | null>(null);
  
  let files = $state<{ path: string; name: string; id: string }[]>([]);
  let convertedFiles = $state<Map<string, { path: string; format: string }>>(new Map());
  let convertingFiles = $state<Set<string>>(new Set());
  let counter = $state(0);
  let fileHashes = $state<Map<string, string>>(new Map());

  function loadFilesFromStorage() {
    if (!browser) return;
    try {
      const saved = sessionStorage.getItem(getStorageKey('files'));
      if (saved) files = JSON.parse(saved);
      
      const savedConverted = sessionStorage.getItem(getStorageKey('converted'));
      if (savedConverted) convertedFiles = new Map(JSON.parse(savedConverted));
      
      const savedCounter = sessionStorage.getItem(getStorageKey('counter'));
      if (savedCounter) counter = parseInt(savedCounter);

      const savedHashes = sessionStorage.getItem(getStorageKey('hashes'));
      if (savedHashes) fileHashes = new Map(JSON.parse(savedHashes));
    } catch (e) {
      console.warn('Failed to load files from sessionStorage', e);
    }
  }

  function saveFilesToStorage() {
    if (!browser) return;
    try {
      sessionStorage.setItem(getStorageKey('files'), JSON.stringify(files));
      sessionStorage.setItem(getStorageKey('converted'), JSON.stringify(Array.from(convertedFiles.entries())));
      sessionStorage.setItem(getStorageKey('counter'), String(counter));
      sessionStorage.setItem(getStorageKey('hashes'), JSON.stringify(Array.from(fileHashes.entries())));
    } catch (e) {
      console.warn('Failed to save files to sessionStorage', e);
    }
  }

  loadFilesFromStorage();

  onMount(() => {
    if (sourceFormat) {
      targetFormats = getFormats().filter(f => f.id !== sourceFormatId);
      if (savedTargetId) {
        const found = targetFormats.find(f => f.id === savedTargetId);
        if (found) selectedTarget = found;
      }
      isLoading = false;
      return;
    }

    if (!isFormatsLoaded()) {
      const checkFormats = setInterval(() => {
        if (isFormatsLoaded()) {
          const f = getFormatById(sourceFormatId);
          if (f) {
            sourceFormat = f;
            targetFormats = getFormats().filter(f => f.id !== sourceFormatId);
            if (savedTargetId) {
              const found = targetFormats.find(f => f.id === savedTargetId);
              if (found) selectedTarget = found;
            }
            isLoading = false;
          } else {
            loadError = m.format_not_found() + ` "${sourceFormatId}"`;
            isLoading = false;
          }
          clearInterval(checkFormats);
        }
      }, 100);
      return () => clearInterval(checkFormats);
    } else {
      loadError = m.format_not_found() + ` "${sourceFormatId}"`;
      isLoading = false;
    }
  });

  $effect(() => {
    saveFilesToStorage();
  });

  function goBack() { 
    goto('/'); 
  }
  
  function selectTarget(format: Format) {
    if (selectedTarget?.id === format.id) {
      selectedTarget = null;
      if (browser) sessionStorage.removeItem(getStorageKey('selectedTargetId'));
      return;
    }
    selectedTarget = format;
    if (browser) sessionStorage.setItem(getStorageKey('selectedTargetId'), format.id);
  }
  
  function handleFilesChange(newFiles: typeof files) {
    const newIds = new Set(newFiles.map(f => f.id));
    for (const id of fileHashes.keys()) {
      if (!newIds.has(id)) fileHashes.delete(id);
    }
    files = newFiles;
  }

  async function convertOne(index: number, skipPreview = false) {
    const file = files[index];
    if (!selectedTarget || convertingFiles.has(file.id)) return;
    convertingFiles.add(file.id);
    try {
      const result = await invoke<{ success: boolean; content: string; extension: string | null; error: string | null }>(
        'convert_file', { 
          path: file.path, 
          from: sourceFormatId, 
          to: selectedTarget.id,
          enableCache: settings?.enable_cache ?? true
        }
      );
      if (result.success) {
        convertedFiles = new Map(convertedFiles.set(file.id, {
          path: result.content,
          format: result.extension || selectedTarget.id
        }));

        if (!skipPreview && settings?.auto_preview) {
          previewFileFn(file.id);
        }
      }
    } catch (e) { console.error(`Conversion failed: ${file.name}`, e); }
    finally { convertingFiles.delete(file.id); }
  }

  async function convertAll() { 
    for (let i = 0; i < files.length; i++) await convertOne(i, true); 
  }
  
  function clearAll() { 
    files = []; 
    convertedFiles = new Map();
    fileHashes = new Map();
    if (browser) {
      sessionStorage.removeItem(getStorageKey('files'));
      sessionStorage.removeItem(getStorageKey('converted'));
      sessionStorage.removeItem(getStorageKey('hashes'));
    }
  }

  async function downloadFile(fileId: string) {
    const converted = convertedFiles.get(fileId);
    if (!converted) return;
    const file = files.find(f => f.id === fileId);
    const baseName = file?.name.replace(/\.[^.]+$/, '') ?? 'file';
    
    try {
      const isArchive = settings?.enable_archive && settings?.archive_format;
      const ext = isArchive ? settings.archive_format : converted.format;
      const defaultName = `${baseName}.${ext}`;
      
      const filePath = await save({
        defaultPath: defaultName,
        title: isArchive ? m.settings_archive() : 'Save file',
      });
      
      if (!filePath) return;
      
      if (isArchive) {
        await invoke('archive_file', { 
          sourcePath: converted.path, 
          outputPath: filePath, 
          format: settings.archive_format 
        });
      } else {
        const content = await invoke<string>('read_file_content', { path: converted.path });
        await writeTextFile(filePath, content);
      }
    } catch (e) { console.error('[Download] Failed:', e); }
  }

  async function previewFileFn(fileId: string) {
    const converted = convertedFiles.get(fileId);
    const savedPath = converted?.path ?? files.find(f => f.id === fileId)?.path;
    if (!savedPath) return;
    try {
      const actualSize = await invoke<number>('get_file_size', { path: savedPath });
      const format = converted?.format ?? sourceFormatId;
      const file = files.find(f => f.id === fileId);
      const baseName = file?.name.replace(/\.[^.]+$/, '') ?? 'file';
      const title = converted ? `${baseName}.${format}` : file?.name ?? 'file';
      const windowId = `preview-${Date.now()}`;
      const maxSizeMB = settings?.max_preview_size ?? 5;
      
      new WebviewWindow(windowId, {
        url: `/preview?path=${encodeURIComponent(savedPath)}&lang=${format}&title=${encodeURIComponent(title)}&size=${actualSize}&maxSize=${maxSizeMB}`,
        title,
        width: 900, height: 700,
        resizable: true, center: true,
        maximizable: true, minimizable: true, closable: true,
        transparent: false,
        backgroundColor: { red: 30, green: 30, blue: 30, alpha: 1 },
        theme: 'dark',
        minWidth: 400, minHeight: 300
      });
    } catch (e) { console.error('Preview failed:', e); }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<ScrollContainer>
  {#if isLoading}
    <div class="flex items-center justify-center min-h-screen bg-background">
      <LoaderCircle class="h-16 w-16 text-primary animate-spin" />
    </div>
  {:else if loadError}
    <div class="flex flex-col items-center justify-center min-h-screen bg-background gap-4">
      <p class="text-red-400 text-xl">{loadError}</p>
      <button onclick={() => goto('/')} class="text-primary hover:underline text-sm">
        {m.settings_back()}
      </button>
    </div>
  {:else if sourceFormat}
    <TooltipProvider>
      <div class="flex flex-col bg-background text-foreground min-h-screen">
        <main class="flex flex-col items-center gap-10 px-8 py-20 max-w-[1700px] mx-auto w-full">
          <button onclick={goBack} class="cursor-pointer absolute top-6 left-6 flex items-center gap-2 dark:text-muted-foreground light:text-muted-foreground/70 dark:hover:text-primary light:hover:text-primary transition-colors">
            <ArrowLeft class="h-5 w-5" />
            <span class="text-sm">{m.settings_back()}</span>
          </button>

          <SourceFormatHeader format={sourceFormat} />
          <TargetFormatGrid formats={targetFormats} {selectedTarget} onselect={selectTarget} />
                
      <FileDropZone
        sourceFormatId={sourceFormatId}
        sourceFormatName={sourceFormat?.name ?? ''}
        sourceFormatExtensions={sourceFormat?.extensions ?? [sourceFormatId]}
        {selectedTarget}
        {files}
        {convertingFiles}
        {convertedFiles}
        {counter}
        {fileHashes}
        showExtensions={settings?.show_extensions ?? true}
        onfileschange={handleFilesChange}
        onconvertone={convertOne}
        onconvertall={convertAll}
        onclearall={clearAll}
        onpreview={previewFileFn}
        ondownload={downloadFile}
      />
        </main>
      </div>
    </TooltipProvider>
  {/if}
</ScrollContainer>