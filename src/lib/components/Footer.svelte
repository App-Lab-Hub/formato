<!-- Footer.svelte -->
<script lang="ts">
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';
  import { m } from '$lib/paraglide/messages';
</script>

<footer class="w-full border-t dark:border-purple-500/10 light:border-purple-300/30 dark:bg-background/80 light:bg-purple-200/50 backdrop-blur-xl mt-auto">
  <div class="flex flex-col sm:flex-row items-center justify-between gap-3 px-5 sm:px-8 py-3.5 max-w-[1700px] mx-auto">
    <span class="text-base dark:text-purple-300/30 light:text-purple-700/40 font-medium">v0.1.0</span>
    
    <div class="flex items-center gap-3 text-base flex-wrap justify-center">
      <a 
        href="/files" 
        class="dark:text-purple-300/50 light:text-purple-700/70 dark:hover:text-purple-200 light:hover:text-purple-800 transition-colors duration-200 font-medium"
      >
        {m.files_title()}
      </a>
      <span class="dark:text-purple-500/20 light:text-purple-400/30">|</span>
      <a 
        href="/about" 
        class="dark:text-purple-300/50 light:text-purple-700/70 dark:hover:text-purple-200 light:hover:text-purple-800 transition-colors duration-200 font-medium"
      >
        {m.about_title()}
      </a>
      <span class="dark:text-purple-500/20 light:text-purple-400/30">|</span>
      <a 
        href="/settings" 
        class="dark:text-purple-300/50 light:text-purple-700/70 dark:hover:text-purple-200 light:hover:text-purple-800 transition-colors duration-200 font-medium"
      >
        {m.settings_title()}
      </a>
      <span class="dark:text-purple-500/20 light:text-purple-400/30">|</span>
      <a 
        href="/dependencies" 
        class="dark:text-purple-300/50 light:text-purple-700/70 dark:hover:text-purple-200 light:hover:text-purple-800 transition-colors duration-200 font-medium"
      >
        {m.deps_title()}
      </a>
      <span class="dark:text-purple-500/20 light:text-purple-400/30">|</span>
      
      <!-- GitHub с таким же стилем как Boosty -->
      <a 
        href="https://github.com/App-Lab-Hub/formato" 
        target="_blank"
        rel="noopener noreferrer"
        class="flex items-center gap-2 px-3 py-0.5 rounded-full bg-gradient-to-r from-purple-500/15 to-indigo-500/15 dark:from-purple-500/25 dark:to-indigo-500/25 border border-purple-500/30 dark:text-purple-300 light:text-purple-700 hover:scale-105 hover:shadow-lg transition-all duration-200 font-medium"
      >
        <div class="h-3.5 w-3.5">
          <FaGithub />
        </div>
        <span>GitHub</span>
      </a>
      
      <span class="dark:text-purple-500/20 light:text-purple-400/30">|</span>
      
      <!-- Boosty -->
      <a 
        href="https://boosty.to/yourusername" 
        target="_blank"
        rel="noopener noreferrer"
        class="flex items-center gap-2 px-3 py-0.5 rounded-full bg-gradient-to-r from-amber-500/15 to-orange-500/15 dark:from-amber-500/25 dark:to-orange-500/25 border border-amber-500/30 dark:text-amber-300 light:text-amber-700 hover:scale-105 hover:shadow-lg transition-all duration-200 font-medium"
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