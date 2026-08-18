<!-- src/lib/components/convert/FormatCard.svelte -->
<script lang="ts">
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';
  import { Lock } from 'lucide-svelte';

  let {
    format,
    isAvailable,
    isSelected,
    onselect,
  }: {
    format: Format;
    isAvailable: boolean;
    isSelected: boolean;
    onselect: (format: Format) => void;
  } = $props();

  function handleCardClick() {
    if (isAvailable) {
      onselect(format);
    }
  }
</script>

{#if isAvailable}
<button
  onclick={handleCardClick}
  class="cursor-pointer group relative flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-6 w-full aspect-[4/5] transition-all duration-500 overflow-hidden text-center select-none backdrop-blur-sm
         {isSelected 
           ? 'border-primary bg-primary/5 scale-[1.04]' 
           : 'dark:border-border light:border-purple-300/40 dark:bg-card/50 light:bg-purple-200/40 hover:scale-[1.01]'}
         {isSelected ? format.glow : ''} 
         {format.borderHover}"
>
  <!-- Мягкий контур при наведении -->
  <div class="absolute -inset-px bg-gradient-to-b from-current to-transparent rounded-2xl opacity-0 group-hover:opacity-10 transition-opacity duration-500 -z-10 {format.textColor}"></div>

  <!-- Центрированная иконка -->
  <div class="relative rounded-2xl bg-gradient-to-br p-6 transition-all duration-500 group-hover:scale-110 group-hover:shadow-md {format.color} {format.glow}">
    <div class="absolute inset-0 rounded-2xl bg-gradient-to-br opacity-20 blur-xl transition-opacity group-hover:opacity-40 {format.color}"></div>
    <div class="flex-shrink-0 h-14 w-14 relative z-10 flex items-center justify-center">
      <format.icon class="w-full h-full transition-transform duration-500 group-hover:rotate-12 {format.textColor}" />
    </div>
  </div>

  <!-- Блок текстов -->
  <div class="flex flex-col items-center gap-1.5 z-10 px-3 w-full mt-1">
    <span class="text-lg font-extrabold tracking-wide dark:text-foreground light:text-purple-800 transition-colors duration-300 truncate w-full">
      {format.name}
    </span>
    <span class="text-sm font-medium leading-relaxed dark:text-muted-foreground light:text-purple-700/60 line-clamp-3 max-w-[200px]">
      {(m as any)[`format_desc_${format.id}`]()}
    </span>
  </div>

  <div class="absolute bottom-0 left-0 right-0 h-1/4 pointer-events-none rounded-b-2xl bg-gradient-to-t dark:from-neutral-950/20 light:from-purple-200/40 to-transparent z-10"></div>
</button>

{:else}
<div
  class="group relative flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-6 w-full aspect-[4/5] overflow-hidden text-center select-none
         dark:border-red-950/40 light:border-red-200/60 dark:bg-neutral-950/40 light:bg-red-50/30 
         cursor-not-allowed transition-all duration-500"
>
  <div class="absolute -inset-px bg-gradient-to-b dark:from-red-500/10 light:from-red-500/5 to-transparent rounded-2xl -z-10"></div>

  <!-- Бейдж статуса -->
  <div class="absolute top-3 right-3 z-20">
    <div class="flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[9px] font-extrabold tracking-wider uppercase border border-red-500/20 bg-red-500/5 dark:text-red-400 light:text-red-600">
      <Lock class="h-2.5 w-2.5" />
      <span>{m.format_status_locked()}</span>
    </div>
  </div>

  <!-- Иконка -->
  <div class="relative rounded-2xl bg-gradient-to-br from-neutral-200/50 via-neutral-100/40 to-neutral-200/20 dark:from-neutral-900/60 dark:to-neutral-950/40 p-6 grayscale opacity-40 border dark:border-neutral-800/50 light:border-red-200/30">
    <div class="absolute inset-0 rounded-2xl bg-gradient-to-br from-red-500/10 to-transparent opacity-20 blur-xl"></div>
    <div class="flex-shrink-0 h-14 w-14 relative z-10 flex items-center justify-center">
      <format.icon class="w-full h-full dark:text-neutral-500 light:text-neutral-400" />
    </div>
  </div>

  <!-- Текст -->
  <div class="flex flex-col items-center gap-1.5 z-10 px-3 w-full mt-1
              [mask-image:linear-gradient(to_bottom,rgba(0,0,0,1)_50%,rgba(0,0,0,0.6)_75%,rgba(0,0,0,0)_100%)]">
    <span class="text-lg font-extrabold tracking-wide dark:text-neutral-500 light:text-neutral-400 truncate w-full">
      {format.name}
    </span>
    <span class="text-sm font-medium leading-relaxed dark:text-neutral-600 light:text-neutral-400/80 line-clamp-3 max-w-[200px]">
      {(m as any)[`format_desc_${format.id}`]()}
    </span>
  </div>

  <div class="absolute bottom-0 left-0 right-0 h-1/3 pointer-events-none rounded-b-2xl bg-gradient-to-t dark:from-neutral-950/80 dark:via-neutral-950/30 light:from-red-100/60 light:via-red-100/20 to-transparent z-10"></div>
  
  <div class="absolute bottom-4 left-1/2 -translate-x-1/2 z-20 flex items-center justify-center gap-1.5 px-4 py-2 rounded-xl border text-[10px] font-black tracking-widest uppercase whitespace-nowrap max-w-[calc(100%-32px)]
              dark:border-red-950 dark:bg-red-950/30 light:border-red-200 light:bg-red-100/50 dark:text-red-400/80 light:text-red-600/80">
    <Lock class="h-3.5 w-3.5 stroke-[2.5]" />
    <span>{m.format_status_unavailable()}</span>
  </div>
</div>
{/if}