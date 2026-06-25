<!-- src/lib/components/convert/TargetFormatGrid.svelte -->
<script lang="ts">
  import type { Format } from '$lib/types/format';

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
  <div class="h-px w-20 bg-border"></div>
  <span class="text-xs text-muted-foreground/50 uppercase tracking-widest">convert to</span>
  <div class="h-px w-20 bg-border"></div>
</div>

<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-5 w-full max-w-7xl mx-auto">
  {#each formats as target}
    {@const Icon = target.icon}
    <button
      onclick={() => onselect(target)}
      class="cursor-pointer group flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-5 w-full aspect-[4/5] transition-all duration-300
             {selectedTarget?.id === target.id ? 'border-primary bg-primary/5 scale-105' : 'border-border bg-card hover:border-primary/40 hover:scale-[1.02]'}
             {target.glow} {selectedTarget?.id === target.id ? 'shadow-xl' : ''}"
    >
      <div class="relative rounded-2xl bg-gradient-to-br p-5 {target.color}">
        <div class="absolute inset-0 rounded-2xl bg-gradient-to-br opacity-30 blur-2xl {target.color}"></div>
        <Icon class="relative h-11 w-11 {target.textColor}" />
      </div>
      <span class="text-base font-bold">{target.name}</span>
      <span class="text-xs text-muted-foreground line-clamp-3 max-w-[160px]">{target.description}</span>
    </button>
  {/each}
</div>