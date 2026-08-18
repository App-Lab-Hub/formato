<!-- src/lib/components/convert/TargetFormatGrid.svelte -->
<script lang="ts">
  import type { Format } from '$lib/types/format';
  import type { AvailabilityResponse } from '$lib/data/availability';
  import { m } from '$lib/paraglide/messages';
  import { FileText, Image, Music, Film, File } from 'lucide-svelte';
  import FormatCard from './FormatCard.svelte';

  let {
    formats,
    selectedTarget,
    availability,
    onselect,
    sourceFormatId,
  }: {
    formats: Format[];
    selectedTarget: Format | null;
    availability: AvailabilityResponse | null;
    onselect: (format: Format) => void;
    sourceFormatId?: string;
  } = $props();

  // Константы вынесены за пределы компонента
  const groupOrder = ['text', 'image', 'audio', 'video', 'document'] as const;
  
  const groupConfig: Record<string, { label: string; icon: typeof FileText; color: string }> = {
    text: { label: m.format_group_text(), icon: FileText, color: 'text-blue-500 dark:text-blue-400' },
    image: { label: m.format_group_image(), icon: Image, color: 'text-emerald-500 dark:text-emerald-400' },
    audio: { label: m.format_group_audio(), icon: Music, color: 'text-rose-500 dark:text-rose-400' },
    video: { label: m.format_group_video(), icon: Film, color: 'text-amber-500 dark:text-amber-400' },
    document: { label: m.format_group_document(), icon: File, color: 'text-violet-500 dark:text-violet-400' },
  };

  // Кешируем исключённые форматы
  const excludedFormats = $derived.by(() => {
    if (!availability?.exceptions || !sourceFormatId) {
      return new Set<string>();
    }
    const ex = availability.exceptions[sourceFormatId] || [];
    return new Set(ex);
  });

  // Группировка форматов
  const groupedFormats = $derived.by(() => {
    const groups: Record<string, Format[]> = {};
    for (const format of formats) {
      const type = format.formatType || 'text';
      if (!groups[type]) groups[type] = [];
      groups[type].push(format);
    }
    return groups;
  });

  // Карта статусов — вычисляется один раз для всех форматов
  const statusMap = $derived.by(() => {
    const map = new Map<string, boolean>();
    const excluded = excludedFormats;
    for (const f of formats) {
      const type = f.formatType || 'text';
      const status = availability?.[type as keyof AvailabilityResponse] as string || 'unknown';
      map.set(f.id, status === 'available' && !excluded.has(f.id));
    }
    return map;
  });
</script>

<!-- Разделитель "Конвертировать в" -->
<div class="flex items-center justify-center gap-3 w-full my-1 select-none opacity-60">
  <div class="h-px w-12 dark:bg-border light:bg-purple-400/60"></div>
  <span class="text-[10px] font-bold uppercase tracking-widest dark:text-muted-foreground light:text-purple-700/80">
    {m.convert_to()}
  </span>
  <div class="h-px w-12 dark:bg-border light:bg-purple-400/60"></div>
</div>

<div class="flex flex-col gap-8 w-full max-w-7xl mx-auto px-4">
  {#each groupOrder as groupType}
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
        
        <!-- Сетка карточек с КЛЮЧАМИ -->
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 sm:gap-5 mt-1">
          {#each groupItems as target (target.id)}
            <FormatCard
              format={target}
              isAvailable={statusMap.get(target.id) ?? false}
              isSelected={selectedTarget?.id === target.id}
              onselect={onselect}
            />
          {/each}
        </div>

      </div>
    {/if}
  {/each}
</div>