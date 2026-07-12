<!-- src/routes/files/+page.svelte -->
<script lang="ts">
  import type { PageProps } from './$types';
  import { goto } from '$app/navigation';
  import { formatFileSize } from '$lib/utils/format';
  import { FileText, Trash2, FolderOpen, Database, Clock, HardDrive, X } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import type { FileInfo } from '$lib/types/files';
  import { openPath } from "@tauri-apps/plugin-opener";
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from '$lib/utils/toast';

  let { data }: PageProps = $props();
  
  let files: FileInfo[] = $derived(data.files);
  let selectedFile = $state<FileInfo | null>(null);
  let searchQuery = $state('');
  let filterType = $state<'all' | 'converted' | 'temp'>('all');
  
  // Храним путь файла, который сейчас удаляется
  let deletingFilePath = $state<string | null>(null);
  // Храним ID файлов для анимации удаления
  let deletingFileIds = $state<Set<string>>(new Set());

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

  function goBack() {
    goto('/');
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

  // Функция удаления файла с анимацией
  async function deleteFile(file: FileInfo) {
    if (deletingFilePath === file.path) return;
    
    if (!confirm(`Удалить файл "${file.name}"?`)) return;
    
    // Добавляем файл в список удаляемых для анимации
    deletingFileIds.add(file.path);
    deletingFilePath = file.path;
    
    try {
      await invoke('delete_file', { path: file.path });
      
      // Ждем немного для анимации
      await new Promise(resolve => setTimeout(resolve, 300));
      
      // Обновляем список файлов
      await invoke<FileInfo[]>('get_files');
      goto('/files?refresh=true', { invalidateAll: true });
      
      toast.success(`Файл "${file.name}" удалён`);
      
      if (selectedFile?.path === file.path) {
        selectedFile = null;
      }
    } catch (error) {
      console.error('Failed to delete file:', error);
      toast.error(`Не удалось удалить файл: ${error}`);
    } finally {
      deletingFilePath = null;
      deletingFileIds.delete(file.path);
    }
  }
</script>

<ScrollContainer>
  <div class="min-h-screen bg-background text-foreground p-6 sm:p-8">
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
            onclick={() => filterType = 'all'}
            class={[
              'px-4 py-2 rounded-xl text-sm font-medium transition-all',
              filterType === 'all' 
                ? 'dark:bg-primary light:bg-purple-500 text-white' 
                : 'dark:bg-card/30 light:bg-purple-200/30 dark:hover:bg-card/50 light:hover:bg-purple-200/50'
            ]}
          >
            Все
          </button>
          <button
            onclick={() => filterType = 'converted'}
            class={[
              'px-4 py-2 rounded-xl text-sm font-medium transition-all',
              filterType === 'converted' 
                ? 'bg-emerald-500 text-white' 
                : 'dark:bg-card/30 light:bg-purple-200/30 dark:hover:bg-card/50 light:hover:bg-purple-200/50'
            ]}
          >
            Сконвертированные
          </button>
          <button
            onclick={() => filterType = 'temp'}
            class={[
              'px-4 py-2 rounded-xl text-sm font-medium transition-all',
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
      {#if files.length === 0}
        <div class="flex flex-col items-center justify-center py-20 gap-4">
          <FolderOpen class="h-16 w-16 dark:text-muted-foreground/30 light:text-purple-400/30" />
          <p class="dark:text-muted-foreground light:text-purple-700/70">
            {searchQuery ? 'Нет файлов по вашему запросу' : 'Нет файлов в папках'}
          </p>
        </div>
      {:else}
        <div class="flex flex-col gap-2">
          {#each filteredFiles as file (file.path)}
            {@const isDeleting = deletingFileIds.has(file.path)}
            <div
              role="button"
              tabindex="0"
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  selectedFile = file;
                }
              }}
              onclick={() => selectedFile = file}
              class="group flex items-center gap-4 rounded-xl border dark:border-border/50 light:border-purple-300/40 dark:bg-card/30 light:bg-purple-200/30 p-4 transition-all duration-300 hover:dark:bg-card/50 hover:light:bg-purple-200/60 cursor-pointer"
              class:opacity-50={isDeleting}
              class:scale-95={isDeleting}
              class:animate-pulse={isDeleting}
            >
              <div class="shrink-0 w-10 h-10 rounded-lg dark:bg-violet-500/20 light:bg-purple-300/60 flex items-center justify-center transition-all duration-300">
                <FileText class="h-5 w-5 dark:text-violet-400 light:text-purple-700 transition-all duration-300" />
              </div>
              
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium dark:text-foreground light:text-purple-800 truncate transition-all duration-300">{file.name}</span>
                  <span class={[
                    'shrink-0 text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded-md transition-all duration-300',
                    getTypeColor(file.file_type)
                  ]}>
                    {getTypeLabel(file.file_type)}
                  </span>
                </div>
                <div class="flex items-center gap-4 text-xs dark:text-muted-foreground/70 light:text-purple-700/60 transition-all duration-300">
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
                  class="cursor-pointer p-2 rounded-lg dark:hover:bg-destructive/10 light:hover:bg-destructive/10 dark:hover:text-destructive light:hover:text-destructive transition-all duration-200 opacity-0 group-hover:opacity-100 disabled:opacity-30 disabled:cursor-not-allowed"
                >
                  {#if deletingFileIds.has(file.path)}
                    <div class="h-4 w-4 border-2 border-destructive border-t-transparent rounded-full animate-spin" />
                  {:else}
                    <Trash2 class="h-4 w-4" />
                  {/if}
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

<!-- Модальное окно с информацией о файле -->
{#if selectedFile}
  <div 
    role="button"
    tabindex="0"
    onkeydown={(e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        selectedFile = null;
      }
      if (e.key === 'Escape') {
        selectedFile = null;
      }
    }}
    onclick={() => selectedFile = null}
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm cursor-pointer"
  >
    <div 
      class="max-w-lg w-full dark:bg-card light:bg-white rounded-2xl p-6 border dark:border-border/50 light:border-purple-300/40 shadow-xl"
      onclick={(e) => e.stopPropagation()}
      role="presentation"
    >
      <div class="flex items-start justify-between mb-4">
        <h3 class="text-lg font-semibold dark:text-foreground light:text-purple-800 truncate">
          {selectedFile.name}
        </h3>
        <button 
          onclick={() => selectedFile = null}
          class="p-1 rounded-lg hover:bg-muted/50 transition-colors cursor-pointer"
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
          onclick={() => selectedFile = null}
          class="px-4 py-2 rounded-lg text-sm font-medium dark:bg-card/30 light:bg-purple-200/30 hover:dark:bg-card/50 hover:light:bg-purple-200/50 transition-colors"
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
          class="px-4 py-2 rounded-lg text-sm font-medium bg-destructive text-white hover:bg-destructive/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          {#if deletingFilePath === selectedFile?.path}
            <div class="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
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