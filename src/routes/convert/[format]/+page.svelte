<!-- src/routes/convert/[format]/+page.svelte -->
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
  import { getConvertStore, clearConvertStore } from '$lib/stores/convertStore';

  const sourceFormatId: string = page.params.format!;
  
  const convertStore = getConvertStore(sourceFormatId);
  
  let sourceFormat = $state<Format | undefined>(getFormatById(sourceFormatId));
  let isLoading = $state(!isFormatsLoaded() && !sourceFormat);
  let loadError = $state<string | null>(null);
  let targetFormats = $state<Format[]>([]);

  let selectedTarget = $state<Format | null>(null);
  let files = $state<{ path: string; name: string; id: string }[]>([]);
  let convertedFiles = $state<Map<string, string>>(new Map());
  let convertingFiles = $state<Set<string>>(new Set());
  let counter = $state(0);

  onMount(() => {
    const unsubscribe = convertStore.subscribe(state => {
      selectedTarget = state.selectedTarget;
      files = state.files;
      convertedFiles = state.convertedFiles;
      convertingFiles = state.convertingFiles;
      counter = state.counter;
    });

    return () => unsubscribe();
  });

  $effect(() => {
    convertStore.update(state => ({
      ...state,
      selectedTarget,
      files,
      convertedFiles,
      convertingFiles,
      counter,
    }));
  });

  onMount(() => {
    if (sourceFormat) {
      targetFormats = getFormats().filter(f => f.id !== sourceFormatId);
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
            isLoading = false;
          } else {
            loadError = `Формат "${sourceFormatId}" не найден`;
            isLoading = false;
          }
          clearInterval(checkFormats);
        }
      }, 100);
      
      return () => clearInterval(checkFormats);
    } else {
      loadError = `Формат "${sourceFormatId}" не найден`;
      isLoading = false;
    }
  });

  function goBack() { 
    clearConvertStore(sourceFormatId);
    goto('/'); 
  }
  
  function selectTarget(format: Format) {
    if (selectedTarget?.id === format.id) {
      selectedTarget = null;
      return;
    }
    selectedTarget = format;
  }
  
  function handleFilesChange(newFiles: typeof files) { 
    files = newFiles; 
  }

  async function convertOne(index: number) {
    const file = files[index];
    if (!selectedTarget || convertingFiles.has(file.id)) return;
    convertingFiles.add(file.id);
    try {
      const result = await invoke<{ success: boolean; content: string; error: string | null }>(
        'convert_file', { path: file.path, from: sourceFormatId, to: selectedTarget.id }
      );
      if (result.success) {
        convertedFiles = new Map(convertedFiles.set(file.id, result.content));
      }
    } catch (e) { console.error(`Conversion failed: ${file.name}`, e); }
    finally { convertingFiles.delete(file.id); }
  }

  async function convertAll() { for (let i = 0; i < files.length; i++) await convertOne(i); }
  
  function clearAll() { 
    files = []; 
    convertedFiles = new Map();
  }

  async function downloadFile(fileId: string) {
    const savedPath = convertedFiles.get(fileId);
    if (!savedPath) return;
    const content = await invoke<string>('read_file_content', { path: savedPath });
    const convertedFileName = savedPath.split('/').pop() || 'file.txt';
    try {
      const filePath = await save({
        defaultPath: convertedFileName,
        title: 'Сохранить файл',
        filters: [{ name: 'Все файлы', extensions: ['*'] }]
      });
      if (filePath) await writeTextFile(filePath, content);
    } catch (e) { console.error('[Download] Failed:', e); }
  }

  function getMonacoLang(format: string): string {
    const map: Record<string, string> = {
      json: 'json', json5: 'json', yaml: 'yaml', yml: 'yaml',
      xml: 'xml', toml: 'ini', csv: 'plaintext', tsv: 'plaintext',
      ini: 'ini', properties: 'ini', markdown: 'markdown', md: 'markdown',
      html: 'html', hjson: 'json',
    };
    return map[format] ?? 'plaintext';
  }

  async function previewFileFn(fileId: string) {
    const savedPath = convertedFiles.get(fileId) ?? files.find(f => f.id === fileId)?.path;
    if (!savedPath) return;
    try {
      const raw = await invoke<string>('read_file_content', { path: savedPath });
      const lang = getMonacoLang(selectedTarget?.id ?? sourceFormatId);
      const name = files.find(f => f.id === fileId)?.name ?? 'file';
      const windowId = `preview-${Date.now()}`;
      const webview = new WebviewWindow(windowId, {
        url: `/preview?windowId=${windowId}&title=${encodeURIComponent(name)}`,
        title: name,
        width: 900, height: 700,
        resizable: true, center: true,
        maximizable: true, minimizable: true, closable: true,
        transparent: false,
        backgroundColor: { red: 30, green: 30, blue: 30, alpha: 1 },
        theme: 'dark',
        minWidth: 400, minHeight: 300
      });
      await webview.once('preview-ready', async () => {
        await webview.emit('preview-data', { content: raw, lang, title: name });
      });
    } catch (e) { console.error('Preview failed:', e); }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<ScrollContainer>
  <TooltipProvider>
    <div class="flex flex-col bg-background text-foreground min-h-screen">
      {#if isLoading}
        <div class="flex items-center justify-center flex-1">
          <LoaderCircle class="h-16 w-16 text-primary animate-spin" />
        </div>
      {:else if loadError}
        <div class="flex flex-col items-center justify-center flex-1 gap-4">
          <p class="text-red-400 text-xl">{loadError}</p>
          <button onclick={() => goto('/')} class="text-primary hover:underline text-sm">
            Вернуться на главную
          </button>
        </div>
      {:else if sourceFormat}
        <main class="flex flex-col items-center gap-10 px-8 py-20 max-w-[1700px] mx-auto w-full">
          <button onclick={goBack} class="cursor-pointer absolute top-6 left-6 flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors">
            <ArrowLeft class="h-5 w-5" />
            <span class="text-sm">Back</span>
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
            onfileschange={handleFilesChange}
            onconvertone={convertOne}
            onconvertall={convertAll}
            onclearall={clearAll}
            onpreview={previewFileFn}
            ondownload={downloadFile}
          />
        </main>
      {/if}
    </div>
  </TooltipProvider>
</ScrollContainer>