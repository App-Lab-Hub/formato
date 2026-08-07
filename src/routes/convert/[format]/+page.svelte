<!-- +page.svelte -->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { getFormatById, getFormats, isFormatsLoaded } from '$lib/data/formats';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, tick } from 'svelte';
  import type { Format } from '$lib/types/format';
  import { browser } from '$app/environment';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { m } from '$lib/paraglide/messages';
  import { toast } from '$lib/utils/toast';
  import { animate } from '@motionone/dom';
  import { confirm, save } from '@tauri-apps/plugin-dialog';
  
  // Import components
  import SourceFormatHeader from '$lib/components/convert/SourceFormatHeader.svelte';
  import TargetFormatGrid from '$lib/components/convert/TargetFormatGrid.svelte';
  import FileDropZone from '$lib/components/convert/FileDropZone.svelte';
  import TextInputZone from '$lib/components/convert/TextInputZone.svelte';
  import FileList from '$lib/components/convert/FileList.svelte';
  import ModeTabs from '$lib/components/convert/ModeTabs.svelte';
  import BackButton from '$lib/components/BackButton.svelte';
  import { AlertTriangle, Settings } from 'lucide-svelte';
  
  // Import store
  import { appState, type FileItem } from '$lib/stores/app.svelte';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { writeFile, writeTextFile } from '@tauri-apps/plugin-fs';
  import { SvelteSet } from 'svelte/reactivity';
  import { loader } from '$lib/stores/loader.svelte';

  let isAddToList = $state(false);
  
  const sourceFormatId: string = page.params.format!;
  appState.currentFormatId = sourceFormatId;
  
  let settings = $derived(page.data.settings);
  const availability = $derived(page.data.availability);
  
  let isAnimating = $state(false);
  let sourceFormat = $state<Format | undefined>(getFormatById(sourceFormatId));
  let isLoading = $state(!isFormatsLoaded() && !sourceFormat);
  let loadError = $state<string | null>(null);
  let targetFormats = $state<Format[]>([]);

  let selectedTarget = $state<Format | null>(null);
  
  // Получаем файлы для текущего формата из store
  let files = $derived(appState.getFilesForFormat(sourceFormatId));
  let totalFiles = $derived(appState.getTotalFilesForFormat(sourceFormatId));
  
  // Получаем сконвертированные файлы из store
  let convertedFiles = $derived(appState.getConvertedFilesForFormat(sourceFormatId));

  // Состояние конвертации — используем SvelteSet для реактивности
  let convertingFileIds = $state(new SvelteSet<string>());

  let inputMode = $state<'file' | 'text'>(
    availability?.enable_text_mode ? 'file' : 'file'
  );
  let containerEl: HTMLDivElement | undefined = $state();

  // ============================================================
  // AI МОДЕЛИ — УМНЫЙ БАННЕР
  // ============================================================
  
  let modelsStatus = $derived(page.data.modelsStatus);
  let appSettings = $derived(settings);
  
  // Проверяем наличие скачанных моделей синтеза
  let hasSynthesisModel = $derived(
    modelsStatus !== null && modelsStatus.has_any_synthesis === true
  );
  
  // Проверяем, выбраны ли модели синтеза для ru и en
  let synthesisConfigured = $derived(
    appSettings?.synthesis_model && 
    appSettings.synthesis_model.ru && 
    appSettings.synthesis_model.en
  );
  
  // Проверяем, скачаны ли выбранные модели синтеза (обе)
  let selectedSynthesisDownloaded = $derived(() => {
    if (!modelsStatus || !synthesisConfigured) return false;
    const ruModel = appSettings.synthesis_model.ru;
    const enModel = appSettings.synthesis_model.en;
    return modelsStatus.synthesis[ruModel]?.exists && modelsStatus.synthesis[enModel]?.exists;
  });
  
  // Проверяем, скачана ли выбранная модель распознавания
  let selectedRecognitionDownloaded = $derived(() => {
    if (!modelsStatus || !appSettings?.recognition_model) return false;
    const modelName = appSettings.recognition_model as string;
    return modelsStatus.recognition[modelName]?.exists === true;
  });
  
  // Проверяем наличие скачанных моделей распознавания
  let hasRecognitionModel = $derived(
    modelsStatus !== null && modelsStatus.has_any_recognition === true
  );
  
  // Формируем короткое сообщение
  let bannerMessage = $derived(() => {
    const missing: string[] = [];
    
    // Проверяем синтез речи (если нет скачанных моделей ИЛИ не выбраны обе модели ИЛИ выбранные модели не скачаны)
    if (!hasSynthesisModel || !synthesisConfigured || !selectedSynthesisDownloaded()) {
      missing.push(m.ai_model_synthesis());
    }
    
    // Проверяем распознавание речи
    if (!hasRecognitionModel || !selectedRecognitionDownloaded()) {
      missing.push(m.ai_model_recognition());
    }
    
    if (missing.length === 0) return '';
    
    const joined = missing.join(` ${m.ai_model_and()} `);
    return m.ai_model_banner_message({ models: joined });
  });
  
  // Показываем баннер если есть проблемы
  let showAIBanner = $derived(
    modelsStatus !== null && bannerMessage() !== ''
  );

  // Перейти в настройки
  function goToSettings() {
    goto('/settings');
  }

  // ============================================================
  // ФУНКЦИИ ДЛЯ АНИМАЦИЙ
  // ============================================================

  async function animateFileAdd(fileId: string) {
    await tick();
    const el = document.querySelector(`[data-file-id="${fileId}"]`) as HTMLElement;
    if (el) {
      await animate(el, {
        opacity: [0, 1],
        transform: ['translateX(300px)', 'translateX(0px)'],
      }, {
        duration: 0.3,
        easing: 'ease-out'
      }).finished;
    }
  }

  async function animateFileRemove(fileId: string): Promise<void> {
    const el = document.querySelector(`[data-file-id="${fileId}"]`) as HTMLElement;
    if (el) {
      await animate(el, {
        opacity: [1, 0],
        transform: ['translateX(0px)', 'translateX(300px)'],
      }, {
        duration: 0.3,
        easing: 'ease-in'
      }).finished;
    }
  }

  async function animateAllFilesRemove() {
    const items = document.querySelectorAll('[data-file-item]');
    const animations = Array.from(items).map(el =>
      (el as HTMLElement).animate(
        [
          { transform: 'translateX(0)', opacity: 1 },
          { transform: 'translateX(300px)', opacity: 0 },
        ],
        { duration: 300, easing: 'ease-in', fill: 'forwards' }
      ).finished
    );
    await Promise.all(animations);
  }

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

  // ============================================================
  // ИНИЦИАЛИЗАЦИЯ И ВОССТАНОВЛЕНИЕ СОСТОЯНИЯ
  // ============================================================

  onMount(() => {
    if (sourceFormat) {
      targetFormats = getFormats().filter(f => f.id !== sourceFormatId);
      
      // Восстанавливаем выбранный target из store
      const savedTargetId = appState.getSelectedTargetForFormat(sourceFormatId);
      if (savedTargetId && targetFormats.length > 0) {
        const found = targetFormats.find(f => f.id === savedTargetId);
        if (found) {
          selectedTarget = found;
        }
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
            
            const savedTargetId = appState.getSelectedTargetForFormat(sourceFormatId);
            if (savedTargetId && targetFormats.length > 0) {
              const found = targetFormats.find(f => f.id === savedTargetId);
              if (found) {
                selectedTarget = found;
              }
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
    if (isAddToList) {
      toast.warning(m.adding_in_progress());
      return;
    }
    goto('/');
  }
    
  // ============================================================
  // ВЫБОР TARGET ФОРМАТА С СОХРАНЕНИЕМ
  // ============================================================

  function selectTarget(format: Format) {
    if (selectedTarget?.id === format.id) {
      selectedTarget = null;
      appState.clearSelectedTargetForFormat(sourceFormatId);
      return;
    }
    selectedTarget = format;
    appState.setSelectedTargetForFormat(sourceFormatId, format.id);
  }

  // ============================================================
  // ФУНКЦИИ ДЛЯ РАБОТЫ С ФАЙЛАМИ
  // ============================================================

  async function addFilesHandler(filesToAdd: { path: string; name: string }[], suppressToast: boolean = false) {
    const existingHashes = new Set<string>();
    for (const file of appState.getFilesForFormat(sourceFormatId)) {
      try {
        const hash = await invoke<string>('hash_file', { path: file.path });
        existingHashes.add(hash);
      } catch (e) {
        console.warn('Failed to get hash for existing file:', file.path);
      }
    }

    const newFiles: FileItem[] = [];
    let duplicates = 0;
    
    for (const file of filesToAdd) {
      const hash = await invoke<string>('hash_file', { path: file.path });
      
      if (existingHashes.has(hash)) {
        duplicates++;
        continue;
      }
      
      existingHashes.add(hash);
      
      const newId = appState.getNextIdForFormat(sourceFormatId);
      
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
    
    for (let i = 0; i < newFiles.length; i++) {
      const file = newFiles[i];
      
      appState.addFileToFormat(sourceFormatId, file);
      await animateFileAdd(file.id);
      
      if (i < newFiles.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }
    
    if (!suppressToast) {
      if (duplicates > 0) {
        toast.success(m.file_added({ count: newFiles.length }));
        toast.warning(m.file_duplicate_skipped({ count: duplicates }));
      } else {
        toast.success(m.file_added({ count: newFiles.length }));
      }
    }
  }

  async function removeFileWithConfirm(index: number) {
    const file = files[index];
    if (!file) return;

    const confirmed = await confirm(m.confirm_delete_file({ name: file.name }), {
      title: m.confirm_delete_title(),
      kind: 'warning',
    });
    if (!confirmed) return;

    await animateFileRemove(file.id);
    
    appState.removeFileFromFormat(sourceFormatId, file.id);
    toast.info(m.file_removed({ name: file.name }));
  }

 // В clearAllWithConfirm используем loader
  async function clearAllWithConfirm() {
    if (files.length === 0) {
      toast.warning(m.no_files_to_clear());
      return;
    }

    const confirmed = await confirm(m.confirm_clear_all(), {
      title: m.confirm_clear_all_title(),
      kind: 'warning',
    });
    if (!confirmed) return;
    
    // Удаляем только файлы, которые НЕ конвертируются
    const filesToRemove = files.filter(f => !loader.isConverting(f.id)); // 👈 Используем loader
    
    if (filesToRemove.length === 0) {
      toast.warning(m.no_files_to_delete());
      return;
    }
    
    // Анимируем удаление только тех файлов, которые удаляем
    const items = document.querySelectorAll('[data-file-item]');
    const animations = Array.from(items)
      .filter(el => {
        const fileId = (el as HTMLElement).dataset.fileId;
        return fileId && filesToRemove.some(f => f.id === fileId);
      })
      .map(el =>
        (el as HTMLElement).animate(
          [
            { transform: 'translateX(0)', opacity: 1 },
            { transform: 'translateX(300px)', opacity: 0 },
          ],
          { duration: 300, easing: 'ease-in', fill: 'forwards' }
        ).finished
      );
    await Promise.all(animations);
    
    // Удаляем только те файлы, которые не конвертируются
    for (const file of filesToRemove) {
      appState.removeFileFromFormat(sourceFormatId, file.id);
    }
    
    toast.success(m.files_deleted({ count: filesToRemove.length }));
    
    if (filesToRemove.length < files.length) {
      toast.warning(m.files_skipped_converting());
    }
  }
  // ============================================================
  // ФУНКЦИИ ДЛЯ КОНВЕРТАЦИИ
  // ============================================================

  async function convertOne(index: number) {
    const file = files[index];
    if (!file) return;
    if (loader.isConverting(file.id)) return;
    if (!selectedTarget) {
      toast.warning(m.select_target_format());
      return;
    }

    // ✅ Сохраняем имя целевого формата в момент начала конвертации
    const targetName = selectedTarget.name;
    const targetId = selectedTarget.id;

    const startTime = Date.now();
    loader.startConverting(file.id);
    
    try {
      const result = await invoke<{ success: boolean; content: string; extension: string | null; error: string | null }>(
        'convert_file', { 
          path: file.path, 
          from: sourceFormatId, 
          to: targetId,
          fromType: sourceFormat?.formatType || 'text',
          toType: selectedTarget?.formatType || 'text',
          enableCache: settings?.enable_cache ?? true
        }
      );
      
      if (result.success) {
        appState.addConvertedFile(sourceFormatId, file.id, {
          path: result.content,
          format: result.extension || targetId
        });

        // ✅ Используем сохраненное имя, а не текущий selectedTarget
        toast.success(m.convert_success({ from: file.name, to: targetName }));
      } else {
        const errorMsg = result.error || m.unknown_error();
        toast.error(m.convert_error({ name: file.name, error: errorMsg }));
      }
    } catch (e) { 
      console.error(`Conversion failed: ${file.name}`, e);
      const errorMsg = e instanceof Error ? e.message : m.backend_connection_error();
      toast.error(m.convert_error({ name: file.name, error: errorMsg }));
    } finally { 
      const elapsed = Date.now() - startTime;
      const minDelay = 500;
      if (elapsed < minDelay) {
        await new Promise(resolve => setTimeout(resolve, minDelay - elapsed));
      }
      loader.stopConverting(file.id);
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
      await convertOne(i);
    }
  }

  // ============================================================
  // ПРЕВЬЮ
  // ============================================================

  async function previewFileFn(fileId: string) {
    // Получаем сконвертированный файл
    const converted = appState.getConvertedFile(sourceFormatId, fileId);
    if (!converted) {
      toast.warning(m.convert_first_download());
      return;
    }

    try {
      // Проверяем размер файла
      const actualSize = await invoke<number>('get_file_size', { path: converted.path });
      const maxSizeMB = settings?.max_preview_size ?? 5;
      const maxSizeBytes = maxSizeMB === 0 ? Infinity : maxSizeMB * 1024 * 1024;

      if (actualSize > maxSizeBytes) {
        toast.warning(m.preview_too_large_monaco({
          size: formatFileSize(actualSize),
          limit: maxSizeMB === 0 ? m.preview_unlimited() : formatSize(maxSizeMB)
        }));
        return;
      }

      // Открываем файл в системном приложении
      await openPath(converted.path);
      
    } catch (e) {
      console.error('Preview failed:', e);
      toast.error(m.preview_error());
    }
  }


// ============================================================
// СКАЧИВАНИЕ
// ============================================================


async function downloadFile(fileId: string) {
  const converted = appState.getConvertedFile(sourceFormatId, fileId);
  if (!converted) {
    toast.warning(m.convert_first_download());
    return;
  }

  const file = files.find(f => f.id === fileId);
  // Извлекаем оригинальное имя (до @hash@)
  let baseName = file?.name.replace(/\.[^.]+$/, '') ?? 'file';
  if (baseName.includes('@hash@')) {
    baseName = baseName.split('@hash@')[0];
  }
  
  try {
    const isArchive = settings?.enable_archive && settings?.archive_format;
    
    const ext = isArchive ? settings.archive_format : converted.format;
    const defaultName = `formato_${baseName}.${ext}`;
    
    const filePath = await save({
      defaultPath: defaultName,
      title: isArchive ? m.save_archive() : m.save_file(),
      filters: isArchive ? [
        {
          name: `${(settings?.archive_format || 'zip').toUpperCase()} Archive`,
          extensions: [settings?.archive_format || 'zip'],
        },
      ] : undefined,
    });
    
    if (!filePath) {
      toast.info(m.save_cancelled());
      return;
    }
    
    if (isArchive) {
      // ✅ Передаём имя файла внутри архива
      const nameInsideArchive = `formato_${baseName}.${converted.format}`;
      await invoke('archive_file', { 
        sourcePath: converted.path, 
        outputPath: filePath, 
        format: settings.archive_format,
        nameInArchive: nameInsideArchive,  // ← новое имя внутри архива
      });
    } else {
      const bytes = await invoke<number[]>('read_file_bytes', { path: converted.path });
      await writeFile(filePath, new Uint8Array(bytes));
    }
    
    const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'file';
    toast.success(m.file_saved({ name: fileName }));
  } catch (e) { 
    console.error('[Download] Failed:', e);
    toast.error(m.save_error());
  }
}




// ============================================================
// СКАЧИВАНИЕ ВСЕХ КАК АРХИВ
// ============================================================

async function downloadAllAsArchive() {
  if (files.length === 0) {
    toast.warning(m.no_files_to_archive());
    return;
  }

  const allConverted = files.every(f => convertedFiles.has(f.id));
  if (!allConverted) {
    toast.warning(m.convert_all_first());
    return;
  }

  try {
    // Получаем пути и оригинальные имена файлов
    const filesData: { path: string; name: string }[] = [];
    const usedNames = new Set<string>();
    
    for (const file of files) {
      const converted = convertedFiles.get(file.id);
      if (converted) {
        // Извлекаем оригинальное имя (до @hash@)
        let baseName = file.name.replace(/\.[^.]+$/, '');
        if (baseName.includes('@hash@')) {
          baseName = baseName.split('@hash@')[0];
        }
        
        let finalName = `formato_${baseName}.${converted.format}`;
        
        // Проверяем на дубликаты
        if (usedNames.has(finalName)) {
          let counter = 1;
          const nameWithoutExt = finalName.replace(/\.[^.]+$/, '');
          const ext = finalName.split('.').pop() || converted.format;
          do {
            finalName = `${nameWithoutExt}${counter}.${ext}`;
            counter++;
          } while (usedNames.has(finalName));
        }
        usedNames.add(finalName);
        
        filesData.push({
          path: converted.path,
          name: finalName,
        });
      }
    }

    if (filesData.length === 0) {
      toast.warning(m.no_converted_files());
      return;
    }

    // Формируем имя архива
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

    // Для ZIP используем archive_multiple_files_with_names
    await invoke('archive_multiple_files', {
      files: filesData,
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


  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }

  // Вспомогательные функции для форматирования размера
  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    if (bytes < 1024 * 1024 * 1024) return (bytes / 1024 / 1024).toFixed(1) + ' MB';
    return (bytes / 1024 / 1024 / 1024).toFixed(1) + ' GB';
  }

  function formatSize(mb: number): string {
    return mb + ' MB';
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
          
          {#if showAIBanner}
            <div class="w-full max-w-4xl px-4 py-3 bg-amber-500/10 border border-amber-500/30 rounded-xl flex items-center justify-between gap-4">
              <div class="flex items-center gap-3">
                <AlertTriangle class="h-5 w-5 text-amber-400 flex-shrink-0" />
                <p class="text-sm text-amber-400">
                  {bannerMessage()}
                </p>
              </div>
              <button
                onclick={goToSettings}
                class="cursor-pointer px-3 py-1.5 rounded-lg bg-amber-500/20 hover:bg-amber-500/30 text-amber-400 text-sm font-medium transition-colors flex items-center gap-2 whitespace-nowrap"
              >
                <Settings class="h-4 w-4" />
                {m.settings_go_to()}
              </button>
            </div>
          {/if}

          <BackButton
            onClick={goBack} 
            opacity={1}
            disabled={false}
          />

          <SourceFormatHeader format={sourceFormat} />
        <TargetFormatGrid 
          formats={targetFormats} 
          {selectedTarget} 
          availability={availability}
          onselect={selectTarget}
          sourceFormatId={sourceFormatId}  // 👈 Передаем ID исходного формата
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

<FileList
  {sourceFormatId}
  {selectedTarget}
  showExtensions={settings?.show_extensions ?? true}
  onconvertone={convertOne}
  onconvertall={convertAll}
  onclearall={clearAllWithConfirm}
  onpreview={previewFileFn}
  ondownload={downloadFile}
  onremove={removeFileWithConfirm}
  ondownloadallarchive={downloadAllAsArchive}
  convertedFiles={convertedFiles}
  settings={settings}
/>
        </main>
      </div>
  {/if}
</ScrollContainer>