<!-- src/routes/settings/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Sun, Moon, Monitor, Languages, Palette, Eye, Database, PlayCircle, FolderOpen, FileCheck, Shield, Archive } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { onMount } from 'svelte';
  import { getSettings, saveSettings, type AppSettings } from '$lib/data/settings';

  let settings = $state<AppSettings>(getSettings());
  let theme = $state(settings.theme);
  let language = $state(settings.language);
  let autoPreview = $state(settings.auto_preview);
  let maxPreviewSize = $state(settings.max_preview_size);
  let afterConvert = $state(settings.after_convert);
  let showExtensions = $state(settings.show_extensions);
  let enableCache = $state(settings.enable_cache);
  let enableArchive = $state(settings.enable_archive);
  let archiveFormat = $state(settings.archive_format);

  const maxPreviewSizes = [1, 10, 50, 100, 500, 1000];

  function goBack() {
    goto('/');
  }

  async function save() {
    await saveSettings({
      theme, language, auto_preview: autoPreview,
      max_preview_size: maxPreviewSize, after_convert: afterConvert,
      show_extensions: showExtensions, enable_cache: enableCache,
      enable_archive: enableArchive, archive_format: archiveFormat,
    });
  }

  onMount(() => {
    document.documentElement.style.backgroundColor = '#0a0a0f';
    return () => {
      document.documentElement.style.backgroundColor = '';
    };
  });

  $effect(() => {
    save();
  });
</script>

<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-full">
    <main class="flex flex-col items-center px-8 py-16">
      
      <div class="w-full max-w-[1700px] flex justify-start mb-8">
        <button 
          onclick={goBack} 
          class="cursor-pointer flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors"
        >
          <ArrowLeft class="h-5 w-5" />
          <span class="text-sm">На главную</span>
        </button>
      </div>

      <div class="w-full max-w-[1700px]">
        <div class="text-center mb-12">
          <h1 class="text-3xl sm:text-4xl font-bold mb-3">
            <span class="bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
              Настройки
            </span>
          </h1>
          <div class="mt-4 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-border to-transparent"></div>
          <p class="text-muted-foreground/60 text-sm mt-2">
            Настройте приложение под себя
          </p>
        </div>

        <div class="space-y-6 max-w-3xl mx-auto">
          
          <!-- Тема -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-4">
              <Palette class="h-5 w-5 text-primary" />
              <h2 class="text-lg font-semibold">Тема</h2>
            </div>
            <div class="grid grid-cols-3 gap-3">
              {#each [
                { id: 'light', icon: Sun, label: 'Светлая' },
                { id: 'dark', icon: Moon, label: 'Тёмная' },
                { id: 'system', icon: Monitor, label: 'Системная' }
              ] as opt}
                <button 
                  onclick={() => theme = opt.id}
                  class="cursor-pointer px-4 py-3 rounded-xl border transition-all flex items-center justify-center gap-2 {theme === opt.id ? 'border-primary bg-primary/10 text-primary' : 'border-border hover:border-primary/50'}"
                >
                  <opt.icon class="h-4 w-4" />
                  <span class="text-sm">{opt.label}</span>
                </button>
              {/each}
            </div>
          </div>

          <!-- Язык -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-4">
              <Languages class="h-5 w-5 text-primary" />
              <h2 class="text-lg font-semibold">Язык интерфейса</h2>
            </div>
            <div class="grid grid-cols-2 gap-3">
              {#each [
                { id: 'ru', label: '🇷🇺 Русский' },
                { id: 'en', label: '🇬🇧 English' }
              ] as opt}
                <button 
                  onclick={() => language = opt.id}
                  class="cursor-pointer px-4 py-3 rounded-xl border transition-all {language === opt.id ? 'border-primary bg-primary/10 text-primary' : 'border-border hover:border-primary/50'}"
                >
                  <span class="text-sm">{opt.label}</span>
                </button>
              {/each}
            </div>
          </div>

          <!-- Авто-превью -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <FolderOpen class="h-5 w-5 text-primary" />
                <div>
                  <h2 class="text-lg font-semibold">Авто-превью</h2>
                  <p class="text-sm text-muted-foreground">Открывать предпросмотр файла после завершения конвертации</p>
                </div>
              </div>
              <button 
                onclick={() => autoPreview = !autoPreview}
                aria-label="Авто-превью"
                class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {autoPreview ? 'bg-primary' : 'bg-muted-foreground/20'}"
              >
                <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {autoPreview ? 'left-6' : 'left-0.5'}"></span>
              </button>
            </div>
          </div>

          <!-- Архивация -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-4">
              <Archive class="h-5 w-5 text-primary" />
              <div>
                <h2 class="text-lg font-semibold">Архивировать результат</h2>
                <p class="text-sm text-muted-foreground">Упаковать сконвертированный файл в архив</p>
              </div>
            </div>
            <div class="flex items-center justify-between mb-4">
              <span class="text-sm text-muted-foreground">Включить архивацию</span>
              <button 
                onclick={() => enableArchive = !enableArchive}
                class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {enableArchive ? 'bg-primary' : 'bg-muted-foreground/20'}"
              >
                <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {enableArchive ? 'left-6' : 'left-0.5'}" />
              </button>
            </div>
            {#if enableArchive}
              <div class="grid grid-cols-3 gap-3 mt-3 pt-3 border-t border-border/50">
                {#each [
                  { id: 'zip', label: 'ZIP', desc: 'Универсальный' },
                  { id: 'tar.gz', label: 'TAR.GZ', desc: 'Linux/macOS' },
                  { id: 'tar.xz', label: 'TAR.XZ', desc: 'Макс. сжатие' }
                ] as opt}
                  <button 
                    onclick={() => archiveFormat = opt.id}
                    class="cursor-pointer px-4 py-3 rounded-xl border transition-all text-left {archiveFormat === opt.id ? 'border-primary bg-primary/10 text-primary' : 'border-border hover:border-primary/50'}"
                  >
                    <span class="text-sm font-medium">{opt.label}</span>
                    <p class="text-xs text-muted-foreground mt-1">{opt.desc}</p>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Макс. размер предпросмотра -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-4">
              <Eye class="h-5 w-5 text-primary" />
              <div>
                <h2 class="text-lg font-semibold">Лимит предпросмотра</h2>
                <p class="text-sm text-muted-foreground">Максимальный размер файла для предпросмотра</p>
              </div>
            </div>
            <div class="flex flex-wrap gap-2">
              {#each maxPreviewSizes as size}
                <button 
                  onclick={() => maxPreviewSize = size}
                  class="cursor-pointer px-4 py-2 rounded-lg border text-sm transition-all {maxPreviewSize === size ? 'border-primary bg-primary/10 text-primary' : 'border-border hover:border-primary/50'}"
                >
                  {size >= 1000 ? `${size / 1000} GB` : `${size} MB`}
                </button>
              {/each}
              <button 
                onclick={() => maxPreviewSize = 0}
                class="cursor-pointer px-4 py-2 rounded-lg border text-sm transition-all {maxPreviewSize === 0 ? 'border-primary bg-primary/10 text-primary' : 'border-border hover:border-primary/50'}"
              >
                ∞
              </button>
            </div>
          </div>
          <!-- Поведение после конвертации -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-4">
              <PlayCircle class="h-5 w-5 text-primary" />
              <div>
                <h2 class="text-lg font-semibold">После конвертации</h2>
                <p class="text-sm text-muted-foreground">Что делать после успешной конвертации</p>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              {#each [
                { id: 'stay', label: 'Остаться', desc: 'Остаться на странице конвертации' },
                { id: 'home', label: 'На главную', desc: 'Вернуться на главный экран' }
              ] as opt}
                <button 
                  onclick={() => afterConvert = opt.id}
                  class="cursor-pointer px-4 py-3 rounded-xl border transition-all text-left {afterConvert === opt.id ? 'border-primary bg-primary/10 text-primary' : 'border-border hover:border-primary/50'}"
                >
                  <span class="text-sm font-medium">{opt.label}</span>
                  <p class="text-xs text-muted-foreground mt-1">{opt.desc}</p>
                </button>
              {/each}
            </div>
          </div>

          <!-- Показывать расширения -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <FileCheck class="h-5 w-5 text-primary" />
                <div>
                  <h2 class="text-lg font-semibold">Расширения файлов</h2>
                  <p class="text-sm text-muted-foreground">Показывать расширения в списке (data.json вместо data)</p>
                </div>
              </div>
              <button 
                onclick={() => showExtensions = !showExtensions}
                class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {showExtensions ? 'bg-primary' : 'bg-muted-foreground/20'}"
              >
                <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {showExtensions ? 'left-6' : 'left-0.5'}" />
              </button>
            </div>
          </div>

          <!-- Кэширование -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <Database class="h-5 w-5 text-primary" />
                <div>
                  <h2 class="text-lg font-semibold">Кэширование конвертаций</h2>
                  <p class="text-sm text-muted-foreground">Использовать хэш для кэширования результатов</p>
                </div>
              </div>
              <button 
                onclick={() => enableCache = !enableCache}
                class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {enableCache ? 'bg-primary' : 'bg-muted-foreground/20'}"
              >
                <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {enableCache ? 'left-6' : 'left-0.5'}" />
              </button>
            </div>
          </div>

          <!-- Безопасность -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-3">
              <Shield class="h-5 w-5 text-primary" />
              <h2 class="text-lg font-semibold">Безопасность</h2>
            </div>
            <p class="text-sm text-muted-foreground mb-3">
              Все данные обрабатываются локально. Никакие данные не передаются на сервер.
            </p>
            <div class="flex items-center gap-2 text-xs">
              <span class="px-2 py-1 bg-emerald-500/10 rounded border border-emerald-500/20 text-emerald-400">● Защищено</span>
              <span class="text-muted-foreground/60">Локальная обработка</span>
            </div>
          </div>

        </div>

        <div class="text-center mt-8 text-xs text-muted-foreground/40">
          Настройки сохраняются автоматически на вашем устройстве
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>