<!-- src/lib/components/convert/TargetFormatGrid.svelte -->
<script lang="ts">
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';
  import { FileText, Image, Music, Film, File } from 'lucide-svelte';
  import FormatCard from './FormatCard.svelte';

  let {
    formats,
    selectedTarget,
    availability,
    onselect,
  }: {
    formats: Format[];
    selectedTarget: Format | null;
    availability: Record<string, string> | null;
    onselect: (format: Format) => void;
  } = $props();

  // Исправлено для Svelte 5 и TS: убрана стрелочная функция из $derived, чтобы возвращался чистый объект, а не функция
  const groupedFormats = $derived.by(() => {
    const groups: Record<string, Format[]> = {};
    for (const format of formats) {
      const type = format.formatType || 'text';
      if (!groups[type]) groups[type] = [];
      groups[type].push(format);
    }
    return groups;
  });

  const groupOrder = ['text', 'image', 'audio', 'video', 'document'];
  
  const groupConfig: Record<string, { label: string; icon: typeof FileText; color: string }> = {
    text: { label: 'Текстовые', icon: FileText, color: 'text-blue-500 dark:text-blue-400' },
    image: { label: 'Изображения', icon: Image, color: 'text-emerald-500 dark:text-emerald-400' },
    audio: { label: 'Аудио', icon: Music, color: 'text-rose-500 dark:text-rose-400' },
    video: { label: 'Видео', icon: Film, color: 'text-amber-500 dark:text-amber-400' },
    document: { label: 'Документы', icon: File, color: 'text-violet-500 dark:text-violet-400' },
  };

  function getStatusForFormat(format: Format): string {
    if (!availability) return 'unknown';
    const type = format.formatType || 'text';
    return availability[type] || 'unknown';
  }
</script>

<!-- Разделитель "Конвертировать в" -->
<div class="flex items-center justify-center gap-3 w-full my-1 select-none opacity-40">
  <div class="h-px w-12 dark:bg-border light:bg-neutral-300"></div>
  <span class="text-[10px] font-bold uppercase tracking-widest dark:text-muted-foreground light:text-neutral-500">
    {m.convert_to()}
  </span>
  <div class="h-px w-12 dark:bg-border light:bg-neutral-300"></div>
</div>

<div class="flex flex-col gap-8 w-full max-w-7xl mx-auto px-4">
  {#each groupOrder as groupType}
    <!-- Исправлено: теперь обращаемся как к объекту groupedFormats[groupType] без вызова функции () -->
    {@const groupItems = groupedFormats[groupType]}
    {@const config = groupConfig[groupType]}
    {#if groupItems && groupItems.length > 0}
      <div class="w-full flex flex-col gap-3">
        
        <!-- Заголовок группы -->
        <div class="flex items-center gap-2 select-none h-5">
          <config.icon class="h-4 w-4 flex-shrink-0 {config.color}" />
          <span class="text-xs font-bold dark:text-neutral-400 light:text-neutral-600 whitespace-nowrap">
            {config.label}
          </span>
          <div class="h-px flex-1 dark:bg-gradient-to-r dark:from-border/60 dark:to-transparent light:bg-gradient-to-r light:from-neutral-200 light:to-transparent"></div>
          <span class="text-[10px] font-mono font-medium dark:text-neutral-500 light:text-neutral-400">
            {groupItems.length}
          </span>
        </div>
        
        <!-- Сетка карточек -->
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3 sm:gap-4 mt-1">
          {#each groupItems as target}
            {@const status = getStatusForFormat(target)}
            <FormatCard
              format={target}
              {status}
              isSelected={selectedTarget?.id === target.id}
              onselect={onselect}
            />
          {/each}
        </div>

      </div>
    {/if}
  {/each}
</div>
