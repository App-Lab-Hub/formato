<script lang="ts">
  import { onMount } from 'svelte';
  import "$lib/styles/splashScreen.css";

  let { onComplete = () => {} } = $props();
  let visible = $state(true);
  let containerRef = $state<HTMLDivElement | null>(null);

  onMount(async () => {
    // Генерируем метеоры
    // if (containerRef) {
    //   for (let i = 0; i < 40; i++) {
    //     const m = document.createElement("div");
    //     m.className = "meteor";
    //     m.style.setProperty("--delay", Math.random() * 6 + "s");
    //     m.style.setProperty("--duration", Math.random() * 10 + 8 + "s");
    //     m.style.left = Math.random() < 0.7
    //       ? Math.random() * 55 + "vw"
    //       : 40 + Math.random() * 60 + "vw";
    //     m.style.top = Math.random() * -20 - 5 + "px";
    //     const colors = ["#60a5fa", "#a78bfa", "#a855f7", "#60a5fa"];
    //     m.style.background = colors[i % 4];
    //     containerRef.appendChild(m);
    //   }
    // }

    // Ждём 5.5 секунд (анимация 4.5s + пауза 1s)
    await new Promise(r => setTimeout(r, 7500));

    // Запускаем fade-out
    containerRef?.classList.add('fade-out');

    // Ждём окончания анимации
    await new Promise(r => setTimeout(r, 1100));
    onComplete();
  });
</script>

{#if visible}
  <div bind:this={containerRef} class="splashscreen">
    <div class="bg-radial-vignette"></div>
    <div class="aurora-left"></div>
    <div class="shimmer-right"></div>
    <div class="shimmer-right2"></div>
    <div class="glow glow-1"></div>
    <div class="glow glow-2"></div>
    <div class="glow glow-3"></div>

    <div class="content">
      <div class="logo-wrapper">
        <div class="logo-glow"></div>
        <div class="logo-inner"></div>
        <svg class="logo-svg" viewBox="0 0 120 120">
          <defs>
            <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#60a5fa" />
              <stop offset="50%" stop-color="#a78bfa" />
              <stop offset="100%" stop-color="#a855f7" />
            </linearGradient>
            <filter id="glow-filter" x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur stdDeviation="3" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          <path class="bracket" d="M48 22 Q36 22 36 34 L36 46 Q36 54 24 54 Q36 54 36 62 L36 74 Q36 86 48 86" />
          <path class="bracket" d="M72 22 Q84 22 84 34 L84 46 Q84 54 96 54 Q84 54 84 62 L84 74 Q84 86 72 86" />
        </svg>
      </div>

      <h1>
        <span style="animation-delay: 1.5s">F</span>
        <span style="animation-delay: 1.7s">O</span>
        <span style="animation-delay: 1.9s">R</span>
        <span style="animation-delay: 2.1s">M</span>
        <span style="animation-delay: 2.3s">A</span>
        <span style="animation-delay: 2.5s">T</span>
        <span style="animation-delay: 2.7s">O</span>
      </h1>

      <div class="subtitle">
        <p>Universal Data Converter</p>
        <span>v0.1.0</span>
      </div>

      <div class="progress">
        <div class="progress-bar"></div>
      </div>
    </div>
  </div>
{/if}