<script lang="ts">
  import type { Format } from '$lib/data/formats';

  let { format }: { format: Format } = $props();
</script>

{#if format}
  {@const SourceIcon = format.icon}
  <div class="flex flex-col items-center gap-4">
    <div class="relative rounded-3xl bg-gradient-to-br p-8 {format.color}">
      <div class="absolute inset-0 rounded-3xl bg-gradient-to-br opacity-30 blur-2xl {format.color}"></div>
      <SourceIcon class="relative h-16 w-16 {format.textColor}" />
    </div>
    <div class="text-center max-w-xl">
      <h1 class="text-2xl font-bold {format.textColor}">{format.name}</h1>
      <p class="mt-2 text-sm text-muted-foreground leading-relaxed">{format.description}</p>
    </div>
  </div>
{/if}