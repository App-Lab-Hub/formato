<!-- BoostyBtn.svelte -->
<script lang="ts">
  interface Props {
    hideTextOnMobile?: boolean;
    class?: string;
  }
  
  let { hideTextOnMobile = false, class: customClass = '' }: Props = $props();
</script>

<a 
  href="https://boosty.to/applabhub" 
  target="_blank"
  rel="noopener noreferrer"
  class="boosty-pulse inline-flex items-center justify-center gap-2 rounded-full bg-gradient-to-r from-amber-500/15 to-orange-500/15 dark:from-amber-500/25 dark:to-orange-500/25 border border-amber-500/30 dark:text-amber-300 light:text-amber-700 hover:scale-105 hover:shadow-lg transition-all duration-200 font-medium {customClass || 'px-3.5 py-1.5 sm:py-0.5 text-xs sm:text-sm'}"
>
  <img 
    src="/boosty.svg" 
    alt="Boosty" 
    class="h-3.5 w-3.5 flex-shrink-0"
  />
  <span class={hideTextOnMobile ? "hidden xs:inline" : ""}>Boosty</span>
</a>
