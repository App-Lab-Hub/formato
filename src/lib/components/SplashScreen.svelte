<script lang="ts">
  import { onMount } from 'svelte';
  import { animate, stagger } from '@motionone/dom';
  import "$lib/styles/splashScreen.css";

  let { onComplete = () => {} } = $props();
  let visible = $state(true);
  let container = $state<HTMLDivElement | null>(null);
  let bracketL = $state<SVGPathElement | null>(null);
  let bracketR = $state<SVGPathElement | null>(null);
  let logoGlow = $state<HTMLDivElement | null>(null);
  let title = $state<HTMLHeadingElement | null>(null);
  let subtitle = $state<HTMLDivElement | null>(null);
  let progressBar = $state<HTMLDivElement | null>(null);
  let progressWrapper = $state<HTMLDivElement | null>(null);
  let glow1 = $state<HTMLDivElement | null>(null);
  let glow2 = $state<HTMLDivElement | null>(null);
  let splashContent = $state<HTMLDivElement | null>(null);
  let scanLine = $state<HTMLDivElement | null>(null);
  let overlay = $state<HTMLDivElement | null>(null);

  onMount(async () => {
    // Фоновые свечения – вход и вечная пульсация
    async function startGlowPulse() {
      if (glow1) {
        await animate(glow1, { opacity: [0, 0.35] }, { duration: 2.5, easing: 'ease-out' }).finished;
        animate(glow1, { opacity: [0.25, 0.45] }, { duration: 2, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' });
      }
      if (glow2) {
        await animate(glow2, { opacity: [0, 0.3] }, { duration: 3, delay: 0.3, easing: 'ease-out' }).finished;
        animate(glow2, { opacity: [0.15, 0.35] }, { duration: 2.5, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' });
      }
    }
    startGlowPulse();

    // Логотип – вращение + мягкий scale
    if (logoGlow) {
      animate(logoGlow, { rotate: 360 }, { duration: 35, easing: 'linear', repeat: Infinity });
      animate(logoGlow, { opacity: [0, 0.45] }, { duration: 2.5, easing: 'ease-out' });
      animate(logoGlow, { scale: [1, 1.08] }, { duration: 4, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' });
    }

    // Сканирующая линия
    async function startScanLine() {
      await new Promise(r => setTimeout(r, 3500));
      if (scanLine) {
        animate(scanLine, { top: ['-5%', '105%'] }, { duration: 5, repeat: Infinity, easing: 'linear' });
        animate(scanLine, { opacity: [0, 0.3, 0] }, { duration: 5, repeat: Infinity, easing: 'linear' });
      }
    }
    startScanLine();

    // Основная анимация – скобки, заголовок, подзаголовок, прогресс
    if (bracketL && bracketR) {
      animate(bracketL, { opacity: [0, 1] }, { duration: 0.6, easing: 'ease-out' });
      animate(bracketR, { opacity: [0, 1] }, { duration: 0.6, easing: 'ease-out' });

      await Promise.all([
        animate(bracketL, { strokeDashoffset: [150, 0] }, { duration: 3, easing: 'ease-in-out' }).finished,
        animate(bracketR, { strokeDashoffset: [150, 0] }, { duration: 3, easing: 'ease-in-out' }).finished,
      ]);
    }

    if (title) {
      const letters = title.querySelectorAll('.letter');
      await animate(letters,
        { opacity: [0, 1] },
        { delay: stagger(0.08), duration: 0.8, easing: [0.34, 1.56, 0.64, 1] }
      ).finished;
    }

    if (subtitle) {
      await animate(subtitle,
        { opacity: [0, 1], transform: ['translateY(30px)', 'translateY(0)'] },
        { duration: 1.2, easing: [0.34, 1.56, 0.64, 1] }
      ).finished;
    }

    if (progressWrapper && progressBar) {
      animate(progressWrapper, { opacity: [0, 1] }, { duration: 0.8, easing: 'ease-out' });
      await animate(progressBar,
        { transform: ['scaleX(0)', 'scaleX(1)'] },
        { duration: 3.5, easing: [0.4, 0, 0.2, 1] }
      ).finished;
    }

    await new Promise(r => setTimeout(r, 2000));

    // Эффект стирания и финал
    if (container && splashContent && title) {
      const letters = title.querySelectorAll('.letter');

      // Анимируем исчезновение всех элементов одновременно
      await Promise.all([
        // Буквы заголовка
        ...Array.from(letters).map((letter, i) =>
          animate(letter as HTMLElement, { opacity: [1, 0] }, { duration: 0.6, delay: i * 0.06, easing: 'ease-out' }).finished
        ),
        // Подзаголовок
        subtitle ? animate(subtitle, { opacity: [1, 0] }, { duration: 0.6, delay: 0.2, easing: 'ease-out' }).finished : Promise.resolve(),
        // Прогресс-бар
        progressWrapper ? animate(progressWrapper, { opacity: [1, 0] }, { duration: 0.6, delay: 0.1, easing: 'ease-out' }).finished : Promise.resolve(),
        // SVG логотип (весь элемент)
        logoGlow?.parentElement ? animate(logoGlow.parentElement, { opacity: [1, 0] }, { duration: 0.6, delay: 0.1, easing: 'ease-out' }).finished : Promise.resolve(),
        // Свечение логотипа
        logoGlow ? animate(logoGlow, { opacity: [0.45, 0] }, { duration: 0.6, delay: 0.1, easing: 'ease-out' }).finished : Promise.resolve(),
        // Фоновые свечения
        glow1 ? animate(glow1, { opacity: [0.45, 0] }, { duration: 0.8, easing: 'ease-out' }).finished : Promise.resolve(),
        glow2 ? animate(glow2, { opacity: [0.35, 0] }, { duration: 0.8, delay: 0.1, easing: 'ease-out' }).finished : Promise.resolve(),
        // Сканирующая линия
        scanLine ? animate(scanLine, { opacity: [0.3, 0] }, { duration: 0.6, easing: 'ease-out' }).finished : Promise.resolve(),
        // Виньетка
        overlay ? animate(overlay, { opacity: [0.9, 0] }, { duration: 0.8, easing: 'ease-out' }).finished : Promise.resolve(),
        // Основной контент
        animate(splashContent, { opacity: [1, 0] }, { duration: 0.6, easing: 'ease-out' }).finished,
      ]);

      await new Promise(r => setTimeout(r, 300));
      
      // Финальное исчезновение контейнера
      await animate(container, { opacity: [1, 0] }, { duration: 0.8, easing: 'ease-out' }).finished;
      await new Promise(r => setTimeout(r, 100));
    }

    visible = false;
    onComplete();
  });
</script>

{#if visible}
  <div bind:this={container} class="fixed inset-0 z-[9999] flex items-center justify-center bg-[#0a0a0f] overflow-hidden">
    
    <!-- Тёмная виньетка -->
    <div bind:this={overlay} class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_30%,rgba(0,0,0,0.8)_100%)] opacity-90" />
    
    <!-- Пульсирующие пятна (циан + пурпур) -->
    <div bind:this={glow1} class="absolute w-[700px] h-[700px] bg-cyan-500/10 blur-[140px] opacity-0 pointer-events-none" style="top: 50%; left: 50%; transform: translate(-50%, -50%);" />
    <div bind:this={glow2} class="absolute w-[450px] h-[450px] bg-fuchsia-500/10 blur-[140px] opacity-0 pointer-events-none" style="top: 30%; left: 20%;" />

    <!-- Сканирующая линия (циан) -->
    <div bind:this={scanLine} class="absolute left-0 w-full h-px bg-gradient-to-r from-transparent via-cyan-400/20 to-transparent pointer-events-none" style="top: -5%; opacity: 0;" />

    <div bind:this={splashContent} class="relative z-10 flex flex-col items-center gap-12">
      <div class="relative">
        <!-- Свечение вокруг логотипа -->
        <div bind:this={logoGlow} class="absolute inset-[-10px] rounded-full bg-gradient-to-br from-cyan-400 via-fuchsia-500 to-cyan-400 blur-[56px] opacity-0" />
        <svg class="relative w-32 h-32" viewBox="0 0 120 120">
          <defs>
            <linearGradient id="g" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#22d3ee" />   <!-- cyan-400 -->
              <stop offset="50%" stop-color="#c084fc" />  <!-- purple-400 -->
              <stop offset="100%" stop-color="#f472b6" /> <!-- pink-400 -->
            </linearGradient>
          </defs>
          <path bind:this={bracketL} d="M48 22 Q36 22 36 34 L36 46 Q36 54 24 54 Q36 54 36 62 L36 74 Q36 86 48 86" fill="none" stroke="url(#g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="150" stroke-dashoffset="150" style="opacity:0" />
          <path bind:this={bracketR} d="M72 22 Q84 22 84 34 L84 46 Q84 54 96 54 Q84 54 84 62 L84 74 Q84 86 72 86" fill="none" stroke="url(#g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="150" stroke-dashoffset="150" style="opacity:0" />
        </svg>
      </div>

      <!-- Заголовок с динамической неоновой подсветкой -->
      <h1 bind:this={title} class="text-5xl font-bold tracking-[0.35em] uppercase">
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">F</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">O</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">R</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">M</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">A</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">T</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">O</span>
      </h1>

      <div bind:this={subtitle} class="flex flex-col items-center gap-4 text-center opacity-0">
        <p class="text-sm text-white/70 tracking-[0.5em] uppercase font-light">
          Universal Data Converter
        </p>
        
        <div class="w-48 h-px bg-gradient-to-r from-transparent via-fuchsia-400/40 to-transparent" />
        
        <span class="text-xs text-white/40 font-mono tracking-[0.4em]">
          v0.1.0
        </span>
        
        <div class="w-32 h-px bg-gradient-to-r from-transparent via-cyan-400/20 to-transparent" />
      </div>

      <div bind:this={progressWrapper} class="w-72 h-px bg-white/5 rounded-full overflow-hidden opacity-0">
        <div bind:this={progressBar} class="h-full bg-gradient-to-r from-cyan-400 via-fuchsia-500 to-rose-400 rounded-full origin-left" style="transform:scaleX(0)" />
      </div>
    </div>
  </div>
{/if}