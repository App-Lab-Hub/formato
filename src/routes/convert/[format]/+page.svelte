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
  import { confirm } from '@tauri-apps/plugin-dialog';
  
  // Import components
  import SourceFormatHeader from '$lib/components/convert/SourceFormatHeader.svelte';
  import TargetFormatGrid from '$lib/components/convert/TargetFormatGrid.svelte';
  import FileDropZone from '$lib/components/convert/FileDropZone.svelte';
  import TextInputZone from '$lib/components/convert/TextInputZone.svelte';
  import FileList from '$lib/components/convert/FileList.svelte';
  import ModeTabs from '$lib/components/convert/ModeTabs.svelte';
  import BackButton from '$lib/components/BackButton.svelte';
  
  // Import store
  import { appState, type FileItem } from '$lib/stores/app.svelte';

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
  
  // Получаем файлы для текущего формата
  let files = $derived(appState.getFilesForFormat(sourceFormatId));
  let totalFiles = $derived(appState.getTotalFilesForFormat(sourceFormatId));

  let inputMode = $state<'file' | 'text'>(
    availability?.enable_text_mode ? 'file' : 'file'
  );
  let containerEl: HTMLDivElement | undefined = $state();

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
    
  function selectTarget(format: Format) {
    if (selectedTarget?.id === format.id) {
      selectedTarget = null;
      return;
    }
    selectedTarget = format;
  }

  // ============================================================
  // ФУНКЦИИ ДЛЯ РАБОТЫ С ФАЙЛАМИ
  // ============================================================

  async function addFilesHandler(filesToAdd: { path: string; name: string }[], suppressToast: boolean = false) {
    const newFiles: FileItem[] = [];
    
    for (const file of filesToAdd) {
      const newId = appState.getNextIdForFormat(sourceFormatId);
      
      newFiles.push({
        path: file.path,
        name: file.name,
        id: newId
      });
    }
    
    if (newFiles.length === 0) {
      if (!suppressToast) {
        toast.warning(m.file_no_new());
      }
      return;
    }
    
    // Добавляем файлы по одному с анимацией
    for (let i = 0; i < newFiles.length; i++) {
      const file = newFiles[i];
      
      // Добавляем в store для конкретного формата
      appState.addFileToFormat(sourceFormatId, file);
      
      // Ждем появления в DOM и анимируем
      await animateFileAdd(file.id);
      
      if (i < newFiles.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }
    
    if (!suppressToast) {
      toast.success(m.file_added({ count: newFiles.length }));
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
    
    await animateAllFilesRemove();
    
    appState.clearFilesForFormat(sourceFormatId);
    toast.info(m.all_files_cleared());
  }

  // Заглушки
  function convertOne(index: number) {
    console.log('🔜 convertOne:', index);
    toast.info('Конвертация временно отключена');
  }

  function convertAll() {
    console.log('🔜 convertAll');
    toast.info('Конвертация временно отключена');
  }

  function downloadFile(fileId: string) {
    console.log('🔜 downloadFile:', fileId);
    toast.info('Скачивание временно отключено');
  }

  function previewFileFn(fileId: string) {
    console.log('🔜 previewFile:', fileId);
    toast.info('Превью временно отключено');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }
</script>

<!-- Шаблон без изменений -->

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
            opacity={1}
            disabled={false}
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
          />
        </main>
      </div>
  {/if}
</ScrollContainer>