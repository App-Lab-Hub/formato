<!-- Footer.svelte -->
<script lang="ts">
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';
  import { m } from '$lib/paraglide/messages';
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
    
    <!-- Контейнер для кнопок со смещением вниз для оптического выравнивания -->
    <div class="flex items-center gap-3 w-full sm:w-auto justify-center mt-1 sm:mt-0 transform translate-y-[1px]">
      <!-- GitHub -->
      <a 
        href="https://github.com" 
        target="_blank"
        rel="noopener noreferrer"
        class="flex items-center gap-2 px-3.5 py-1.5 sm:py-0.5 rounded-full bg-gradient-to-r from-purple-500/15 to-indigo-500/15 dark:from-purple-500/25 dark:to-indigo-500/25 border border-purple-500/30 dark:text-purple-300 light:text-purple-700 hover:scale-105 hover:shadow-lg transition-all duration-200 font-medium text-xs sm:text-sm"
      >
        <div class="h-3.5 w-3.5">
          <FaGithub />
        </div>
        <span>GitHub</span>
      </a>

      <!-- Boosty -->
      <a 
        href="https://boosty.to" 
        target="_blank"
        rel="noopener noreferrer"
        class="flex items-center gap-2 px-3.5 py-1.5 sm:py-0.5 rounded-full bg-gradient-to-r from-amber-500/15 to-orange-500/15 dark:from-amber-500/25 dark:to-orange-500/25 border border-amber-500/30 dark:text-amber-300 light:text-amber-700 hover:scale-105 hover:shadow-lg transition-all duration-200 font-medium text-xs sm:text-sm"
      >
        <img 
          src="/boosty.svg" 
          alt="Boosty" 
          class="h-3.5 w-3.5"
        />
        <span>Boosty</span>
      </a>
    </div>

  </div>
</footer>
