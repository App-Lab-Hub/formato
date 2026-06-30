<script lang="ts">
  import { Settings, Info } from 'lucide-svelte';
</script>

<header class="w-full border-b border-purple-500/10 bg-background/80 backdrop-blur-xl">
  <div class="flex justify-between items-center px-5 sm:px-8 py-3.5 max-w-[1700px] mx-auto">
    <a href="/" class="flex items-center gap-2 group">
      <img
        src="/favicon.svg"
        alt="Formato"
        class="w-7 h-7 sm:w-8 sm:h-8 transition-all duration-500 group-hover:scale-105 group-hover:drop-shadow-[0_0_8px_rgba(168,85,247,0.3)]"
      />
      <div class="h-4 w-px bg-purple-500/20 mx-1 hidden sm:block"></div>
      <span class="hidden sm:block text-sm font-bold bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
        Formato
      </span>
    </a>

    <nav class="flex items-center gap-1">
      <a 
        href="/about" 
        class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm text-purple-300/60 hover:text-purple-200 hover:bg-purple-500/10 transition-all duration-300"
      >
        <Info class="h-4 w-4" />
        <span class="hidden sm:inline font-medium">О нас</span>
      </a>
      <a 
        href="/settings" 
        class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm text-purple-300/60 hover:text-purple-200 hover:bg-purple-500/10 transition-all duration-300"
      >
        <Settings class="h-4 w-4" />
        <span class="hidden sm:inline font-medium">Настройки</span>
      </a>
    </nav>
  </div>
</header>