<!-- GithubBtn.svelte -->
<script lang="ts">
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';

  interface Props {
    hideTextOnMobile?: boolean;
    class?: string; // Позволяет накидывать кастомные классы (например, px-6 py-3 для блока About)
  }
  
  let { hideTextOnMobile = false, class: customClass = '' }: Props = $props(); // Сниппет Svelte 5 (если Svelte 4, замени на: export let hideTextOnMobile = false; export let class = '';)
</script>

<a 
  href="https://github.com" 
  target="_blank"
  rel="noopener noreferrer"
  class="github-pulse inline-flex items-center justify-center gap-2 rounded-full bg-gradient-to-r from-purple-500/15 to-indigo-500/15 dark:from-purple-500/25 dark:to-indigo-500/25 border border-purple-500/30 dark:text-purple-300 light:text-purple-700 hover:scale-105 hover:shadow-lg transition-all duration-200 font-medium {customClass || 'px-3.5 py-1.5 sm:py-0.5 text-xs sm:text-sm'}"
>
  <div class="h-3.5 w-3.5 flex-shrink-0">
    <FaGithub />
  </div>
  <span class={hideTextOnMobile ? "hidden xs:inline" : ""}>GitHub</span>
</a>
