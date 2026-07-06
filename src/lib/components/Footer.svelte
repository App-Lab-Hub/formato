<script lang="ts">
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';
</script>

<footer class="w-full border-t dark:border-purple-500/10 light:border-purple-300/20 dark:bg-background/80 light:bg-white/80 backdrop-blur-xl mt-auto">
  <div class="flex flex-col sm:flex-row items-center justify-between gap-3 px-5 sm:px-8 py-3.5 max-w-[1700px] mx-auto">
    <span class="text-base dark:text-purple-300/30 light:text-purple-500/30 font-medium">v0.1.0</span>
    
    <div class="flex items-center gap-3 text-base">
      <a 
        href="/about" 
        class="dark:text-purple-300/50 light:text-purple-600/50 dark:hover:text-purple-200 light:hover:text-purple-700 transition-colors duration-200 font-medium"
      >
        О нас
      </a>
      <span class="dark:text-purple-500/20 light:text-purple-400/20">|</span>
      <a 
        href="/settings" 
        class="dark:text-purple-300/50 light:text-purple-600/50 dark:hover:text-purple-200 light:hover:text-purple-700 transition-colors duration-200 font-medium"
      >
        Настройки
      </a>
      <span class="dark:text-purple-500/20 light:text-purple-400/20">|</span>
      <a 
        href="/dependencies" 
        class="dark:text-purple-300/50 light:text-purple-600/50 dark:hover:text-purple-200 light:hover:text-purple-700 transition-colors duration-200 font-medium"
      >
        Зависимости
      </a>
      <span class="dark:text-purple-500/20 light:text-purple-400/20">|</span>
      <a 
        href="https://github.com/yourusername/formato" 
        target="_blank"
        rel="noopener noreferrer"
        class="flex items-center gap-2 dark:text-purple-300/50 light:text-purple-600/50 dark:hover:text-purple-200 light:hover:text-purple-700 transition-colors duration-200 font-medium"
      >
        <div class="h-4 w-4">
          <FaGithub />
        </div>
        <span>GitHub</span>
      </a>
    </div>
  </div>
</footer>