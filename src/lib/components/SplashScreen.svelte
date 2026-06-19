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

  let isFadingOut = $state(false);
  let animationIds: any[] = [];

  // Функция для плавного fade-out
  async function performSmoothFadeOut() {
    if (isFadingOut || !container || !splashContent || !title) return;
    isFadingOut = true;

    // Останавливаем все текущие бесконечные анимации
    animationIds.forEach(anim => {
      try { anim?.stop?.(); } catch(e) {}
    });
    animationIds = [];

    const letters = title.querySelectorAll('.letter');

    // Собираем все анимации fade-out (без указания начального значения — 
    // Motion One сам возьмёт текущее значение элемента)
    const fadeOutAnimations: Promise<any>[] = [];

    // Буквы заголовка исчезают по одной
    letters.forEach((letter, i) => {
      fadeOutAnimations.push(
        animate(letter as HTMLElement, 
          { opacity: 0 }, 
          { duration: 0.6, delay: i * 0.06, easing: 'ease-out' }
        ).finished
      );
    });

    // Подзаголовок
    if (subtitle) {
      fadeOutAnimations.push(
        animate(subtitle, { opacity: 0 }, { duration: 0.6, delay: 0.2, easing: 'ease-out' }).finished
      );
    }

    // Прогресс-бар
    if (progressWrapper) {
      fadeOutAnimations.push(
        animate(progressWrapper, { opacity: 0 }, { duration: 0.6, delay: 0.1, easing: 'ease-out' }).finished
      );
    }

    // SVG логотип (весь элемент)
    const logoSvg = logoGlow?.parentElement;
    if (logoSvg) {
      fadeOutAnimations.push(
        animate(logoSvg, { opacity: 0 }, { duration: 0.6, delay: 0.1, easing: 'ease-out' }).finished
      );
    }

    // Свечение логотипа
    if (logoGlow) {
      fadeOutAnimations.push(
        animate(logoGlow, { opacity: 0 }, { duration: 0.6, delay: 0.1, easing: 'ease-out' }).finished
      );
    }

    // Фоновые свечения
    if (glow1) {
      fadeOutAnimations.push(
        animate(glow1, { opacity: 0 }, { duration: 0.8, easing: 'ease-out' }).finished
      );
    }
    if (glow2) {
      fadeOutAnimations.push(
        animate(glow2, { opacity: 0 }, { duration: 0.8, delay: 0.1, easing: 'ease-out' }).finished
      );
    }

    // Сканирующая линия
    if (scanLine) {
      fadeOutAnimations.push(
        animate(scanLine, { opacity: 0 }, { duration: 0.6, easing: 'ease-out' }).finished
      );
    }

    // Виньетка
    if (overlay) {
      fadeOutAnimations.push(
        animate(overlay, { opacity: 0 }, { duration: 0.8, easing: 'ease-out' }).finished
      );
    }

    // Основной контент
    fadeOutAnimations.push(
      animate(splashContent, { opacity: 0 }, { duration: 0.6, easing: 'ease-out' }).finished
    );

    // Ждём все анимации
    await Promise.all(fadeOutAnimations);

    await new Promise(r => setTimeout(r, 300));
    
    // Финальное исчезновение контейнера
    await animate(container, { opacity: 0 }, { duration: 0.8, easing: 'ease-out' }).finished;
    await new Promise(r => setTimeout(r, 100));

    visible = false;
    onComplete();
  }

  // Обработчик кликов
  function handleClick(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    performSmoothFadeOut();
  }

  // Блокировка правой кнопки
  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
  }

  onMount(async () => {
    // Блокируем правый клик на всём документе пока сплеш активен
    document.addEventListener('contextmenu', handleContextMenu);

    // Фоновые свечения – вход и вечная пульсация
    async function startGlowPulse() {
      if (glow1) {
        const anim1 = await animate(glow1, { opacity: [0, 0.35] }, { duration: 2.5, easing: 'ease-out' });
        animationIds.push(anim1);
        await anim1.finished;
        if (!isFadingOut) {
          const anim2 = animate(glow1, { opacity: [0.25, 0.45] }, { duration: 2, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' });
          animationIds.push(anim2);
        }
      }
      if (glow2) {
        const anim3 = await animate(glow2, { opacity: [0, 0.3] }, { duration: 3, delay: 0.3, easing: 'ease-out' });
        animationIds.push(anim3);
        await anim3.finished;
        if (!isFadingOut) {
          const anim4 = animate(glow2, { opacity: [0.15, 0.35] }, { duration: 2.5, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' });
          animationIds.push(anim4);
        }
      }
    }
    startGlowPulse();

    // Логотип – вращение + мягкий scale
    if (logoGlow && !isFadingOut) {
      const anim5 = animate(logoGlow, { rotate: 360 }, { duration: 35, easing: 'linear', repeat: Infinity });
      const anim6 = animate(logoGlow, { opacity: [0, 0.45] }, { duration: 2.5, easing: 'ease-out' });
      const anim7 = animate(logoGlow, { scale: [1, 1.08] }, { duration: 4, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' });
      animationIds.push(anim5, anim6, anim7);
    }

    // Сканирующая линия
    async function startScanLine() {
      await new Promise(r => setTimeout(r, 3500));
      if (scanLine && !isFadingOut) {
        const anim8 = animate(scanLine, { top: ['-5%', '105%'] }, { duration: 5, repeat: Infinity, easing: 'linear' });
        const anim9 = animate(scanLine, { opacity: [0, 0.3, 0] }, { duration: 5, repeat: Infinity, easing: 'linear' });
        animationIds.push(anim8, anim9);
      }
    }
    startScanLine();

    // Основная анимация – скобки, заголовок, подзаголовок, прогресс
    if (bracketL && bracketR && !isFadingOut) {
      const anim10 = animate(bracketL, { opacity: [0, 1] }, { duration: 0.6, easing: 'ease-out' });
      const anim11 = animate(bracketR, { opacity: [0, 1] }, { duration: 0.6, easing: 'ease-out' });
      animationIds.push(anim10, anim11);

      const anim12 = animate(bracketL, { strokeDashoffset: [150, 0] }, { duration: 3, easing: 'ease-in-out' });
      const anim13 = animate(bracketR, { strokeDashoffset: [150, 0] }, { duration: 3, easing: 'ease-in-out' });
      animationIds.push(anim12, anim13);

      await Promise.all([anim12.finished, anim13.finished]);
    }

    if (title && !isFadingOut) {
      const letters = title.querySelectorAll('.letter');
      const anim14 = animate(letters,
        { opacity: [0, 1] },
        { delay: stagger(0.08), duration: 0.8, easing: [0.34, 1.56, 0.64, 1] }
      );
      animationIds.push(anim14);
      await anim14.finished;
    }

    if (subtitle && !isFadingOut) {
      const anim15 = animate(subtitle,
        { opacity: [0, 1], transform: ['translateY(30px)', 'translateY(0)'] },
        { duration: 1.2, easing: [0.34, 1.56, 0.64, 1] }
      );
      animationIds.push(anim15);
      await anim15.finished;
    }

    if (progressWrapper && progressBar && !isFadingOut) {
      const anim16 = animate(progressWrapper, { opacity: [0, 1] }, { duration: 0.8, easing: 'ease-out' });
      const anim17 = animate(progressBar,
        { transform: ['scaleX(0)', 'scaleX(1)'] },
        { duration: 3.5, easing: [0.4, 0, 0.2, 1] }
      );
      animationIds.push(anim16, anim17);
      await anim17.finished;
    }

    if (!isFadingOut) {
      await new Promise(r => setTimeout(r, 2000));
    }

    // Если не было принудительного fade-out, запускаем обычный
    if (!isFadingOut) {
      await performSmoothFadeOut();
    }

    // Очистка обработчиков
    document.removeEventListener('contextmenu', handleContextMenu);
  });
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    bind:this={container} 
    class="fixed inset-0 z-[9999] flex items-center justify-center bg-[#0a0a0f] overflow-hidden select-none cursor-pointer"
    on:click={handleClick}
    on:contextmenu={handleContextMenu}
  >
    
    <!-- Тёмная виньетка -->
    <div bind:this={overlay} class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_30%,rgba(0,0,0,0.8)_100%)] opacity-90" />
    
    <!-- Пульсирующие пятна (циан + пурпур) -->
    <div bind:this={glow1} class="absolute w-[700px] h-[700px] bg-cyan-500/10 blur-[140px] opacity-0 pointer-events-none" style="top: 50%; left: 50%; transform: translate(-50%, -50%);" />
    <div bind:this={glow2} class="absolute w-[450px] h-[450px] bg-fuchsia-500/10 blur-[140px] opacity-0 pointer-events-none" style="top: 30%; left: 20%;" />

    <!-- Сканирующая линия (циан) -->
    <div bind:this={scanLine} class="absolute left-0 w-full h-px bg-gradient-to-r from-transparent via-cyan-400/20 to-transparent pointer-events-none" style="top: -5%; opacity: 0;" />

    <div bind:this={splashContent} class="relative z-10 flex flex-col items-center gap-12 select-none pointer-events-none">
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
      <h1 bind:this={title} class="text-5xl font-bold tracking-[0.35em] uppercase select-none">
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">F</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">O</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">R</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">M</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">A</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">T</span>
        <span class="letter inline-block bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-rose-400 bg-clip-text text-transparent" style="opacity:0;">O</span>
      </h1>

      <div bind:this={subtitle} class="flex flex-col items-center gap-4 text-center opacity-0 select-none">
        <p class="text-sm text-white/70 tracking-[0.5em] uppercase font-light">
          Universal Data Converter
        </p>
        
        <div class="w-48 h-px bg-gradient-to-r from-transparent via-fuchsia-400/40 to-transparent" />
        
        <span class="text-xs text-white/40 font-mono tracking-[0.4em]">
          v0.1.0
        </span>
        
        <div class="w-32 h-px bg-gradient-to-r from-transparent via-cyan-400/20 to-transparent" />
      </div>

      <div bind:this={progressWrapper} class="w-72 h-px bg-white/5 rounded-full overflow-hidden opacity-0 select-none">
        <div bind:this={progressBar} class="h-full bg-gradient-to-r from-cyan-400 via-fuchsia-500 to-rose-400 rounded-full origin-left" style="transform:scaleX(0)" />
      </div>
    </div>
  </div>
{/if}