<!-- src/routes/settings/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Sun, Moon, Monitor, Languages, Palette, Eye, Database, FolderOpen, FileCheck, Shield, Archive } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { onMount } from 'svelte';
  import { getSettings, saveSettings, type AppSettings } from '$lib/data/settings';
  import { formatSize } from '$lib/utils/format';
  import { m } from '$lib/paraglide/messages';

  let settings = $state<AppSettings>(getSettings());
  let theme = $state(settings.theme);
  let language = $state(settings.language);
  let autoPreview = $state(settings.auto_preview);
  let maxPreviewSize = $state(settings.max_preview_size);
  let showExtensions = $state(settings.show_extensions);
  let enableCache = $state(settings.enable_cache);
  let enableArchive = $state(settings.enable_archive);
  let archiveFormat = $state(settings.archive_format);

  const maxPreviewSizes = [0.25, 0.5, 1.0, 10.0, 50.0, 100.0, 500.0, 1024.0];

  function goBack() {
    goto('/');
  }

  async function save() {
    await saveSettings({
      theme, language, auto_preview: autoPreview,
      max_preview_size: maxPreviewSize,
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
<!-- src/routes/settings/+page.svelte -->
<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-full">
    <main class="flex flex-col items-center px-8 py-16">
      
      <div class="w-full max-w-[1700px] flex justify-start mb-8">
        <button 
          onclick={() => goto('/')} 
          class="cursor-pointer flex items-center gap-2 dark:text-muted-foreground light:text-purple-700/70 dark:hover:text-primary light:hover:text-purple-800 transition-colors"
        >
          <ArrowLeft class="h-5 w-5" />
          <span class="text-sm">{m.settings_back()}</span>
        </button>
      </div>

      <div class="w-full max-w-[1700px]">
        <div class="text-center mb-12">
          <h1 class="text-3xl sm:text-4xl font-bold mb-3">
            <span class="bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 dark:from-cyan-400 dark:via-purple-400 dark:to-pink-400 light:from-cyan-600 light:via-purple-600 light:to-pink-600 bg-clip-text text-transparent">
              {m.settings_title()}
            </span>
          </h1>
          <div class="mt-4 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-purple-400/50 to-transparent"></div>
          <p class="dark:text-muted-foreground/60 light:text-purple-800/60 text-sm mt-2">
            {m.settings_subtitle()}
          </p>
        </div>

        <div class="space-y-6 max-w-3xl mx-auto">
          
          <!-- Тема -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center gap-3 mb-4">
              <Palette class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_theme()}</h2>
            </div>
            <div class="grid grid-cols-3 gap-3">
              {#each [
                { id: 'light', icon: Sun, label: m.settings_theme_light() },
                { id: 'dark', icon: Moon, label: m.settings_theme_dark() },
                { id: 'system', icon: Monitor, label: m.settings_theme_system() }
              ] as opt}
                <button 
                  onclick={() => theme = opt.id}
                  class="cursor-pointer px-4 py-3 rounded-xl border-2 transition-all flex items-center justify-center gap-2 {theme === opt.id ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
                >
                  <opt.icon class="h-4 w-4" />
                  <span class="text-sm font-medium">{opt.label}</span>
                </button>
              {/each}
            </div>
            <p class="mt-3 text-xs dark:text-muted-foreground/60 light:text-purple-700/70">
              {theme === 'system' ? '🔄 ' + m.settings_theme_system_desc() : 
               theme === 'dark' ? '🌙 ' + m.settings_theme_dark() : '☀️ ' + m.settings_theme_light()}
            </p>
          </div>

          <!-- Язык -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center gap-3 mb-4">
              <Languages class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_language()}</h2>
            </div>
            <div class="grid grid-cols-2 gap-3">
              {#each [
                { id: 'ru', label: '🇷🇺 Русский' },
                { id: 'en', label: '🇬🇧 English' }
              ] as opt}
                <button 
                  onclick={() => language = opt.id}
                  class="cursor-pointer px-4 py-3 rounded-xl border-2 transition-all {language === opt.id ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
                >
                  <span class="text-sm font-medium">{opt.label}</span>
                </button>
              {/each}
            </div>
          </div>

          <!-- Авто-превью -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <FolderOpen class="h-5 w-5 text-purple-600 dark:text-purple-400" />
                <div>
                  <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_auto_preview()}</h2>
                  <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_auto_preview_desc()}</p>
                </div>
              </div>
              <button 
                onclick={() => autoPreview = !autoPreview}
                aria-label={m.settings_auto_preview()}
                class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {autoPreview ? 'bg-purple-600 dark:bg-purple-500' : 'dark:bg-muted-foreground/20 light:bg-purple-300/40'}"
              >
                <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {autoPreview ? 'left-6' : 'left-0.5'}"></span>
              </button>
            </div>
          </div>

          <!-- Архивация -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center gap-3 mb-4">
              <Archive class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <div>
                <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_archive()}</h2>
                <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_archive_desc()}</p>
              </div>
            </div>
            <div class="flex items-center justify-between mb-4">
              <span class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_archive_enable()}</span>
              <button 
                onclick={() => enableArchive = !enableArchive}
                class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {enableArchive ? 'bg-purple-600 dark:bg-purple-500' : 'dark:bg-muted-foreground/20 light:bg-purple-300/40'}"
              >
                <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {enableArchive ? 'left-6' : 'left-0.5'}"></span>
              </button>
            </div>
            {#if enableArchive}
              <div class="grid grid-cols-3 gap-3 mt-3 pt-3 dark:border-t border-border/50 light:border-t border-purple-300/40">
                {#each [
                  { id: 'zip', label: 'ZIP', desc: m.settings_archive_zip() },
                  { id: 'tar.gz', label: 'TAR.GZ', desc: m.settings_archive_tar_gz() },
                  { id: 'tar.xz', label: 'TAR.XZ', desc: m.settings_archive_tar_xz() }
                ] as opt}
                  <button 
                    onclick={() => archiveFormat = opt.id}
                    class="cursor-pointer px-4 py-3 rounded-xl border-2 transition-all text-left {archiveFormat === opt.id ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
                  >
                    <span class="text-sm font-medium">{opt.label}</span>
                    <p class="text-xs dark:text-muted-foreground light:text-purple-700/60 mt-1">{opt.desc}</p>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Макс. размер предпросмотра -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center gap-3 mb-4">
              <Eye class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <div>
                <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_preview_limit()}</h2>
                <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_preview_limit_desc()}</p>
              </div>
            </div>
            <div class="flex flex-wrap gap-2">
              {#each maxPreviewSizes as size}
                <button 
                  onclick={() => maxPreviewSize = size}
                  class="cursor-pointer px-4 py-2 rounded-lg border-2 text-sm transition-all {maxPreviewSize === size ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
                >
                  {formatSize(size)}
                </button>
              {/each}
              <button 
                onclick={() => maxPreviewSize = 0}
                class="cursor-pointer px-4 py-2 rounded-lg border-2 text-sm transition-all {maxPreviewSize === 0 ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
              >
                ∞
              </button>
            </div>
          </div>

          <!-- Показывать расширения -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <FileCheck class="h-5 w-5 text-purple-600 dark:text-purple-400" />
                <div>
                  <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_show_extensions()}</h2>
                  <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_show_extensions_desc()}</p>
                </div>
              </div>
              <button 
                onclick={() => showExtensions = !showExtensions}
                class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {showExtensions ? 'bg-purple-600 dark:bg-purple-500' : 'dark:bg-muted-foreground/20 light:bg-purple-300/40'}"
              >
                <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {showExtensions ? 'left-6' : 'left-0.5'}"></span>
              </button>
            </div>
          </div>

          <!-- Кэширование -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <Database class="h-5 w-5 text-purple-600 dark:text-purple-400" />
                <div>
                  <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_enable_cache()}</h2>
                  <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_enable_cache_desc()}</p>
                </div>
              </div>
              <button 
                onclick={() => enableCache = !enableCache}
                class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {enableCache ? 'bg-purple-600 dark:bg-purple-500' : 'dark:bg-muted-foreground/20 light:bg-purple-300/40'}"
              >
                <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {enableCache ? 'left-6' : 'left-0.5'}"></span>
              </button>
            </div>
          </div>

          <!-- Безопасность -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center gap-3 mb-3">
              <Shield class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_security()}</h2>
            </div>
            <p class="text-sm dark:text-muted-foreground light:text-purple-700/70 mb-3">
              {m.settings_security_desc()}
            </p>
            <div class="flex items-center gap-2 text-xs">
              <span class="px-2 py-1 dark:bg-emerald-500/10 light:bg-emerald-500/15 rounded-lg dark:border border-emerald-500/20 light:border border-emerald-500/25 dark:text-emerald-400 light:text-emerald-600 font-medium">● {m.settings_security_badge()}</span>
              <span class="dark:text-muted-foreground/60 light:text-purple-700/50">{m.settings_security_local()}</span>
            </div>
          </div>

        </div>

        <div class="text-center mt-8 text-xs dark:text-muted-foreground/40 light:text-purple-700/40">
          {m.settings_save_message()}
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>