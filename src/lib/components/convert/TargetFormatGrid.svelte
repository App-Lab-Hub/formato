<script lang="ts">
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';

  let {
    formats,
    selectedTarget,
    onselect,
  }: {
    formats: Format[];
    selectedTarget: Format | null;
    onselect: (format: Format) => void;
  } = $props();
</script>

<div class="flex items-center gap-4">
  <div class="h-px w-20 dark:bg-border light:bg-border/60"></div>
  <span class="text-xs dark:text-muted-foreground/50 light:text-muted-foreground/60 uppercase tracking-widest">{m.convert_to()}</span>
  <div class="h-px w-20 dark:bg-border light:bg-border/60"></div>
</div>

<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-5 w-full max-w-7xl mx-auto">
  {#each formats as target}
    {@const Icon = target.icon}
    <button
      onclick={() => onselect(target)}
      class="cursor-pointer group flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-5 w-full aspect-[4/5] transition-all duration-300
             {selectedTarget?.id === target.id ? 'border-primary bg-primary/5 scale-105 shadow-xl' : 'dark:border-border light:border-border/50 dark:bg-card light:bg-white/60 dark:hover:border-primary/40 light:hover:border-primary/50 hover:scale-[1.02]'}
             {target.glow}"
    >
      <div class="relative rounded-2xl bg-gradient-to-br p-5 {target.color}">
        <div class="absolute inset-0 rounded-2xl bg-gradient-to-br opacity-30 blur-2xl {target.color}"></div>
        <Icon class="relative h-11 w-11 {target.textColor}" />
      </div>
      <span class="text-base font-bold dark:text-foreground light:text-foreground/90">{target.name}</span>
      <span class="text-xs dark:text-muted-foreground light:text-muted-foreground/60 line-clamp-3 max-w-[160px]">
        {(m as any)[`format_desc_${target.id}`]()}
      </span>
    </button>
  {/each}
</div>