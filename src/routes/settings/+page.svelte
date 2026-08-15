<!-- src/routes/settings/+page.svelte -->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { invalidateAll } from '$app/navigation';
  import { 
    Sun, Moon, Monitor, Languages, Palette, Eye, Database, 
    FolderOpen, FileCheck, Shield, Archive, ShieldCheck, 
    Mic, Speaker, CheckCircle, XCircle, 
    Download, LoaderCircle, Globe, User, UserRound,
    Cpu, Check, AlertTriangle
  } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { onMount } from 'svelte';
  import { getSettings, saveSettings, type AppSettings } from '$lib/data/settings';
  import { formatSize } from '$lib/utils/format';
  import { m } from '$lib/paraglide/messages';
  import BackButton from '$lib/components/BackButton.svelte';
  import { toast } from '$lib/utils/toast';
  import { getModelsStatus, type ModelsStatus } from '$lib/data/models';
  import { invoke } from '@tauri-apps/api/core';
  import { loader } from '$lib/stores/loader.svelte';

  // ✅ Получаем modelsStatus из layout через page.data
  let modelsStatus = $derived<ModelsStatus | null>(page.data.modelsStatus);
  let loadingModels = $derived(modelsStatus === null);

  let settings = $state<AppSettings>(getSettings());
  let theme = $state(settings.theme);
  let language = $state(settings.language);
  let autoPreview = $state(settings.auto_preview);
  let maxPreviewSize = $state(settings.max_preview_size);
  let showExtensions = $state(settings.show_extensions);
  let enableCache = $state(settings.enable_cache);
  let enableArchive = $state(settings.enable_archive);
  let archiveFormat = $state(settings.archive_format);
  let synthesisModel = $state(settings.synthesis_model);
  let recognitionModel = $state(settings.recognition_model);

  const maxPreviewSizes = [0.25, 0.5, 1.0, 10.0, 50.0, 100.0, 500.0, 1024.0];

  // Список всех моделей синтеза
  const synthesisModels = [
    { id: 'ru', label: m.synthesis_model_dmitri(), model: 'ru_RU-dmitri-medium' },
    { id: 'ru', label: m.synthesis_model_irina(), model: 'ru_RU-irina-medium' },
    { id: 'en', label: m.synthesis_model_lessac(), model: 'en_US-lessac-medium' },
    { id: 'en', label: m.synthesis_model_amy(), model: 'en_US-amy-medium' }
  ];

  // Список моделей распознавания
  const recognitionModels = [
    { id: 'ggml-tiny-q5_1.bin', label: m.recognition_model_tiny(), desc: m.recognition_model_tiny_desc() },
    { id: 'ggml-base-q5_1.bin', label: m.recognition_model_base(), desc: m.recognition_model_base_desc() },
    { id: 'ggml-small-q5_1.bin', label: m.recognition_model_small(), desc: m.recognition_model_small_desc() },
    { id: 'ggml-medium-q5_0.bin', label: m.recognition_model_medium(), desc: m.recognition_model_medium_desc() },
    { id: 'ggml-large-v3-turbo-q5_0.bin', label: m.recognition_model_large(), desc: m.recognition_model_large_desc() }
  ];

  // ✅ Обновляем статус после скачивания через invalidateAll
  async function reloadModelsStatus() {
    try {
      await invalidateAll();
      console.log('✅ Models status reloaded via invalidateAll');
    } catch (e) {
      console.error('❌ Failed to reload models status:', e);
    }
  }

  function isSynthesisModelDownloaded(modelName: string): boolean {
    if (!modelsStatus) return false;
    return modelsStatus.synthesis[modelName]?.exists || false;
  }

  function isRecognitionModelDownloaded(modelName: string): boolean {
    if (!modelsStatus) return false;
    return modelsStatus.recognition[modelName]?.exists || false;
  }

  function goBack() {
    goto('/');
  }

  async function save() {
    await saveSettings({
      theme, language, auto_preview: autoPreview,
      max_preview_size: maxPreviewSize,
      show_extensions: showExtensions, enable_cache: enableCache,
      enable_archive: enableArchive, archive_format: archiveFormat,
      synthesis_model: synthesisModel,
      recognition_model: recognitionModel,
    });
  }

  async function downloadSynthesisModel() {
    if (loader.downloadingSynthesis) return;
    
    const startTime = Date.now();
    loader.startDownloadingSynthesis();
    
    try {
      const modelsToDownload = [
        { lang: 'ru', model: synthesisModel.ru },
        { lang: 'en', model: synthesisModel.en }
      ];
      
      for (const { lang, model } of modelsToDownload) {
        toast.info(m.downloading_model({ lang: lang.toUpperCase() }));
        await invoke('download_synthesis_model', { modelName: model });
      }
      
      toast.success(m.synthesis_models_downloaded());
      await reloadModelsStatus();
    } catch (e) {
      toast.error(m.model_download_error());
      console.error(e);
    } finally {
      const elapsed = Date.now() - startTime;
      const minDelay = 500;
      if (elapsed < minDelay) {
        await new Promise(resolve => setTimeout(resolve, minDelay - elapsed));
      }
      loader.stopDownloadingSynthesis();
    }
  }

  async function downloadRecognitionModel() {
    if (loader.downloadingRecognition) return;
    
    const startTime = Date.now();
    loader.startDownloadingRecognition();
    
    try {
      await invoke('download_recognition_model', { modelName: recognitionModel });
      toast.success(m.recognition_model_downloaded());
      await reloadModelsStatus();
    } catch (e) {
      toast.error(m.model_download_error());
      console.error(e);
    } finally {
      const elapsed = Date.now() - startTime;
      const minDelay = 500;
      if (elapsed < minDelay) {
        await new Promise(resolve => setTimeout(resolve, minDelay - elapsed));
      }
      loader.stopDownloadingRecognition();
    }
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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- src/routes/settings/+page.svelte -->
<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-full">
    <main class="flex flex-col items-center px-8 py-16">
      
      <BackButton onClick={goBack} />

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
              {theme === 'system' ? m.settings_theme_system_desc() : 
               theme === 'dark' ? m.settings_theme_dark() : m.settings_theme_light()}
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
                { id: 'ru', label: '🇷🇺 ' + m.language_russian() },
                { id: 'en', label: '🇬🇧 ' + m.language_english() }
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

          <!-- Модель синтеза речи -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center gap-3 mb-4">
              <Speaker class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <div>
                <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_synthesis_model()}</h2>
                <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">
                  {m.settings_current_model()}: <span class="font-mono text-purple-600 dark:text-purple-400">
                    {synthesisModel.ru}
                  </span> (RU) / <span class="font-mono text-purple-600 dark:text-purple-400">
                    {synthesisModel.en}
                  </span> (EN)
                </p>
                {#if loadingModels}
                  <p class="text-xs text-muted-foreground mt-1">{m.checking_models()}</p>
                {:else if modelsStatus}
                  <p class="text-xs mt-1">
                    {#if modelsStatus.has_any_synthesis}
                      <span class="text-emerald-400 inline-flex items-center gap-1">
                        <Check class="h-3.5 w-3.5" />
                        {m.models_downloaded()}
                      </span>
                    {:else}
                      <span class="text-amber-400 inline-flex items-center gap-1">
                        <AlertTriangle class="h-3.5 w-3.5" />
                        {m.models_not_downloaded()}
                      </span>
                    {/if}
                  </p>
                {/if}
              </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <!-- Русские модели -->
              <div class="space-y-2">
                <p class="text-xs font-medium text-purple-600 dark:text-purple-400">
                  <Globe class="h-3 w-3 inline mr-1" /> {m.language_russian()}
                </p>
                {#each synthesisModels.filter(m => m.id === 'ru') as opt}
                  <button 
                    onclick={() => synthesisModel = { ...synthesisModel, [opt.id]: opt.model }}
                    class="cursor-pointer w-full px-4 py-2 rounded-xl border-2 transition-all text-left {synthesisModel[opt.id] === opt.model ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
                  >
                    <div class="flex items-center justify-between">
                      <div>
                        <span class="text-sm font-medium flex items-center gap-1">
                          {#if opt.label.includes('мужской') || opt.label.includes('male')}
                            <User class="h-3.5 w-3.5" />
                          {:else}
                            <UserRound class="h-3.5 w-3.5" />
                          {/if}
                          {opt.label}
                        </span>
                        <p class="text-xs dark:text-muted-foreground light:text-purple-700/60 mt-0.5 truncate">{opt.model}</p>
                      </div>
                      {#if !loadingModels}
                        {#if isSynthesisModelDownloaded(opt.model)}
                          <CheckCircle class="h-4 w-4 text-emerald-400" />
                        {:else}
                          <XCircle class="h-4 w-4 text-muted-foreground/30" />
                        {/if}
                      {/if}
                    </div>
                  </button>
                {/each}
              </div>
              <!-- Английские модели -->
              <div class="space-y-2">
                <p class="text-xs font-medium text-purple-600 dark:text-purple-400">
                  <Globe class="h-3 w-3 inline mr-1" /> {m.language_english()}
                </p>
                {#each synthesisModels.filter(m => m.id === 'en') as opt}
                  <button 
                    onclick={() => synthesisModel = { ...synthesisModel, [opt.id]: opt.model }}
                    class="cursor-pointer w-full px-4 py-2 rounded-xl border-2 transition-all text-left {synthesisModel[opt.id] === opt.model ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
                  >
                    <div class="flex items-center justify-between">
                      <div>
                        <span class="text-sm font-medium flex items-center gap-1">
                          {#if opt.label.includes('мужской') || opt.label.includes('male')}
                            <User class="h-3.5 w-3.5" />
                          {:else}
                            <UserRound class="h-3.5 w-3.5" />
                          {/if}
                          {opt.label}
                        </span>
                        <p class="text-xs dark:text-muted-foreground light:text-purple-700/60 mt-0.5 truncate">{opt.model}</p>
                      </div>
                      {#if !loadingModels}
                        {#if isSynthesisModelDownloaded(opt.model)}
                          <CheckCircle class="h-4 w-4 text-emerald-400" />
                        {:else}
                          <XCircle class="h-4 w-4 text-muted-foreground/30" />
                        {/if}
                      {/if}
                    </div>
                  </button>
                {/each}
              </div>
            </div>
            <button 
              onclick={downloadSynthesisModel}
              disabled={loader.downloadingSynthesis}
              class="cursor-pointer mt-4 px-4 py-2 rounded-lg border-2 dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60 text-sm font-medium transition-all hover:shadow-md flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {#if loader.downloadingSynthesis}
                <LoaderCircle class="h-4 w-4 animate-spin" />
                {m.downloading()}
              {:else}
                <Download class="h-4 w-4" />
                {m.download_selected_models()}
              {/if}
            </button>
          </div>

          <!-- Модель распознавания речи -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
            <div class="flex items-center gap-3 mb-4">
              <Mic class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <div>
                <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_recognition_model()}</h2>
                <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">
                  {m.settings_current_model()}: <span class="font-mono text-purple-600 dark:text-purple-400">{recognitionModel}</span>
                </p>
                {#if loadingModels}
                  <p class="text-xs text-muted-foreground mt-1">{m.checking_models()}</p>
                {:else if modelsStatus}
                  <p class="text-xs mt-1">
                    {#if modelsStatus.has_any_recognition}
                      <span class="text-emerald-400 inline-flex items-center gap-1">
                        <Check class="h-3.5 w-3.5" />
                        {m.models_downloaded()}
                      </span>
                    {:else}
                      <span class="text-amber-400 inline-flex items-center gap-1">
                        <AlertTriangle class="h-3.5 w-3.5" />
                        {m.models_not_downloaded()}
                      </span>
                    {/if}
                  </p>
                {/if}
              </div>
            </div>
            <div class="grid grid-cols-3 gap-3">
              {#each recognitionModels as opt}
                <button 
                  onclick={() => recognitionModel = opt.id}
                  class="cursor-pointer px-4 py-3 rounded-xl border-2 transition-all text-left {recognitionModel === opt.id ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
                >
                  <div class="flex items-center justify-between">
                    <div>
                      <span class="text-sm font-medium flex items-center gap-1">
                        <Cpu class="h-3.5 w-3.5" />
                        {opt.label}
                      </span>
                      <p class="text-xs dark:text-muted-foreground light:text-purple-700/60 mt-1">{opt.desc}</p>
                    </div>
                    {#if !loadingModels}
                      {#if isRecognitionModelDownloaded(opt.id)}
                        <CheckCircle class="h-4 w-4 text-emerald-400" />
                      {:else}
                        <XCircle class="h-4 w-4 text-muted-foreground/30" />
                      {/if}
                    {/if}
                  </div>
                </button>
              {/each}
            </div>
            <button 
              onclick={downloadRecognitionModel}
              disabled={loader.downloadingRecognition}
              class="cursor-pointer mt-4 px-4 py-2 rounded-lg border-2 dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60 text-sm font-medium hover:shadow-md flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {#if loader.downloadingRecognition}
                <LoaderCircle class="h-4 w-4 animate-spin" />
                {m.downloading()}
              {:else}
                <Download class="h-4 w-4" />
                {m.download_selected_model()}
              {/if}
            </button>
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
            <Archive class="h-5 w-5 text-purple-600 dark:text-purple-400" aria-hidden="true" />
            <div>
              <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_archive()}</h2>
              <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_archive_desc()}</p>
            </div>
          </div>
          
          <div class="flex items-center justify-between mb-4">
            <span id="archive-toggle-label" class="text-sm dark:text-muted-foreground light:text-purple-700/70">
              {m.settings_archive_enable()}
            </span>
            <button 
              onclick={() => enableArchive = !enableArchive}
              class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {enableArchive ? 'bg-purple-600 dark:bg-purple-500' : 'dark:bg-muted-foreground/20 light:bg-purple-300/40'}"
              role="switch"
              aria-checked={enableArchive}
              aria-labelledby="archive-toggle-label"
              type="button"
            >
              <span class="sr-only">
                {enableArchive ? m.settings_archive_disable() : m.settings_archive_enable()}
              </span>
              <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {enableArchive ? 'left-6' : 'left-0.5'}"></span>
            </button>
          </div>
          
          {#if enableArchive}
            <div class="grid grid-cols-3 gap-3 mt-3 pt-3 dark:border-t border-border/50 light:border-t border-purple-300/40" role="radiogroup" aria-label={m.settings_archive_format_selection()}>
              {#each [
                { id: 'zip', label: 'ZIP', desc: m.settings_archive_zip() },
                { id: 'tar.gz', label: 'TAR.GZ', desc: m.settings_archive_tar_gz() },
                { id: 'tar.xz', label: 'TAR.XZ', desc: m.settings_archive_tar_xz() }
              ] as opt}
                <button 
                  onclick={() => archiveFormat = opt.id}
                  class="cursor-pointer px-4 py-3 rounded-xl border-2 transition-all text-left {archiveFormat === opt.id ? 'border-purple-600 bg-purple-300/50 text-purple-800 dark:border-purple-400 dark:bg-purple-500/20 dark:text-purple-300' : 'dark:border-border light:border-purple-300/40 dark:hover:border-purple-400/50 light:hover:border-purple-500/60 dark:bg-transparent light:bg-purple-100/40 dark:hover:bg-transparent light:hover:bg-purple-200/60'}"
                  role="radio"
                  aria-checked={archiveFormat === opt.id}
                  aria-label={m.select_archive_format({ format: opt.label })}
                  type="button"
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
              <FileCheck class="h-5 w-5 text-purple-600 dark:text-purple-400" aria-hidden="true" />
              <div>
                <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_show_extensions()}</h2>
                <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_show_extensions_desc()}</p>
              </div>
            </div>
            <button 
              onclick={() => showExtensions = !showExtensions}
              class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {showExtensions ? 'bg-purple-600 dark:bg-purple-500' : 'dark:bg-muted-foreground/20 light:bg-purple-300/40'}"
              role="switch"
              aria-checked={showExtensions}
              aria-label={showExtensions ? m.settings_show_extensions_hide() : m.settings_show_extensions_show()}
              type="button"
            >
              <span class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all {showExtensions ? 'left-6' : 'left-0.5'}"></span>
            </button>
          </div>
        </div>

        <!-- Кэширование -->
        <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <Database class="h-5 w-5 text-purple-600 dark:text-purple-400" aria-hidden="true" />
              <div>
                <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.settings_enable_cache()}</h2>
                <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.settings_enable_cache_desc()}</p>
              </div>
            </div>
            <button 
              onclick={() => enableCache = !enableCache}
              class="cursor-pointer relative w-12 h-6 rounded-full transition-colors {enableCache ? 'bg-purple-600 dark:bg-purple-500' : 'dark:bg-muted-foreground/20 light:bg-purple-300/40'}"
              role="switch"
              aria-checked={enableCache}
              aria-label={enableCache ? m.settings_enable_cache_disable() : m.settings_enable_cache_enable()}
              type="button"
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
              <span class="inline-flex items-center gap-1.5 px-2 py-1 dark:bg-emerald-500/10 light:bg-emerald-500/15 rounded-lg dark:border border-emerald-500/20 light:border border-emerald-500/25 dark:text-emerald-400 light:text-emerald-600 font-medium">
                <ShieldCheck class="h-3.5 w-3.5" />
                {m.settings_security_badge()}
              </span>
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