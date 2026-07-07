<script>
  import { m } from "$lib/paraglide/messages";
</script>

<div class="text-center mb-8 sm:mb-10">
<div class="relative inline-block">
  <div class="absolute inset-0 blur-2xl dark:bg-gradient-to-r from-cyan-400/20 via-purple-400/20 to-pink-400/20 light:bg-gradient-to-r light:from-purple-400/25 light:via-purple-500/25 light:to-purple-600/25 rounded-full"></div>
  <img
    src="/favicon.svg"
    alt="Formato logo"
    class="relative w-24 h-24 sm:w-32 sm:h-32 mx-auto transition-transform hover:scale-105 duration-300 dark:block light:hidden"
  />
  <img
    src="/favicon.light.svg"
    alt="Formato logo"
    class="relative w-24 h-24 sm:w-32 sm:h-32 mx-auto transition-transform hover:scale-105 duration-300 dark:hidden light:block"
  />
</div>
          
<h1 class="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight mb-3 -mt-3">
  <span class="bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 dark:from-cyan-400 dark:via-purple-400 dark:to-pink-400 light:from-cyan-600 light:via-purple-600 light:to-pink-600 bg-clip-text text-transparent">
    Formato
  </span>
</h1>
  
  <div class="mt-3 h-px w-20 sm:w-28 mx-auto dark:bg-gradient-to-r from-transparent via-purple-400/40 to-transparent light:bg-gradient-to-r light:from-transparent light:via-purple-500/40 light:to-transparent"></div>

  <p class="mt-3 text-sm sm:text-base dark:text-purple-300/50 light:text-purple-700/60 max-w-md mx-auto font-light tracking-wide">
    {m.hero_tagline()}
  </p>
</div>