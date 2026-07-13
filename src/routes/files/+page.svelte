<!-- src/routes/files/+page.svelte -->
<script lang="ts">
  import type { PageProps } from './$types';
  import { goto, invalidateAll } from '$app/navigation';
  import { formatFileSize } from '$lib/utils/format';
  import { FileText, Trash2, FolderOpen, Database, Clock, HardDrive, X, LoaderCircle } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import type { FileInfo } from '$lib/types/files';
  import { openPath } from "@tauri-apps/plugin-opener";
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from '$lib/utils/toast';
  import { animate } from '@motionone/dom';
  import { tick } from 'svelte';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { m } from '$lib/paraglide/messages';
  import { loader } from '$lib/stores/loader.svelte';

  let { data }: PageProps = $props();
  
  let files: FileInfo[] = $derived(data.files);
  let selectedFile = $state<FileInfo | null>(null);
  let searchQuery = $state('');
  let filterType = $state<'all' | 'converted' | 'temp'>('all');
  
  // Храним путь файла, который сейчас удаляется
  let deletingFilePath = $state<string | null>(null);
  // Храним ID файлов для анимации удаления
  let deletingFileIds = $state<Set<string>>(new Set());
  
  // Реф для контейнера списка
  let listContainer: HTMLDivElement | undefined = $state();
  let isFilterAnimating = $state(false);

  // Реф для модального окна
  let modalOverlay: HTMLDivElement | undefined = $state();
  let modalContent: HTMLDivElement | undefined = $state();
  let isModalOpening = $state(false);
  let isModalClosing = $state(false);

  // Фильтрация
  let filteredFiles = $derived(
    files.filter(f => {
      const matchesSearch = f.name.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesType = filterType === 'all' || f.file_type === filterType;
      return matchesSearch && matchesType;
    })
  );

  // Статистика
  let totalFiles = $derived(files.length);
  let totalSize = $derived(files.reduce((acc, f) => acc + f.size, 0));
  let convertedCount = $derived(files.filter(f => f.file_type === 'converted').length);
  let tempCount = $derived(files.filter(f => f.file_type === 'temp').length);

  // Функция для задержки
  const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

  function goBack() {
    goto('/');
  }

  // Функция переустановки базы данных
  async function resetDatabase() {
    if (loader.isResetting) return;
    
    const confirmed = await confirm('Вы уверены, что хотите переустановить базу данных?\n\nБудут удалены:\n• Все сконвертированные файлы\n• Все временные файлы\n• Все записи в базе данных\n\nЭто действие необратимо!', {
      title: 'Переустановка базы данных',
      kind: 'warning',
    });
    if (!confirmed) return;
    
    loader.startResetting();
    
    try {
      if (selectedFile) {
        await closeModal();
      }
      
      await invoke('reset_database');
      
      // Минимальная задержка 1 секунда для лоадера
      await delay(1000);
      
      await invalidateAll();
      toast.success('База данных успешно переустановлена');
    } catch (error) {
      console.error('Failed to reset database:', error);
      toast.error('Не удалось переустановить базу данных');
    } finally {
      loader.stopResetting();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (selectedFile) {
        closeModal();
      } else {
        goBack();
      }
    }
  }

  function getTypeLabel(type: string) {
    return type === 'converted' ? 'Сконвертированные' : 'Временные';
  }

  function getTypeColor(type: string) {
    return type === 'converted' 
      ? 'text-emerald-400 bg-emerald-400/10' 
      : 'text-amber-400 bg-amber-400/10';
  }

  function formatDate(dateStr: string) {
    try {
      const date = new Date(dateStr);
      if (isNaN(date.getTime())) return 'Неизвестно';
      return date.toLocaleString('ru-RU', {
        day: '2-digit',
        month: '2-digit',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return 'Неизвестно';
    }
  }

  function getEmptyMessage() {
    if (searchQuery) return 'Нет файлов по вашему запросу';
    
    switch (filterType) {
      case 'all': return 'Нет файлов в папках';
      case 'converted': return 'Нет сконвертированных файлов';
      case 'temp': return 'Нет временных файлов';
      default: return 'Нет файлов';
    }
  }

  // Функция удаления файла с анимацией
  async function deleteFile(file: FileInfo) {
    if (deletingFilePath === file.path) return;
    
    const confirmed = await confirm(m.confirm_delete_file({ name: file.name }), {
      title: m.confirm_delete_title(),
      kind: 'warning',
    });
    if (!confirmed) return;
    
    if (selectedFile) {
      await closeModal();
      await new Promise(resolve => setTimeout(resolve, 300));
    }
    
    deletingFileIds.add(file.path);
    deletingFilePath = file.path;
    
    try {
      const el = document.querySelector(`[data-file-path="${file.path}"]`) as HTMLElement;
      if (el) {
        const animation = el.animate(
          [
            { transform: 'translateX(0px)', opacity: 1 },
            { transform: 'translateX(300px)', opacity: 0 },
          ],
          {
            duration: 300,
            easing: 'ease-in',
            fill: 'forwards'
          }
        );
        await animation.finished;
        el.style.transform = 'translateX(300px)';
        el.style.opacity = '0';
      }
      
      await invoke('delete_file', { path: file.path });
      await invalidateAll();
      toast.success(m.file_deleted({ name: file.name }));
      
      if (filterType !== 'all' && files.filter(f => f.file_type === filterType).length === 0) {
        filterType = 'all';
      }
      
      if (selectedFile?.path === file.path) {
        selectedFile = null;
      }
    } catch (error) {
      console.error('Failed to delete file:', error);
      toast.error(m.delete_error());
    } finally {
      deletingFilePath = null;
      deletingFileIds.delete(file.path);
    }
  }

  // Функция удаления всех файлов по текущему фильтру
  async function deleteAllFiltered() {
    if (loader.isDeletingAll) return;
    if (filteredFiles.length === 0) {
      toast.warning(m.no_files_to_delete());
      return;
    }
    
    const typeLabel = filterType === 'all' 
      ? m.delete_type_all() 
      : filterType === 'converted' 
        ? m.delete_type_converted() 
        : m.delete_type_temp();
    
    const confirmed = await confirm(m.confirm_delete_all({ 
      type: typeLabel, 
      count: filteredFiles.length 
    }), {
      title: m.confirm_delete_title(),
      kind: 'warning',
    });
    if (!confirmed) return;
    
    loader.startDeletingAll();
    
    try {
      if (selectedFile) {
        await closeModal();
        await new Promise(resolve => setTimeout(resolve, 300));
      }
      
      // Удаляем файлы без анимации, просто через лоадер
      let deletedCount = 0;
      for (const file of filteredFiles) {
        try {
          await invoke('delete_file', { path: file.path });
          deletedCount++;
        } catch (e) {
          console.error(`Failed to delete ${file.path}:`, e);
        }
      }
      
      // Минимальная задержка 1 секунда для лоадера
      await delay(1000);
      
      await invalidateAll();
      toast.success(m.files_deleted({ count: deletedCount }));
    } catch (error) {
      console.error('Failed to delete files:', error);
      toast.error(m.delete_error());
    } finally {
      loader.stopDeletingAll();
    }
  }

  // Функция смены фильтра с анимацией
  async function setFilter(type: 'all' | 'converted' | 'temp') {
    if (filterType === type || isFilterAnimating) return;
    
    if (!listContainer) {
      filterType = type;
      return;
    }
    
    isFilterAnimating = true;
    
    try {
      await animate(listContainer, {
        opacity: 0,
      }, {
        duration: 0.3,
        easing: 'ease-in'
      }).finished;
      
      filterType = type;
      await tick();
      
      await animate(listContainer, {
        opacity: [0, 1],
      }, {
        duration: 0.3,
        easing: 'ease-out'
      }).finished;
    } catch (error) {
      console.warn('Filter animation failed:', error);
      filterType = type;
    } finally {
      isFilterAnimating = false;
    }
  }

  // Открытие модального окна с анимацией
  async function openModal(file: FileInfo) {
    if (selectedFile?.path === file.path) return;
    
    if (isModalClosing) {
      await new Promise(resolve => {
        const checkInterval = setInterval(() => {
          if (!isModalClosing) {
            clearInterval(checkInterval);
            resolve(null);
          }
        }, 50);
      });
    }
    
    if (selectedFile) {
      await closeModal();
    }
    
    if (isModalOpening) return;
    isModalOpening = true;
    
    selectedFile = file;
    await tick();
    
    if (modalOverlay) {
      await animate(modalOverlay, {
        opacity: [0, 1],
      }, {
        duration: 0.3,
        easing: 'ease-out'
      }).finished;
    }
    
    if (modalContent) {
      await animate(modalContent, {
        scale: [0.9, 1],
        opacity: [0, 1],
      }, {
        duration: 0.3,
        easing: 'ease-out'
      }).finished;
    }
    
    isModalOpening = false;
  }

  // Закрытие модального окна с анимацией
  async function closeModal() {
    if (isModalOpening || !selectedFile) return;
    if (isModalClosing) return;
    
    isModalClosing = true;
    
    if (modalContent) {
      await animate(modalContent, {
        scale: [1, 0.9],
        opacity: [1, 0],
      }, {
        duration: 0.25,
        easing: 'ease-in'
      }).finished;
    }
    
    if (modalOverlay) {
      await animate(modalOverlay, {
        opacity: [1, 0],
      }, {
        duration: 0.25,
        easing: 'ease-in'
      }).finished;
    }
    
    selectedFile = null;
    isModalClosing = false;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<ScrollContainer>
<div class="min-h-screen flex flex-col">
  <div class="flex-1 bg-background text-foreground px-6 pt-6 sm:pt-8 sm:px-8 pb-3">
    <!-- Заголовок -->
    <div class="max-w-7xl mx-auto">
      <div class="flex items-center gap-4 mb-6">
        <button 
          onclick={goBack}
          class="cursor-pointer flex items-center gap-2 dark:text-muted-foreground light:text-purple-700/70 dark:hover:text-primary light:hover:text-purple-800 transition-colors"
        >
          <span class="text-sm">← Назад</span>
        </button>
        <h1 class="text-2xl font-bold bg-gradient-to-r from-violet-400 to-purple-400 bg-clip-text text-transparent">
          Управление файлами
        </h1>
      </div>

      <!-- Статистика -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 sm:gap-4 mb-6">
        <div class="dark:bg-card/50 light:bg-purple-200/40 rounded-xl p-4 border dark:border-border/50 light:border-purple-300/40">
          <p class="text-xs dark:text-muted-foreground light:text-purple-700/70">Всего файлов</p>
          <p class="text-2xl font-bold dark:text-foreground light:text-purple-800">{totalFiles}</p>
        </div>
        <div class="dark:bg-card/50 light:bg-purple-200/40 rounded-xl p-4 border dark:border-border/50 light:border-purple-300/40">
          <p class="text-xs dark:text-muted-foreground light:text-purple-700/70">Общий размер</p>
          <p class="text-2xl font-bold dark:text-foreground light:text-purple-800">{formatFileSize(totalSize)}</p>
        </div>
        <div class="dark:bg-card/50 light:bg-purple-200/40 rounded-xl p-4 border dark:border-border/50 light:border-purple-300/40">
          <p class="text-xs dark:text-muted-foreground light:text-purple-700/70">Сконвертированные</p>
          <p class="text-2xl font-bold text-emerald-400">{convertedCount}</p>
        </div>
        <div class="dark:bg-card/50 light:bg-purple-200/40 rounded-xl p-4 border dark:border-border/50 light:border-purple-300/40">
          <p class="text-xs dark:text-muted-foreground light:text-purple-700/70">Временные</p>
          <p class="text-2xl font-bold text-amber-400">{tempCount}</p>
        </div>
      </div>

      <!-- Фильтры -->
      <div class="flex flex-col sm:flex-row gap-3 mb-6">
        <div class="flex-1">
          <input
            type="text"
            placeholder="Поиск файлов..."
            bind:value={searchQuery}
            class="w-full px-4 py-2 rounded-xl border dark:border-border/50 light:border-purple-300/40 dark:bg-card/50 light:bg-purple-200/40 dark:text-foreground light:text-purple-800 placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-primary/50"
          />
        </div>
        <div class="flex gap-2">
          <button
            onclick={() => setFilter('all')}
            class={[
              'cursor-pointer px-4 py-2 rounded-xl text-sm font-medium transition-all',
              filterType === 'all' 
                ? 'dark:bg-primary light:bg-purple-500 text-white' 
                : 'dark:bg-card/30 light:bg-purple-200/30 dark:hover:bg-card/50 light:hover:bg-purple-200/50'
            ]}
          >
            Все
          </button>
          <button
            onclick={() => setFilter('converted')}
            class={[
              'cursor-pointer px-4 py-2 rounded-xl text-sm font-medium transition-all',
              filterType === 'converted' 
                ? 'bg-emerald-500 text-white' 
                : 'dark:bg-card/30 light:bg-purple-200/30 dark:hover:bg-card/50 light:hover:bg-purple-200/50'
            ]}
          >
            Сконвертированные
          </button>
          <button
            onclick={() => setFilter('temp')}
            class={[
              'cursor-pointer px-4 py-2 rounded-xl text-sm font-medium transition-all',
              filterType === 'temp' 
                ? 'bg-amber-500 text-white' 
                : 'dark:bg-card/30 light:bg-purple-200/30 dark:hover:bg-card/50 light:hover:bg-purple-200/50'
            ]}
          >
            Временные
          </button>
        </div>
      </div>

      <!-- Список файлов -->
      {#if loader.isDeletingAll || loader.isResetting}
        <div class="flex flex-col items-center justify-center py-20 gap-4">
          <LoaderCircle class="h-16 w-16 text-primary animate-spin" />
          <p class="dark:text-muted-foreground light:text-purple-700/70 text-lg">
            {loader.isDeletingAll ? 'Удаление файлов...' : 'Переустановка базы данных...'}
          </p>
        </div>
      {:else if filteredFiles.length === 0}
        <div class="flex flex-col items-center justify-center py-20 gap-4">
          <FolderOpen class="h-20 w-20 dark:text-muted-foreground/30 light:text-purple-400/30" />
          <p class="dark:text-muted-foreground light:text-purple-700/70 text-lg">
            {getEmptyMessage()}
          </p>
        </div>
      {:else}
        <div 
          bind:this={listContainer}
          class="flex flex-col gap-2 w-full"
        >
          {#each filteredFiles as file (file.path)}
            <div
              data-file-path={file.path}
              role="button"
              tabindex="0"
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  openModal(file);
                }
              }}
              onclick={() => openModal(file)}
              class="group flex items-center gap-4 rounded-xl border dark:border-border/50 light:border-purple-300/40 dark:bg-card/30 light:bg-purple-200/30 p-4 transition-all duration-200 hover:dark:bg-card/50 hover:light:bg-purple-200/60 cursor-pointer"
            >
              <div class="shrink-0 w-10 h-10 rounded-lg dark:bg-violet-500/20 light:bg-purple-300/60 flex items-center justify-center">
                <FileText class="h-5 w-5 dark:text-violet-400 light:text-purple-700" />
              </div>
              
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium dark:text-foreground light:text-purple-800 truncate">{file.name}</span>
                  <span class={[
                    'shrink-0 text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded-md',
                    getTypeColor(file.file_type)
                  ]}>
                    {getTypeLabel(file.file_type)}
                  </span>
                </div>
                <div class="flex items-center gap-4 text-xs dark:text-muted-foreground/70 light:text-purple-700/60">
                  <span class="flex items-center gap-1">
                    <HardDrive class="h-3 w-3" />
                    {formatFileSize(file.size)}
                  </span>
                  <span class="flex items-center gap-1">
                    <Clock class="h-3 w-3" />
                    {formatDate(file.created)}
                  </span>
                </div>
              </div>

              <div class="shrink-0 flex items-center gap-2">
                <button
                  onclick={(e) => { 
                    e.stopPropagation(); 
                    deleteFile(file);
                  }}
                  disabled={deletingFilePath === file.path}
                  class="cursor-pointer p-2 rounded-lg dark:hover:bg-destructive/10 light:hover:bg-destructive/10 dark:hover:text-destructive light:hover:text-destructive transition-colors opacity-0 group-hover:opacity-100 disabled:opacity-30 disabled:cursor-not-allowed"
                >
                  <Trash2 class="h-4 w-4" />
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Кнопка "Удалить все" -->
      {#if filteredFiles.length > 0 && !loader.isDeletingAll && !loader.isResetting}
        <div class="flex justify-end mt-5 mb-2">
          <button
            onclick={deleteAllFiltered}
            disabled={loader.isDeletingAll}
            class="cursor-pointer px-4 py-2 rounded-lg text-sm font-medium bg-destructive text-white hover:bg-destructive/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            {#if loader.isDeletingAll}
              <LoaderCircle class="h-4 w-4 animate-spin" />
              Удаление...
            {:else}
              <Trash2 class="h-4 w-4" />
              Удалить все ({filteredFiles.length})
            {/if}
          </button>
        </div>
      {/if}
    </div>
  </div>

  <footer class="mt-auto pb-5 dark:bg-background/80 light:bg-purple-200/50 backdrop-blur-xl border-t dark:border-border/50 light:border-purple-300/40">
    <div class="px-6 sm:px-8">
      <div class="max-w-7xl mx-auto">
        <div class="flex items-center justify-between mt-4">
          <div class="flex items-center gap-2">
            <Database class="h-4 w-4 dark:text-muted-foreground light:text-purple-600" />
            <span class="text-sm dark:text-muted-foreground light:text-purple-700/70">Управление базой данных</span>
          </div>
          <button
            onclick={resetDatabase}
            disabled={loader.isResetting}
            class="cursor-pointer px-4 py-2 rounded-lg text-sm font-medium dark:bg-amber-500/20 light:bg-amber-400/30 dark:text-amber-400 light:text-amber-700 dark:hover:bg-amber-500/30 light:hover:bg-amber-400/50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            {#if loader.isResetting}
              <LoaderCircle class="h-4 w-4 animate-spin" />
              Переустановка...
            {:else}
              <Database class="h-4 w-4" />
              Переустановить БД
            {/if}
          </button>
        </div>
        <p class="text-xs dark:text-muted-foreground/50 light:text-purple-600/50 mt-1">
          Удаляет все файлы и сбрасывает базу данных. Форматы будут пересозданы автоматически.
        </p>
      </div>
    </div>
  </footer>
</div>

<!-- Модальное окно с информацией о файле -->
{#if selectedFile}
  <div 
    bind:this={modalOverlay}
    role="button"
    tabindex="0"
    onkeydown={(e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        closeModal();
      }
      if (e.key === 'Escape') {
        closeModal();
      }
    }}
    onclick={() => closeModal()}
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm cursor-pointer"
    style="opacity: 0;"
  >
    <div 
      bind:this={modalContent}
      class="cursor-default max-w-lg w-full dark:bg-card light:bg-white rounded-2xl p-6 border dark:border-border/50 light:border-purple-300/40 shadow-xl"
      onclick={(e) => e.stopPropagation()}
      role="presentation"
      style="opacity: 0; transform: scale(0.9);"
    >
      <div class="flex items-start justify-between mb-4">
        <h3 class="text-lg font-semibold dark:text-foreground light:text-purple-800 truncate">
          {selectedFile.name}
        </h3>
        <button 
          onclick={() => closeModal()}
          class="cursor-pointer p-1 rounded-lg hover:bg-muted/50 transition-colors"
          aria-label="Закрыть"
        >
          <X class="h-5 w-5 dark:text-muted-foreground light:text-purple-600" />
        </button>
      </div>

      <div class="space-y-3 text-sm">
        <div class="flex justify-between py-2 border-b dark:border-border/50 light:border-purple-300/40">
          <span class="dark:text-muted-foreground light:text-purple-700/70">Тип</span>
          <span class={['px-2 py-0.5 rounded-md text-xs font-semibold uppercase', getTypeColor(selectedFile.file_type)]}>
            {getTypeLabel(selectedFile.file_type)}
          </span>
        </div>
        <div class="flex justify-between py-2 border-b dark:border-border/50 light:border-purple-300/40">
          <span class="dark:text-muted-foreground light:text-purple-700/70">Размер</span>
          <span class="dark:text-foreground light:text-purple-800">{formatFileSize(selectedFile.size)}</span>
        </div>
        <div class="flex justify-between py-2 border-b dark:border-border/50 light:border-purple-300/40">
          <span class="dark:text-muted-foreground light:text-purple-700/70">Создан</span>
          <span class="dark:text-foreground light:text-purple-800">{formatDate(selectedFile.created)}</span>
        </div>
        <div class="py-2">
          <span class="dark:text-muted-foreground light:text-purple-700/70 block mb-1 text-sm font-medium">Путь к файлу</span>
          <div 
            class="dark:bg-background/50 light:bg-purple-200/50 p-3 rounded-xl border dark:border-border/30 light:border-purple-300/30 cursor-pointer transition-colors hover:dark:bg-background/70 hover:light:bg-purple-200/70 group"
            onclick={async () => {
              if (!selectedFile) return;
              console.log("path to file -> ",selectedFile.path);
              await openPath(selectedFile.path);
            }}
            role="button"
            tabindex="0"
            onkeydown={async (e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                if (!selectedFile) return;
                console.log("path to file -> ",selectedFile.path);
                await openPath(selectedFile.path);
              }
            }}
          >
            <code class="text-xs dark:text-foreground/70 light:text-purple-800/80 font-mono break-all select-all leading-relaxed group-hover:dark:text-foreground group-hover:light:text-purple-900 transition-colors">
              {selectedFile.path}
            </code>
          </div>
        </div>
      </div>

      <!-- Кнопка удаления -->
      <div class="flex justify-end gap-2 mt-4 pt-3 border-t dark:border-border/50 light:border-purple-300/40">
        <button
          onclick={() => closeModal()}
          class="cursor-pointer px-4 py-2 rounded-lg text-sm font-medium dark:bg-card/30 light:bg-purple-200/30 hover:dark:bg-card/50 hover:light:bg-purple-200/50 transition-colors"
        >
          Отмена
        </button>
        <button
          onclick={() => {
            if (selectedFile) {
              deleteFile(selectedFile);
            }
          }}
          disabled={deletingFilePath === selectedFile?.path}
          class="cursor-pointer px-4 py-2 rounded-lg text-sm font-medium bg-destructive text-white hover:bg-destructive/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          {#if deletingFilePath === selectedFile?.path}
            <div class="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" ></div>
            Удаление...
          {:else}
            Удалить файл
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
</ScrollContainer>