<script lang="ts">
  import { Heart } from 'lucide-svelte';
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';
</script>

<footer class="w-full border-t border-border/30 bg-background/50 backdrop-blur-sm mt-auto">
  <div class="flex flex-col sm:flex-row items-center justify-between gap-2 px-4 sm:px-6 py-3 max-w-[1700px] mx-auto">
    <div class="flex items-center gap-3 text-xs sm:text-sm text-muted-foreground/40">
      <span>v0.1.0</span>
      <span class="hidden sm:inline">•</span>
      <span class="flex items-center gap-2">
        Сделано с <Heart class="h-3 w-3 text-red-400/60 fill-red-400/20" />
      </span>
    </div>
    <div class="flex items-center gap-2 text-xs sm:text-sm">
      <a 
        href="/about" 
        class="text-muted-foreground/40 hover:text-primary/70 transition-colors duration-200"
      >
        О нас
      </a>
      <span class="text-muted-foreground/20">|</span>
      <a 
        href="/settings" 
        class="text-muted-foreground/40 hover:text-primary/70 transition-colors duration-200"
      >
        Настройки
      </a>
      <span class="text-muted-foreground/20">|</span>
      <a 
        href="/dependencies" 
        class="text-muted-foreground/40 hover:text-primary/70 transition-colors duration-200"
      >
        Зависимости
      </a>
      <span class="text-muted-foreground/20">|</span>
      <a 
        href="https://github.com/yourusername/formato" 
        target="_blank"
        rel="noopener noreferrer"
        class="flex items-center gap-1 text-muted-foreground/40 hover:text-primary/70 transition-colors duration-200"
      >
        <div class="h-3.5 w-3.5">
          <FaGithub />
        </div>
        <span>GitHub</span>
      </a>
    </div>
  </div>
</footer>