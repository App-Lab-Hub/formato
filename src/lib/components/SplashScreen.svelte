<script lang="ts">
  import { onMount } from 'svelte';
  import { animate } from '@motionone/dom';
  import "$lib/styles/splashScreen.css";

  let { onComplete = () => {} } = $props();
  let visible = $state(true);
  let exiting = $state(false);
  let titleRef = $state<HTMLHeadingElement | null>(null);
  let containerRef = $state<HTMLDivElement | null>(null);
  let contentRef = $state<HTMLDivElement | null>(null);

  onMount(async () => {
    // Ждём отрисовки скобок
    await new Promise(r => setTimeout(r, 1600));

    // Анимация появления букв — с задержкой и драматичным эффектом
    if (titleRef) {
      const letters = titleRef.querySelectorAll('.letter');
      await animate(
        letters,
        { opacity: [0, 1], y: [30, 0], filter: ['blur(8px)', 'blur(0px)'] },
        { delay: (_, i) => i * 0.1, duration: 0.8, easing: [0.22, 0.61, 0.36, 1] }
      ).finished;
    }

    // Держим 3 секунды
    await new Promise(r => setTimeout(r, 3000));

    // Запускаем анимацию проваливания
    exiting = true;
    
    if (contentRef) {
      await animate(
        contentRef,
        { 
          opacity: [1, 0], 
          scale: [1, 0.85], 
          y: [0, 40],
          filter: ['blur(0px)', 'blur(12px)']
        },
        { duration: 0.7, easing: [0.55, 0.055, 0.675, 0.19] }
      ).finished;
    }

    if (containerRef) {
      await animate(
        containerRef,
        { opacity: [1, 0] },
        { duration: 0.5, easing: 'ease-in' }
      ).finished;
    }

    visible = false;
    onComplete();
  });
</script>

{#if visible}
  <div
    bind:this={containerRef}
    class="fixed inset-0 z-[9999] flex items-center justify-center bg-[#060608] overflow-hidden"
    class:exiting
  >
    <!-- {/* Тёмный виньетка-градиент по краям для таинственности */} -->
    <div class="absolute inset-0 bg-radial-vignette opacity-80" />
    
    <!-- {/* Световые блики — как отсветы в темноте */} -->
    <div class="absolute left-0 top-1/4 w-[2px] h-64 bg-gradient-to-b from-transparent via-blue-400/30 to-transparent animate-shimmer-left" />
    <div class="absolute right-0 top-1/3 w-[2px] h-48 bg-gradient-to-b from-transparent via-purple-400/25 to-transparent animate-shimmer-right" />
    <div class="absolute left-1/3 bottom-1/4 w-[1px] h-32 bg-gradient-to-b from-transparent via-cyan-400/20 to-transparent animate-shimmer-left-delayed" />
    <div class="absolute right-1/4 top-1/4 w-[1px] h-40 bg-gradient-to-b from-transparent via-pink-400/15 to-transparent animate-shimmer-right-delayed" />

    <!-- {/* Фоновые свечения — более заметные */} -->
    <div class="absolute w-[800px] h-[800px] bg-blue-500/6 blur-[200px] rounded-full animate-pulse-glow" />
    <div class="absolute w-[500px] h-[500px] bg-purple-500/8 blur-[150px] rounded-full top-1/3 left-1/4 animate-float" />
    <div class="absolute w-[350px] h-[350px] bg-cyan-500/4 blur-[120px] rounded-full bottom-1/4 right-1/4 animate-float-delayed" />

    <!-- {/* Метеоры — больше и заметнее */} -->
    {#each Array(25) as _, i}
      <span
        class="absolute h-[2px] w-[2px] rotate-[215deg] animate-meteor rounded-full"
        style="
          top: -5px;
          left: {Math.random() * 100}vw;
          animation-delay: {Math.random() * 3}s;
          animation-duration: {Math.random() * 5 + 4}s;
          background: linear-gradient(90deg, {['#60a5fa', '#a78bfa', '#a855f7', '#60a5fa'][i % 4]}, transparent);
        "
      >
        <span class="absolute top-1/2 w-[60px] -translate-y-1/2 bg-gradient-to-r from-white/20 to-transparent h-[1px]" />
      </span>
    {/each}

    <!-- {/* Центральный блок */} -->
    <div bind:this={contentRef} class="z-10 flex flex-col items-center gap-12">
      
      <!-- {/* Логотип со скобками */} -->
      <div class="relative">
        <div class="absolute inset-0 rounded-full bg-gradient-to-r from-blue-500 via-purple-500 to-blue-500 blur-3xl opacity-20 animate-spin-slow" />
        <div class="absolute inset-0 rounded-full bg-gradient-to-r from-blue-400 to-purple-600 blur-2xl opacity-15 animate-pulse-glow" />
        
        <svg class="relative w-32 h-32" viewBox="0 0 120 120">
          <defs>
            <linearGradient id="splashGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#60a5fa" />
              <stop offset="50%" stop-color="#a78bfa" />
              <stop offset="100%" stop-color="#a855f7" />
            </linearGradient>
            <filter id="glow">
              <feGaussianBlur stdDeviation="2.5" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          
          <!-- {/* Левая и правая скобки — рисуются одновременно */} -->
          <g filter="url(#glow)">
            <path d="M48 22 Q36 22 36 34 L36 46 Q36 54 24 54 Q36 54 36 62 L36 74 Q36 86 48 86"
                  fill="none" stroke="url(#splashGrad)" stroke-width="5"
                  stroke-linecap="round" stroke-linejoin="round"
                  stroke-dasharray="200" stroke-dashoffset="200"
                  class="animate-draw-both" />
            
            <path d="M72 22 Q84 22 84 34 L84 46 Q84 54 96 54 Q84 54 84 62 L84 74 Q84 86 72 86"
                  fill="none" stroke="url(#splashGrad)" stroke-width="5"
                  stroke-linecap="round" stroke-linejoin="round"
                  stroke-dasharray="200" stroke-dashoffset="200"
                  class="animate-draw-both" />
          </g>
        </svg>
      </div>

      <!-- {/* Название с драматичным появлением */} -->
      <h1 bind:this={titleRef} class="text-6xl font-bold tracking-[0.35em] uppercase">
        {#each 'FORMATO'.split('') as letter}
          <span class="letter inline-block bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent"
                style="opacity: 0; filter: blur(8px);">
            {letter}
          </span>
        {/each}
      </h1>

      <!-- {/* Подпись */} -->
      <div class="flex flex-col items-center gap-2 fade-up-delayed">
        <p class="text-sm text-white/40 tracking-[0.35em] uppercase">Universal Data Converter</p>
        <span class="text-xs text-white/20 font-mono">v0.1.0</span>
      </div>

      <!-- {/* Прогресс-бар */} -->
      <div class="w-72 h-px bg-white/5 rounded-full overflow-hidden fade-up-more-delayed">
        <div class="h-full bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 rounded-full animate-progress-bar origin-left" />
      </div>

    </div>
  </div>
{/if}