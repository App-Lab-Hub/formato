<script lang="ts">
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';
  import { FileText, Image, Music, Film, File } from 'lucide-svelte';

  let {
    formats,
    selectedTarget,
    onselect,
  }: {
    formats: Format[];
    selectedTarget: Format | null;
    onselect: (format: Format) => void;
  } = $props();

  // Группируем форматы по formatType
  const groupedFormats = $derived(() => {
    const groups: Record<string, Format[]> = {};
    
    for (const format of formats) {
      const type = format.formatType || 'text';
      if (!groups[type]) {
        groups[type] = [];
      }
      groups[type].push(format);
    }
    
    return groups;
  });

  // Порядок групп для отображения
  const groupOrder = ['text', 'image', 'audio', 'video', 'document'];
  
  // Названия и иконки групп
  const groupConfig: Record<string, { label: string; icon: typeof FileText; color: string }> = {
    text: {
      label: 'Текстовые',
      icon: FileText,
      color: 'text-blue-400',
    },
    image: {
      label: 'Изображения',
      icon: Image,
      color: 'text-green-400',
    },
    audio: {
      label: 'Аудио',
      icon: Music,
      color: 'text-rose-400',
    },
    video: {
      label: 'Видео',
      icon: Film,
      color: 'text-red-400',
    },
    document: {
      label: 'Документы',
      icon: File,
      color: 'text-purple-400',
    },
  };
</script>

<div class="flex items-center gap-4">
  <div class="h-px w-20 dark:bg-border light:bg-purple-300/50"></div>
  <span class="text-xs dark:text-muted-foreground/50 light:text-purple-700/60 uppercase tracking-widest">{m.convert_to()}</span>
  <div class="h-px w-20 dark:bg-border light:bg-purple-300/50"></div>
</div>

{#each groupOrder as groupType}
  {@const groupItems = groupedFormats()[groupType]}
  {@const config = groupConfig[groupType]}
  {#if groupItems && groupItems.length > 0}
    <div class="w-full max-w-7xl mx-auto">
      <!-- Заголовок группы -->
      <div class="flex items-center gap-3 mb-4 px-1">
        <div class="flex items-center gap-2">
          <svelte:component this={config.icon} class="h-4 w-4 {config.color}" />
          <span class="text-sm font-medium dark:text-muted-foreground light:text-purple-700/70">
            {config.label}
          </span>
        </div>
        <div class="h-px flex-1 dark:bg-gradient-to-r dark:from-border/50 dark:to-transparent light:bg-gradient-to-r light:from-purple-300/30 light:to-transparent"></div>
        <span class="flex items-center justify-center min-w-[20px] h-5 px-1.5 rounded-full text-[10px] font-mono font-medium dark:bg-muted/30 light:bg-purple-200/50 dark:text-muted-foreground/60 light:text-purple-700/50">
          {groupItems.length}
        </span>
      </div>
      
      <!-- Сетка форматов в группе -->
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-5">
        {#each groupItems as target}
          {@const Icon = target.icon}
          <button
            onclick={() => onselect(target)}
            class="cursor-pointer group flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-5 w-full aspect-[4/5] transition-all duration-300
                   {selectedTarget?.id === target.id ? 'border-primary bg-primary/5 scale-105 shadow-xl' : 'dark:border-border light:border-purple-300/40 dark:bg-card light:bg-purple-200/50 dark:hover:border-primary/40 light:hover:border-purple-500/60 hover:scale-[1.02]'}
                   {target.glow}"
          >
            <div class="relative rounded-2xl bg-gradient-to-br p-5 {target.color}">
              <div class="absolute inset-0 rounded-2xl bg-gradient-to-br opacity-30 blur-2xl {target.color}"></div>
              <div class="flex-shrink-0 h-11 w-11">
                <Icon class="relative w-full h-full {target.textColor}" />
              </div>
            </div>
            <span class="text-base font-bold dark:text-foreground light:text-purple-800">{target.name}</span>
            <span class="text-xs dark:text-muted-foreground light:text-purple-700/60 line-clamp-3 max-w-[160px]">
              {(m as any)[`format_desc_${target.id}`]()}
            </span>
          </button>
        {/each}
      </div>
    </div>
  {/if}
{/each}