<!-- src/lib/components/convert/FormatCard.svelte -->
<script lang="ts">
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';
  import { Lock, Sparkles, AlertCircle, Download, Check } from 'lucide-svelte';

  let {
    format,
    status,
    isSelected,
    onselect,
  }: {
    format: Format;
    status: string;
    isSelected: boolean;
    onselect: (format: Format) => void;
  } = $props();

  const Icon = format.icon;
  const isAvailable = status === 'available' || status === 'available_with_ai';
  const isNotAvailable = status === 'not_available';
  const isAvailableWithAI = status === 'available_with_ai';

  function getStatusInfo() {
    switch (status) {
      case 'available':
        return { 
          label: 'Доступно', 
          color: 'text-green-400 border-green-400/30 bg-green-500/10',
          icon: Check,
        };
      case 'available_with_ai':
        return { 
          label: 'Требуется AI', 
          color: 'text-yellow-400 border-yellow-400/30 bg-yellow-500/10',
          icon: Sparkles,
        };
      case 'not_available':
        return { 
          label: 'Недоступно', 
          color: 'text-red-400 border-red-400/30 bg-red-500/10',
          icon: Lock,
        };
      default:
        return { 
          label: 'Неизвестно', 
          color: 'text-gray-400 border-gray-400/30 bg-gray-500/10',
          icon: AlertCircle,
        };
    }
  }

  const statusInfo = getStatusInfo();

  function handleDownloadAI(e: MouseEvent) {
    e.stopPropagation();
    console.log('📦 Скачивание AI модели для формата:', format.name);
  }

  function handleCardClick() {
    if (isAvailable) {
      onselect(format);
    }
  }
</script>

<!-- ДЕФОЛТНАЯ КАРТОЧКА -->
{#if !isAvailableWithAI && !isNotAvailable}
  <button
    onclick={handleCardClick}
    class="group relative flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-5 w-full aspect-[4/5] transition-all duration-300
           {isSelected 
             ? 'border-primary bg-primary/5 scale-105 shadow-xl' 
             : 'dark:border-border light:border-purple-300/40 dark:bg-card light:bg-purple-200/50 dark:hover:border-primary/40 light:hover:border-purple-500/60 hover:scale-[1.02]'}
           {format.glow}"
  >


    <div class="relative rounded-2xl bg-gradient-to-br p-5 {format.color}">
      <div class="absolute inset-0 rounded-2xl bg-gradient-to-br opacity-30 blur-2xl {format.color}"></div>
      <div class="flex-shrink-0 h-11 w-11">
        <Icon class="relative w-full h-full {format.textColor}" />
      </div>
    </div>

    <span class="text-base font-bold dark:text-foreground light:text-purple-800">{format.name}</span>
    <span class="text-xs dark:text-muted-foreground light:text-purple-700/60 line-clamp-3 max-w-[160px]">
      {(m as any)[`format_desc_${format.id}`]()}
    </span>
  </button>
{/if}

<!-- КАРТОЧКА С AI - ОРАНЖЕВАЯ ТЕМА (DIV вместо BUTTON) -->
{#if isAvailableWithAI}
  <div
    onclick={handleCardClick}
    class="group relative flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-5 w-full aspect-[4/5] transition-all duration-300 cursor-pointer
           {isSelected 
             ? 'border-orange-500 bg-orange-500/10 scale-105 shadow-xl shadow-orange-500/20' 
             : 'dark:border-orange-500/30 light:border-orange-400/40 dark:bg-orange-500/10 light:bg-orange-400/10 dark:hover:border-orange-400/60 light:hover:border-orange-500/70 hover:scale-[1.02]'}
           shadow-orange-500/10"
  >


    <div class="relative rounded-2xl bg-gradient-to-br p-5 from-orange-500/40 to-amber-500/40">
      <div class="absolute inset-0 rounded-2xl bg-gradient-to-br from-orange-500/30 to-amber-500/30 opacity-30 blur-2xl"></div>
      <div class="flex-shrink-0 h-11 w-11">
        <Icon class="relative w-full h-full text-orange-400" />
      </div>
    </div>

    <span class="text-base font-bold text-orange-400">{format.name}</span>
    <span class="text-xs text-orange-400/60 line-clamp-3 max-w-[160px]">
      {(m as any)[`format_desc_${format.id}`]()}
    </span>

    <button
      onclick={handleDownloadAI}
      class="absolute bottom-3 left-1/2 -translate-x-1/2 flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[10px] font-medium bg-orange-500/20 text-orange-400 hover:bg-orange-500/30 border border-orange-400/30 transition-all"
    >
      <Download class="h-3 w-3" />
      <span>Скачать AI</span>
    </button>
  </div>
{/if}

<!-- КАРТОЧКА НЕДОСТУПНО - КРАСНАЯ ТЕМА -->
{#if isNotAvailable}
  <div
    class="group relative flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-5 w-full aspect-[4/5] transition-all duration-300
           dark:border-red-500/30 light:border-red-400/40 
           dark:bg-red-500/10 light:bg-red-400/10
           opacity-60 cursor-not-allowed"
  >

    <div class="relative rounded-2xl bg-gradient-to-br p-5 from-red-500/30 to-rose-500/30">
      <div class="absolute inset-0 rounded-2xl bg-gradient-to-br from-red-500/20 to-rose-500/20 opacity-30 blur-2xl"></div>
      <div class="flex-shrink-0 h-11 w-11">
        <Icon class="relative w-full h-full text-red-400/60" />
      </div>
    </div>

    <span class="text-base font-bold text-red-400/60">{format.name}</span>
    <span class="text-xs text-red-400/40 line-clamp-3 max-w-[160px]">
      {(m as any)[`format_desc_${format.id}`]()}
    </span>

    <div class="absolute inset-0 rounded-2xl flex items-center justify-center bg-black/20 backdrop-blur-[1px]">
      <div class="flex flex-col items-center gap-1">
        <Lock class="h-6 w-6 text-red-400/60" />
        <span class="text-[10px] text-red-400/60 font-medium">Недоступно</span>
      </div>
    </div>
  </div>
{/if}