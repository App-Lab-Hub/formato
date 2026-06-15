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
    // Анимация появления букв названия
    if (titleRef) {
      const letters = titleRef.querySelectorAll('.letter');
      await animate(
        letters,
        { opacity: [0, 1], y: [24, 0] },
        { delay: (_, i) => i * 0.08, duration: 0.7, easing: [0.22, 0.61, 0.36, 1] }
      ).finished;
    }

    // Пауза чтобы рассмотреть
    await new Promise(r => setTimeout(r, 2000));

    // Запускаем анимацию выхода
    exiting = true;
    
    // Анимируем исчезновение контента
    if (contentRef) {
      await animate(
        contentRef,
        { opacity: [1, 0], scale: [1, 0.95], y: [0, -10] },
        { duration: 0.8, easing: [0.55, 0.055, 0.675, 0.19] }
      ).finished;
    }

    // Анимируем фон
    if (containerRef) {
      await animate(
        containerRef,
        { opacity: [1, 0] },
        { duration: 0.6, easing: 'ease-in' }
      ).finished;
    }

    visible = false;
    onComplete();
  });
</script>

{#if visible}
  <div
    bind:this={containerRef}
    class="fixed inset-0 z-[9999] flex items-center justify-center bg-[#0a0a0c] overflow-hidden"
    class:exiting
  >
    <!-- Фоновое свечение -->
    <div class="absolute w-[700px] h-[700px] bg-blue-500/4 blur-[180px] rounded-full animate-pulse-glow" />
    <div class="absolute w-[500px] h-[500px] bg-purple-500/6 blur-[140px] rounded-full top-1/3 left-1/4 animate-float" />
    <div class="absolute w-[300px] h-[300px] bg-cyan-500/3 blur-[100px] rounded-full bottom-1/4 right-1/4 animate-float-delayed" />

    <!-- Метеоры -->
    {#each Array(20) as _, i}
      <span
        class="absolute h-px w-px rotate-[215deg] animate-meteor rounded-full bg-white/15"
        style="top: -5px; left: {Math.random() * 100}vw; animation-delay: {Math.random() * 2}s; animation-duration: {Math.random() * 4 + 3}s;"
      >
        <span class="absolute top-1/2 w-[50px] -translate-y-1/2 bg-gradient-to-r from-white/25 to-transparent h-px" />
      </span>
    {/each}

    <!-- Центральный блок -->
    <div bind:this={contentRef} class="z-10 flex flex-col items-center gap-10">
      
      <!-- Логотип -->
      <div class="relative">
        <div class="absolute inset-0 rounded-full bg-gradient-to-r from-blue-500 via-purple-500 to-blue-500 blur-3xl opacity-15 animate-spin-slow" />
        <div class="absolute inset-0 rounded-full bg-gradient-to-r from-blue-400 to-purple-600 blur-2xl opacity-10 animate-pulse-glow" />
        
        <svg class="relative w-28 h-28" viewBox="0 0 120 120">
          <defs>
            <linearGradient id="splashGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#60a5fa" />
              <stop offset="50%" stop-color="#a78bfa" />
              <stop offset="100%" stop-color="#a855f7" />
            </linearGradient>
            <filter id="glow">
              <feGaussianBlur stdDeviation="2" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          
          <path d="M48 22 Q36 22 36 34 L36 46 Q36 54 24 54 Q36 54 36 62 L36 74 Q36 86 48 86"
                fill="none" stroke="url(#splashGrad)" stroke-width="5"
                stroke-linecap="round" stroke-linejoin="round"
                stroke-dasharray="200" stroke-dashoffset="200"
                filter="url(#glow)"
                class="animate-draw-left" />
          
          <path d="M72 22 Q84 22 84 34 L84 46 Q84 54 96 54 Q84 54 84 62 L84 74 Q84 86 72 86"
                fill="none" stroke="url(#splashGrad)" stroke-width="5"
                stroke-linecap="round" stroke-linejoin="round"
                stroke-dasharray="200" stroke-dashoffset="200"
                filter="url(#glow)"
                class="animate-draw-right" />
        </svg>
      </div>

      <!-- Название -->
      <h1 bind:this={titleRef} class="text-5xl font-bold tracking-[0.3em] uppercase">
        {#each 'FORMATO'.split('') as letter}
          <span class="letter inline-block opacity-0 bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
            {letter}
          </span>
        {/each}
      </h1>

      <!-- Подпись -->
      <div class="flex flex-col items-center gap-2 fade-up-delayed">
        <p class="text-sm text-white/40 tracking-[0.3em] uppercase">Universal Data Converter</p>
        <span class="text-xs text-white/20 font-mono">v0.1.0</span>
      </div>

      <!-- Прогресс-бар -->
      <div class="w-64 h-px bg-white/5 rounded-full overflow-hidden fade-up-more-delayed">
        <div class="h-full bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 rounded-full animate-progress-bar origin-left" />
      </div>

    </div>
  </div>
{/if}