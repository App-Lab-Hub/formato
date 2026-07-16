<script lang="ts">
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';

  let { format }: { format: Format } = $props();
</script>

{#if format}
  {@const SourceIcon = format.icon}
  <div class="flex flex-col items-center gap-4">
    <div class="relative rounded-3xl bg-gradient-to-br p-8 {format.color}">
      <div class="absolute inset-0 rounded-3xl bg-gradient-to-br opacity-30 blur-2xl {format.color}"></div>
      <div class="flex-shrink-0 h-16 w-16">
        <SourceIcon class="relative w-full h-full {format.textColor}" />
      </div>
    </div>
    <div class="text-center max-w-xl">
      <h1 class="text-2xl font-bold {format.textColor}">{format.name}</h1>
      <p class="mt-2 text-sm dark:text-muted-foreground light:text-purple-700/60 leading-relaxed">
        {(m as any)[`format_desc_${format.id}`]()}
      </p>
    </div>
  </div>
{/if}