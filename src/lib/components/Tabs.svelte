<!-- src/lib/components/Tabs.svelte -->
<script lang="ts">
  import { FolderOpen, Database, Clock } from 'lucide-svelte';
  import { m } from '$lib/paraglide/messages';
  import Tooltip from '$lib/components/ui/tooltip/tooltip.svelte';
  import TooltipTrigger from '$lib/components/ui/tooltip/tooltip-trigger.svelte';
  import TooltipContent from '$lib/components/ui/tooltip/tooltip-content.svelte';

  let { 
    filterType = 'all',
    onFilterChange
  }: {
    filterType?: 'all' | 'converted' | 'temp';
    onFilterChange?: (type: 'all' | 'converted' | 'temp') => void;
  } = $props();

  function setFilter(type: 'all' | 'converted' | 'temp') {
    if (type === filterType) return;
    if (onFilterChange) {
      onFilterChange(type);
    }
  }
</script>

<div class="flex items-center w-full sm:w-auto">
  <div 
    data-active={filterType}
    class="tabs-container relative flex items-center w-full sm:w-fit dark:bg-card/30 light:bg-purple-200/30 p-1 rounded-xl border dark:border-border/50 light:border-purple-300/40 h-10 min-w-[200px]"
  >
    <!-- Плавающая плашка -->
    <div class="tab-indicator absolute top-1 bottom-1 left-1 rounded-lg z-10 shadow-lg shadow-primary/20"
      style="
        width: calc(33.333% - 6px);
        background: {filterType === 'all' 
          ? 'linear-gradient(135deg, #8b5cf6, #6d28d9)' 
          : filterType === 'converted' 
            ? 'linear-gradient(135deg, #10b981, #059669)'
            : 'linear-gradient(135deg, #f59e0b, #d97706)'};
      "
    ></div>

    <!-- Все файлы -->
    <Tooltip>
      <TooltipTrigger>
        <button
          type="button"
          onclick={() => setFilter('all')}
          class="cursor-pointer relative z-20 px-2 sm:px-4 py-1.5 text-[11px] sm:text-sm font-medium transition-colors duration-200 text-center rounded-lg flex items-center justify-center gap-1 whitespace-nowrap w-24"
          class:text-white={filterType === 'all'}
          class:text-muted-foreground={filterType !== 'all'}
          class:hover:text-foreground={filterType !== 'all'}
        >
          <FolderOpen class="h-3.5 w-3.5 flex-shrink-0" />
          <span class="truncate">{m.files_filter_all()}</span>
        </button>
      </TooltipTrigger>
      <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
        <p>{m.files_filter_all_tooltip()}</p>
      </TooltipContent>
    </Tooltip>

    <!-- Сконвертированные -->
    <Tooltip>
      <TooltipTrigger>
        <button
          type="button"
          onclick={() => setFilter('converted')}
          class="cursor-pointer relative z-20 px-2 sm:px-4 py-1.5 text-[11px] sm:text-sm font-medium transition-colors duration-200 text-center rounded-lg flex items-center justify-center gap-1 whitespace-nowrap w-24"
          class:text-white={filterType === 'converted'}
          class:text-muted-foreground={filterType !== 'converted'}
          class:hover:text-foreground={filterType !== 'converted'}
        >
          <Database class="h-3.5 w-3.5 flex-shrink-0" />
          <span class="truncate">{m.files_filter_converted()}</span>
        </button>
      </TooltipTrigger>
      <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
        <p>{m.files_filter_converted_tooltip()}</p>
      </TooltipContent>
    </Tooltip>

    <!-- Временные -->
    <Tooltip>
      <TooltipTrigger>
        <button
          type="button"
          onclick={() => setFilter('temp')}
          class="cursor-pointer relative z-20 px-2 sm:px-4 py-1.5 text-[11px] sm:text-sm font-medium transition-colors duration-200 text-center rounded-lg flex items-center justify-center gap-1 whitespace-nowrap w-24"
          class:text-white={filterType === 'temp'}
          class:text-muted-foreground={filterType !== 'temp'}
          class:hover:text-foreground={filterType !== 'temp'}
        >
          <Clock class="h-3.5 w-3.5 flex-shrink-0" />
          <span class="truncate">{m.files_filter_temp()}</span>
        </button>
      </TooltipTrigger>
      <TooltipContent side="bottom" class="bg-popover text-popover-foreground border shadow-md">
        <p>{m.files_filter_temp_tooltip()}</p>
      </TooltipContent>
    </Tooltip>
  </div>
</div>