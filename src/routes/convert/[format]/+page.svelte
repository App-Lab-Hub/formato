<!-- +page.svelte -->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { getFormatById, getFormats, isFormatsLoaded } from '$lib/data/formats';
  import { invoke } from '@tauri-apps/api/core';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import 'overlayscrollbars/overlayscrollbars.css';
  import TooltipProvider from '$lib/components/ui/tooltip/tooltip-provider.svelte';
  import SourceFormatHeader from '$lib/components/convert/SourceFormatHeader.svelte';
  import TargetFormatGrid from '$lib/components/convert/TargetFormatGrid.svelte';
  import FileDropZone from '$lib/components/convert/FileDropZone.svelte';
  import TextInputZone from '$lib/components/convert/TextInputZone.svelte';
  import FileList from '$lib/components/convert/FileList.svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { onMount, tick } from 'svelte';
  import type { Format } from '$lib/types/format';
  import { browser } from '$app/environment';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { m } from '$lib/paraglide/messages';
  import { toast } from '$lib/utils/toast';
  import { formatFileSize, formatSize } from '$lib/utils/format';
  import { Type, FolderOpen } from 'lucide-svelte';
  import { animate } from '@motionone/dom';
  import ModeTabs from '$lib/components/convert/ModeTabs.svelte';
  import BackButton from '$lib/components/BackButton.svelte';

  let isClearing = $state(false);
  let isAddToList = $state(false);
  




  const sourceFormatId: string = page.params.format!;
  let settings = $derived(page.data.settings);
  function getStorageKey(base: string): string {
    return `convert_${sourceFormatId}_${base}`;
  }
  
  let isAnimating = $state(false);
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

  let inputMode = $state<'file' | 'text'>('file');
  let containerEl: HTMLDivElement | undefined = $state();


async function switchMode(mode: 'file' | 'text') {
  if (inputMode === mode || isAnimating) return;
  
  const container = containerEl;
  if (!container) return;
  
  isAnimating = true;
  
  try {
    // Находим текущий дочерний элемент
    const currentChild = container.firstElementChild as HTMLElement;
    
    // Если есть текущий элемент — анимируем исчезновение
    if (currentChild) {
      await animate(currentChild, {
        opacity: 0,
      }, {
        duration: 0.3,
        easing: 'ease-in'
      }).finished;
    }
    
    // Меняем режим
    inputMode = mode;
    await tick();
    
    // Находим новый дочерний элемент
    const newChild = container.firstElementChild as HTMLElement;
    if (!newChild) return;
    

    
    // Анимация появления
    await new Promise(resolve => {
      requestAnimationFrame(() => {
        animate(newChild, {
          opacity: [0, 1],
        }, {
          duration: 0.3,
          easing: 'ease-out'
        }).finished.then(resolve);
      });
    });
  } finally {
    isAnimating = false;
  }
}

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

// Первый onMount - для синхронных операций
onMount(() => {
  // Загружаем форматы
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

onMount(async () => {
  // 1. Обрабатываем pending_removes_${sourceFormatId}
  const removesKey = `pending_removes_${sourceFormatId}`;
  const pendingRemoves = JSON.parse(sessionStorage.getItem(removesKey) || '[]');
  const removedPaths = new Set(pendingRemoves); // Сохраняем пути для фильтрации
  
  if (pendingRemoves.length > 0) {
    let removedCount = 0;
    const newFiles = [...files];
    
    // Удаляем файлы по пути
    for (const path of pendingRemoves) {
      const index = newFiles.findIndex(f => f.path === path);
      if (index !== -1) {
        const file = newFiles[index];
        newFiles.splice(index, 1);
        convertedFiles.delete(file.id);
        fileHashes.delete(file.id);
        removedCount++;
      }
    }
    
    if (removedCount > 0) {
      handleFilesChange(newFiles);
    }
    
    sessionStorage.removeItem(removesKey);
  }

  // 2. Обрабатываем pending_files_${sourceFormatId} с фильтрацией от removedPaths
  const filesKey = `pending_files_${sourceFormatId}`;
  const pending = sessionStorage.getItem(filesKey);
  
  if (pending) {
    try {
      const paths: string[] = JSON.parse(pending);
      
      // ✅ Фильтруем: убираем пути, которые есть в removedPaths (удаленные)
      // и пути, которые уже есть в files
      const existingPaths = new Set(files.map(f => f.path));
      const newPaths = paths.filter(p => 
        !existingPaths.has(p) && !removedPaths.has(p) // ← Фильтруем от удаленных!
      );
      
      if (newPaths.length > 0) {
        const filesToAdd = newPaths.map(path => ({
          path,
          name: path.split('/').pop() || path.split('\\').pop() || path,
        }));
        await addFiles(filesToAdd, true);
      }
      
      // Очищаем pending_files после обработки
      sessionStorage.removeItem(filesKey);
    } catch (e) {
      console.warn('Failed to process pending files:', e);
      sessionStorage.removeItem(filesKey);
    }
  }

});

  $effect(() => {
    saveFilesToStorage();
  });

  function goBack() { 
    if (isClearing) {
      toast.warning(m.clearing_in_progress());
      return;
    }
    if (isAddToList) {
      toast.warning(m.adding_in_progress());
      return;
    }

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

  function removeFile(index: number) {
    const file = files[index];
    if (!file) return;
    
    const newFiles = [...files];
    newFiles.splice(index, 1);
    handleFilesChange(newFiles);
    
    convertedFiles.delete(file.id);
    fileHashes.delete(file.id);
    
    toast.info(m.file_removed({ name: file.name }));
  }

  async function addFiles(filesToAdd: { path: string; name: string }[], suppressToast: boolean = false) {
    const knownHashes = new Set(fileHashes.values());
    const newFiles: { path: string; name: string; id: string }[] = [];
    let duplicates = 0;
    
    for (const file of filesToAdd) {
      const hash = await invoke<string>('hash_file', { path: file.path });
      
      if (knownHashes.has(hash)) {
        duplicates++;
        continue;
      }
      
      const newId = `file-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
      fileHashes.set(newId, hash);
      
      newFiles.push({
        path: file.path,
        name: file.name,
        id: newId
      });
    }
    
    if (newFiles.length === 0) {
      if (!suppressToast) {
        if (duplicates > 0) {
          toast.warning(m.file_duplicate_all({ count: duplicates }));
        } else {
          toast.warning(m.file_no_new());
        }
      }
      return;
    }
    
    const updatedFiles = [...files, ...newFiles];
    handleFilesChange(updatedFiles);
    
    if (!suppressToast) {
      if (duplicates > 0) {
        toast.success(m.file_added({ count: newFiles.length }));
        toast.warning(m.file_duplicate_skipped({ count: duplicates }));

      } else {
        toast.success(m.file_added({ count: newFiles.length }));
      }
    }
    
    for (let i = files.length; i < updatedFiles.length; i++) {
      await convertOne(i);
    }
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

        toast.success(m.convert_success({ from: file.name, to: selectedTarget.name }));

        if (!skipPreview && settings?.auto_preview) {
          previewFileFn(file.id);
        }
      } else {
        const errorMsg = result.error || m.unknown_error();
        toast.error(m.convert_error({ name: file.name, error: errorMsg }));
      }
    } catch (e) { 
      console.error(`Conversion failed: ${file.name}`, e);
      const errorMsg = e instanceof Error ? e.message : m.backend_connection_error();
      toast.error(m.convert_error({ name: file.name, error: errorMsg }));
    }
    finally { 
      convertingFiles.delete(file.id); 
    }
  }

  async function convertAll() { 
    if (files.length === 0) {
      toast.warning(m.no_files_to_convert());
      return;
    }
    
    if (!selectedTarget) {
      toast.warning(m.select_target_format());
      return;
    }
    
    for (let i = 0; i < files.length; i++) {
      await convertOne(i, true);
    }
  }
  
  function clearAll() { 
    if (files.length === 0) {
      toast.warning(m.no_files_to_clear());
      return;
    }
    
    files = []; 
    convertedFiles = new Map();
    fileHashes = new Map();
    if (browser) {
      sessionStorage.removeItem(getStorageKey('files'));
      sessionStorage.removeItem(getStorageKey('converted'));
      sessionStorage.removeItem(getStorageKey('hashes'));
      sessionStorage.removeItem(`pending_files_${sourceFormatId}`);

    }
    toast.info(m.all_files_cleared());
  }

  async function downloadFile(fileId: string) {
    const converted = convertedFiles.get(fileId);
    if (!converted) {
      toast.warning(m.convert_first_download());
      return;
    }
    const file = files.find(f => f.id === fileId);
    const baseName = file?.name.replace(/\.[^.]+$/, '') ?? 'file';
    
    try {
      const isArchive = settings?.enable_archive && settings?.archive_format;
      const ext = isArchive ? settings.archive_format : converted.format;
      const defaultName = `formato_${baseName}.${ext}`;
      
      const filePath = await save({
        defaultPath: defaultName,
        title: isArchive ? m.save_archive() : m.save_file(),
      });
      
      if (!filePath) {
        toast.info(m.save_cancelled());
        return;
      }
      
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
      
      const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'file';
      toast.success(m.file_saved({ name: fileName }));
    } catch (e) { 
      console.error('[Download] Failed:', e);
      toast.error(m.save_error());
    }
  }

  async function previewFileFn(fileId: string) {
    const converted = convertedFiles.get(fileId);
    const savedPath = converted?.path ?? files.find(f => f.id === fileId)?.path;
    if (!savedPath) {
      toast.warning(m.file_not_found());
      return;
    }
    try {
      const actualSize = await invoke<number>('get_file_size', { path: savedPath });
      const format = converted?.format ?? sourceFormatId;
      const file = files.find(f => f.id === fileId);
      const baseName = file?.name.replace(/\.[^.]+$/, '') ?? 'file';
      const title = converted ? `${baseName}.${format}` : file?.name ?? 'file';
      const windowId = `preview-${Date.now()}`;
      const maxSizeMB = settings?.max_preview_size ?? 5;
      const language = settings?.language ?? 'en';
      const theme = settings?.theme ?? 'dark';
      
      const maxSizeBytes = maxSizeMB === 0 ? Infinity : maxSizeMB * 1024 * 1024;
      if (actualSize > maxSizeBytes) {
        toast.warning(m.preview_too_large_monaco({ 
          size: formatFileSize(actualSize),
          limit: maxSizeMB === 0 ? m.preview_unlimited() : formatSize(maxSizeMB)
        }));
      }
      
      new WebviewWindow(windowId, {
        url: `/preview?path=${encodeURIComponent(savedPath)}&lang=${format}&title=${encodeURIComponent(title)}&size=${actualSize}&maxSize=${maxSizeMB}&locale=${language}&theme=${theme}&windowId=${windowId}`,
        title,
        width: 900, height: 700,
        resizable: true, center: true,
        maximizable: true, minimizable: true, closable: true,
        transparent: false,
        backgroundColor: { red: 30, green: 30, blue: 30, alpha: 1 },
        
        minWidth: 400, minHeight: 300
      });
    } catch (e) { 
      console.error('Preview failed:', e);
      toast.error(m.preview_error());
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }
</script>

<svelte:window on:keydown={handleKeydown} />
<ScrollContainer>
  {#if isLoading}
    <div class="flex items-center justify-center min-h-screen bg-background">
      <div class="h-16 w-16 text-primary animate-spin">⏳</div>
    </div>
  {:else if loadError}
    <div class="flex flex-col items-center justify-center min-h-screen bg-background gap-4">
      <p class="text-red-400 text-xl">{loadError}</p>
      <button onclick={() => goto('/')} class="text-primary hover:underline text-sm">
        {m.settings_back()}
      </button>
    </div>
  {:else if sourceFormat}

      <div class="flex flex-col bg-background text-foreground min-h-screen">
        <main class="flex flex-col items-center gap-10 px-8 py-20 max-w-[1700px] mx-auto w-full">

        <BackButton
            onClick={goBack} 
            opacity={isClearing ? 0.5 : 1}
            disabled={isClearing}
        />
          

          <!-- <button
           class:opacity-50={isClearing}
          class:cursor-not-allowed={isClearing}
           onclick={goBack} class="cursor-pointer absolute top-6 left-6 flex items-center gap-2 dark:text-muted-foreground light:text-purple-700/70 dark:hover:text-primary light:hover:text-purple-800 transition-colors">
            <span class="text-sm">← {m.settings_back()}</span>
          </button> -->

          <SourceFormatHeader format={sourceFormat} />
          <TargetFormatGrid formats={targetFormats} {selectedTarget} onselect={selectTarget} />
          
        <div class="w-full max-w-4xl">
          <ModeTabs 
            mode={inputMode}
            onModeChange={(mode) => switchMode(mode)}
          />
        </div>

        <!-- Контейнер с анимацией при смене режима -->
        <div class="w-full max-w-4xl relative overflow-hidden" bind:this={containerEl}>
          {#if inputMode === 'file'}
            <FileDropZone
              sourceFormatId={sourceFormatId}
              sourceFormatName={sourceFormat?.name ?? ''}
              sourceFormatExtensions={sourceFormat?.extensions ?? [sourceFormatId]}
              onfilesadd={addFiles}
            />
          {:else}
            <TextInputZone
              {sourceFormatId}
              sourceFormatName={sourceFormat?.name ?? ''}
              onfilesadd={addFiles}
              bind:isAddToList={isAddToList} 

            />
          {/if}
        </div>

<FileList
  {files}
  {sourceFormatId}
  {selectedTarget}
  {convertedFiles}
  {convertingFiles}
  bind:isClearing={isClearing} 
  showExtensions={settings?.show_extensions ?? true}
  onconvertone={convertOne}
  onconvertall={convertAll}
  onclearall={clearAll}
  onpreview={previewFileFn}
  ondownload={downloadFile}
  onremove={removeFile}
  settings={settings}

/> 

        </main>
      </div>
  {/if}
</ScrollContainer>