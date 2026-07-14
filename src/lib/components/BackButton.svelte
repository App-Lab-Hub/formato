<!-- src/lib/components/BackButton.svelte -->
<script lang="ts">
  import { ArrowLeft } from 'lucide-svelte';
  import { m } from '$lib/paraglide/messages';

  let { 
    onClick,
    disabled = false,
    opacity = 1,
    class: className = ''
  }: {
    onClick: () => void;
    disabled?: boolean;
    opacity?: number;
    class?: string;
  } = $props();
</script>

<button 
  onclick={onClick} 
  disabled={disabled} 
  class="cursor-pointer absolute top-6 left-6 group shrink-0 flex items-center gap-1 px-5 py-3 rounded-2xl transition-all duration-300 !overflow-visible
    dark:bg-card/30 light:bg-white/50 backdrop-blur-sm
    border dark:border-purple-500/10 light:border-purple-300/30
    shadow-sm hover:shadow-md
    dark:text-purple-300/70 light:text-purple-700/70
    dark:hover:text-purple-200 light:hover:text-purple-800
    dark:hover:bg-purple-500/15 light:hover:bg-purple-200/60
    active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed
    focus:outline-none focus:ring-2 focus:ring-primary/40
    {className}"
  style="opacity: {opacity};"
>
  <ArrowLeft class="h-5 w-5 transition-all duration-300 group-hover:-translate-x-0.5" />
  <span class="text-sm font-medium whitespace-nowrap">{m.settings_back()}</span>
</button>