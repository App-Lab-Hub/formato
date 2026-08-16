<!-- Footer.svelte -->
<script lang="ts">
  import { m } from '$lib/paraglide/messages';
  import GithubBtn from '$lib/components/GithubBtn.svelte';
  import BoostyBtn from '$lib/components/BoostyBtn.svelte';
  import "$lib/styles/footer.css";
</script>

<footer class="w-full border-t dark:border-purple-500/10 light:border-purple-300/30 dark:bg-background/80 light:bg-purple-200/50 backdrop-blur-xl mt-auto">
  <div class="flex flex-wrap items-center justify-center gap-x-4 gap-y-3 px-4 sm:px-8 py-4 max-w-[1700px] mx-auto text-sm sm:text-base">
    
    <a 
      href="/files" 
      class="dark:text-purple-300/50 light:text-purple-700/70 dark:hover:text-purple-200 light:hover:text-purple-800 transition-colors duration-200 font-medium"
    >
      {m.files_title()}
    </a>
    <span class="hidden sm:inline dark:text-purple-500/20 light:text-purple-400/30">|</span>
    
    <a 
      href="/about" 
      class="dark:text-purple-300/50 light:text-purple-700/70 dark:hover:text-purple-200 light:hover:text-purple-800 transition-colors duration-200 font-medium"
    >
      {m.about_title()}
    </a>
    <span class="hidden sm:inline dark:text-purple-500/20 light:text-purple-400/30">|</span>
    
    <a 
      href="/settings" 
      class="dark:text-purple-300/50 light:text-purple-700/70 dark:hover:text-purple-200 light:hover:text-purple-800 transition-colors duration-200 font-medium"
    >
      {m.settings_title()}
    </a>
    <span class="hidden sm:inline dark:text-purple-500/20 light:text-purple-400/30">|</span>
    
    <a 
      href="/dependencies" 
      class="dark:text-purple-300/50 light:text-purple-700/70 dark:hover:text-purple-200 light:hover:text-purple-800 transition-colors duration-200 font-medium"
    >
      {m.deps_title()}
    </a>
    <span class="hidden sm:inline dark:text-purple-500/20 light:text-purple-400/30">|</span>
    
    <!-- Контейнер для кнопок -->
    <div class="flex items-center gap-3 w-full sm:w-auto justify-center mt-1 sm:mt-0 transform translate-y-[1px]">
      <GithubBtn />
      <BoostyBtn />
    </div>

  </div>
</footer>
