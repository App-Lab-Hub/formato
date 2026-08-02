<!-- +page.svelte -->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { getFormatById, getFormats, isFormatsLoaded } from '$lib/data/formats';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { onMount, tick } from 'svelte';
  import type { Format } from '$lib/types/format';
  import { browser } from '$app/environment';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { m } from '$lib/paraglide/messages';
  import { toast } from '$lib/utils/toast';
  import { formatFileSize, formatSize } from '$lib/utils/format';
  import { animate } from '@motionone/dom';
  import { openPath } from '@tauri-apps/plugin-opener';
  
  // Import components
  import SourceFormatHeader from '$lib/components/convert/SourceFormatHeader.svelte';
  import TargetFormatGrid from '$lib/components/convert/TargetFormatGrid.svelte';
  import FileDropZone from '$lib/components/convert/FileDropZone.svelte';
  import TextInputZone from '$lib/components/convert/TextInputZone.svelte';
  import FileList from '$lib/components/convert/FileList.svelte';
  import ModeTabs from '$lib/components/convert/ModeTabs.svelte';
  import BackButton from '$lib/components/BackButton.svelte';
  
  // Import store
  import { 
    appState,
    setConvertedFile,
    setFileHash,
    getNextId,
    getFile,
    getConvertedFile,
    getTotalFiles,
    getConvertedCount,
    getOverallProgress,
    startConversion,
    updateProgress,
    completeConversion,
    startBatch,
    getOverallMessage,
    startDeletingAll,
    stopDeletingAll,
    saveSelectedTargetId,
    clearSavedTargetId,
    addFiles,
  } from '$lib/stores/app.svelte';

  let isAddToList = $state(false);
  
  const sourceFormatId: string = page.params.format!;
  appState.sourceFormatId = sourceFormatId;
  
  let settings = $derived(page.data.settings);
  const availability = $derived(page.data.availability);
  
  let isAnimating = $state(false);
  let sourceFormat = $state<Format | undefined>(getFormatById(sourceFormatId));
  let isLoading = $state(!isFormatsLoaded() && !sourceFormat);
  let loadError = $state<string | null>(null);
  let targetFormats = $state<Format[]>([]);

  let savedTargetId = browser ? appState.selectedTargetId : null;
  let selectedTarget = $state<Format | null>(null);
  
  // Используем state из appState
  let files = $derived(appState.files);
  let convertedFiles = $derived(appState.convertedFiles);
  let fileHashes = $derived(appState.fileHashes);
  let convertingFiles = $derived(appState.isConverting);
  let isClearing = $derived(appState.isDeletingAll);
  let isConvertingAll = $derived(appState.isConvertingAll);
  let totalFiles = $derived(getTotalFiles());
  let convertedCount = $derived(getConvertedCount());
  let overallProgress = $derived(getOverallProgress());

  let inputMode = $state<'file' | 'text'>(
    availability?.enable_text_mode ? 'file' : 'file'
  );
  let containerEl: HTMLDivElement | undefined = $state();

  async function switchMode(mode: 'file' | 'text') {
    if (inputMode === mode || isAnimating) return;
    
    const container = containerEl;
    if (!container) return;
    
    isAnimating = true;
    
    try {
      const currentChild = container.firstElementChild as HTMLElement;
      
      if (currentChild) {
        await animate(currentChild, {
          opacity: 0,
        }, {
          duration: 0.3,
          easing: 'ease-in'
        }).finished;
      }
      
      inputMode = mode;
      await tick();
      
      const newChild = container.firstElementChild as HTMLElement;
      if (!newChild) return;
      
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
      clearSavedTargetId();
      return;
    }
    selectedTarget = format;
    saveSelectedTargetId(format.id);
  }

  async function addFilesHandler(filesToAdd: { path: string; name: string }[], suppressToast: boolean = false) {
    const knownHashes = new Set(appState.fileHashes.values());
    const newFiles: { path: string; name: string; id: string }[] = [];
    let duplicates = 0;
    
    for (const file of filesToAdd) {
      const hash = await invoke<string>('hash_file', { path: file.path });
      
      if (knownHashes.has(hash)) {
        duplicates++;
        continue;
      }
      
      const newId = getNextId();
      setFileHash(newId, hash);
      
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
    
    // Используем addFiles из store
    addFiles(newFiles);
    
    if (!suppressToast) {
      if (duplicates > 0) {
        toast.success(m.file_added({ count: newFiles.length }));
        toast.warning(m.file_duplicate_skipped({ count: duplicates }));
      } else {
        toast.success(m.file_added({ count: newFiles.length }));
      }
    }
    
    const startIndex = appState.files.length - newFiles.length;
    for (let i = startIndex; i < appState.files.length; i++) {
      await convertOne(i);
    }
  }

  async function convertOne(index: number, skipPreview = false) {
    const file = appState.files[index];
    if (!selectedTarget || appState.isConverting.has(file.id)) return;
    
    startConversion(file.id, file.name);
    
    try {
      const result = await invoke<{ success: boolean; content: string; extension: string | null; error: string | null }>(
        'convert_file', { 
          path: file.path, 
          from: sourceFormatId, 
          to: selectedTarget.id,
          fromType: sourceFormat?.formatType || 'text',
          toType: selectedTarget?.formatType || 'text',
          enableCache: settings?.enable_cache ?? true
        }
      );
      
      if (result.success) {
        setConvertedFile(file.id, {
          path: result.content,
          format: result.extension || selectedTarget.id
        });

        completeConversion(file.id);
        toast.success(m.convert_success({ from: file.name, to: selectedTarget.name }));

        if (!skipPreview && settings?.auto_preview) {
          previewFileFn(file.id);
        }
      } else {
        const errorMsg = result.error || m.unknown_error();
        completeConversion(file.id, errorMsg);
        toast.error(m.convert_error({ name: file.name, error: errorMsg }));
      }
    } catch (e) { 
      console.error(`Conversion failed: ${file.name}`, e);
      const errorMsg = e instanceof Error ? e.message : m.backend_connection_error();
      completeConversion(file.id, errorMsg);
      toast.error(m.convert_error({ name: file.name, error: errorMsg }));
    }
  }

  async function convertAll() { 
    if (appState.files.length === 0) {
      toast.warning(m.no_files_to_convert());
      return;
    }
    
    if (!selectedTarget) {
      toast.warning(m.select_target_format());
      return;
    }
    
    startBatch(appState.files.length);
    
    for (let i = 0; i < appState.files.length; i++) {
      await convertOne(i, true);
    }
  }

  async function downloadFile(fileId: string) {
    const converted = getConvertedFile(fileId);
    if (!converted) {
      toast.warning(m.convert_first_download());
      return;
    }
    const file = getFile(fileId);
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
    const converted = getConvertedFile(fileId);
    const file = getFile(fileId);
    const savedPath = converted?.path ?? file?.path;
    if (!savedPath) {
      toast.warning(m.file_not_found());
      return;
    }
    try {
      const actualSize = await invoke<number>('get_file_size', { path: savedPath });
      const maxSizeMB = settings?.max_preview_size ?? 5;
      const maxSizeBytes = maxSizeMB === 0 ? Infinity : maxSizeMB * 1024 * 1024;
      
      if (actualSize > maxSizeBytes) {
        toast.warning(m.preview_too_large_monaco({ 
          size: formatFileSize(actualSize),
          limit: maxSizeMB === 0 ? m.preview_unlimited() : formatSize(maxSizeMB)
        }));
        return;
      }
      
      await openPath(savedPath);
      
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

          <SourceFormatHeader format={sourceFormat} />
          <TargetFormatGrid 
            formats={targetFormats} 
            {selectedTarget} 
            availability={availability}
            onselect={selectTarget} 
          />
          
          {#if availability?.enable_text_mode}
            <div class="w-full max-w-4xl">
              <ModeTabs 
                mode={inputMode}
                onModeChange={(mode) => switchMode(mode)}
              />
            </div>
          {/if}

          <div class="w-full max-w-4xl relative overflow-hidden" bind:this={containerEl}>
            {#if inputMode === 'file'}
              <FileDropZone
                sourceFormatId={sourceFormatId}
                sourceFormatName={sourceFormat?.name ?? ''}
                sourceFormatExtensions={sourceFormat?.extensions ?? [sourceFormatId]}
                onfilesadd={addFilesHandler}
              />
            {:else if availability?.enable_text_mode}
              <TextInputZone
                {sourceFormatId}
                sourceFormatName={sourceFormat?.name ?? ''}
                onfilesadd={addFilesHandler}
                bind:isAddToList={isAddToList} 
              />
            {/if}
          </div>

          {#if isConvertingAll}
            <div class="w-full max-w-4xl px-4 py-3 bg-background/80 backdrop-blur-sm rounded-xl border border-border/30 flex items-center gap-4">
              <div class="flex-1">
                <div class="flex justify-between text-sm mb-1">
                  <span class="text-muted-foreground">{getOverallMessage()}</span>
                  <span class="text-muted-foreground">{Math.round(overallProgress * 100)}%</span>
                </div>
                <div class="w-full h-2 bg-gray-700 rounded-full overflow-hidden">
                  <div 
                    class="h-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-300"
                    style="width: {overallProgress * 100}%"
                  />
                </div>
              </div>
              <span class="text-xs text-muted-foreground/70">
                {convertedCount}/{totalFiles}
              </span>
            </div>
          {/if}

          <FileList
            {sourceFormatId}
            {selectedTarget}
            showExtensions={settings?.show_extensions ?? true}
            onconvertone={convertOne}
            onconvertall={convertAll}
            onpreview={previewFileFn}
            ondownload={downloadFile}
            settings={settings}
          />
        </main>
      </div>
  {/if}
</ScrollContainer>