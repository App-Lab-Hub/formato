<script lang="ts">
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';

  let { format }: { format: Format } = $props();
</script>

{#if format}
  {@const SourceIcon = format.icon}
  <div class="flex flex-col items-center gap-5 text-center select-none">
    <!-- Иконка с глубоким, но аккуратным свечением -->
    <div class="relative rounded-3xl bg-gradient-to-br p-7 sm:p-8 transition-transform duration-500 hover:scale-105 {format.color}">
      <!-- Размытое фоновое облако (Blur) под иконкой -->
      <div class="absolute inset-0 rounded-3xl bg-gradient-to-br opacity-25 blur-2xl {format.color}"></div>
      <div class="flex-shrink-0 h-14 w-14 sm:h-16 sm:w-16 relative z-10 flex items-center justify-center">
        <SourceIcon class="w-full h-full {format.textColor}" />
      </div>
    </div>
    
    <!-- Блок текстов (Адаптирован под спокойную палитру) -->
    <div class="max-w-xl px-4">
      <h1 class="text-2xl sm:text-3xl font-black tracking-wide {format.textColor}">
        {format.name}
      </h1>
      <p class="mt-2.5 text-sm font-medium leading-relaxed dark:text-muted-foreground light:text-neutral-500/90 max-w-md mx-auto">
        {(m as any)[`format_desc_${format.id}`]()}
      </p>
    </div>
  </div>
{/if}
