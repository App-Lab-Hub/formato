<!-- src/lib/components/ModeTabs.svelte -->
<script lang="ts">
  import { FolderOpen, Type } from 'lucide-svelte';
  import { m } from '$lib/paraglide/messages';

  let { 
    mode = 'file',
    onModeChange
  }: {
    mode?: 'file' | 'text';
    onModeChange?: (mode: 'file' | 'text') => void;
  } = $props();

  function setMode(type: 'file' | 'text') {
    if (type === mode) return;
    if (onModeChange) {
      onModeChange(type);
    }
  }
</script>

<div class="flex items-center w-full sm:w-auto justify-start">
  <div 
    data-active={mode}
    class="mode-container relative flex items-center w-full sm:w-fit dark:bg-card/30 light:bg-purple-200/30 p-1 rounded-xl border dark:border-border/50 light:border-purple-300/40 h-10 min-w-[200px]"
  >
    <!-- Плавающая плашка -->
    <div class="mode-indicator absolute top-1 bottom-1 left-1 rounded-lg z-10 shadow-lg shadow-primary/20"
      style="
        width: calc(50% - 6px);
        background: {mode === 'file' 
          ? 'linear-gradient(135deg, #8b5cf6, #6d28d9)' 
          : 'linear-gradient(135deg, #06b6d4, #0891b2)'};
      "
    ></div>

    <!-- Файлы -->
    <button
      type="button"
      onclick={() => setMode('file')}
      class="cursor-pointer relative z-20 px-2 sm:px-3 py-1.5 text-[11px] sm:text-sm font-medium transition-colors duration-200 text-center rounded-lg flex items-center justify-center gap-1 whitespace-nowrap flex-1 min-w-0"
      class:text-white={mode === 'file'}
      class:text-muted-foreground={mode !== 'file'}
      class:hover:text-foreground={mode !== 'file'}
    >
      <FolderOpen class="h-3.5 w-3.5 flex-shrink-0" />
      <span class="truncate">{m.input_mode_files()}</span>
    </button>

    <!-- Текст -->
    <button
      type="button"
      onclick={() => setMode('text')}
      class="cursor-pointer relative z-20 px-2 sm:px-3 py-1.5 text-[11px] sm:text-sm font-medium transition-colors duration-200 text-start rounded-lg flex items-center justify-center gap-1 whitespace-nowrap flex-1 min-w-0"
      class:text-white={mode === 'text'}
      class:text-muted-foreground={mode !== 'text'}
      class:hover:text-foreground={mode !== 'text'}
    >
      <Type class="h-3.5 w-3.5 flex-shrink-0" />
      <span class="truncate">{m.input_mode_text()}</span>
    </button>
  </div>
</div>