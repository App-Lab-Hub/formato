<script lang="ts">
  import { Settings, Info } from 'lucide-svelte';
</script>

<header class="w-full border-b border-purple-500/10 bg-background/80 backdrop-blur-xl">
  <div class="flex justify-between items-center px-5 sm:px-8 py-3.5 max-w-[1700px] mx-auto">
    <a href="/" class="flex items-center gap-2 group">
      <img
        src="/favicon.svg"
        alt="Formato"
        class="w-8 h-8 sm:w-9 sm:h-9 transition-all duration-500 group-hover:scale-105 group-hover:drop-shadow-[0_0_8px_rgba(168,85,247,0.3)]"
      />
      <div class="h-5 w-px bg-purple-500/20 mx-1 hidden sm:block"></div>
      <span class="hidden sm:block text-lg font-bold bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
        Formato
      </span>
    </a>

    <nav class="flex items-center gap-1">
      <a 
        href="/about" 
        class="flex items-center gap-2 px-3.5 py-2 rounded-lg text-base font-medium text-purple-300/60 hover:text-purple-200 hover:bg-purple-500/10 transition-all duration-300"
      >
        <Info class="h-4 w-4" />
        <span class="hidden sm:inline">О нас</span>
      </a>
      <a 
        href="/settings" 
        class="flex items-center gap-2 px-3.5 py-2 rounded-lg text-base font-medium text-purple-300/60 hover:text-purple-200 hover:bg-purple-500/10 transition-all duration-300"
      >
        <Settings class="h-4 w-4" />
        <span class="hidden sm:inline">Настройки</span>
      </a>
    </nav>
  </div>
</header>