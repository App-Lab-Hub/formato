<script lang="ts">
  import { onMount } from 'svelte';
  import { animate, stagger } from '@motionone/dom';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import "$lib/styles/splashScreen.css";

  let { onComplete = () => {} } = $props();
  let visible = $state(true);
  let isFadingOut = false;
  
  const refs: Record<string, Element | null> = {};
  
  const animationIds: any[] = [];
  let letters: Element[] = [];

  const FADE = { duration: 0.6, easing: 'ease-out' } as const;

  function stopAll() {
    for (let i = 0; i < animationIds.length; i++) {
      try { animationIds[i]?.stop?.(); } catch {}
    }
    animationIds.length = 0;
  }

  async function fadeOut() {
    if (isFadingOut || !refs.container || !refs.splashContent || !refs.title) return;
    isFadingOut = true;

    getCurrentWindow().setCursorVisible(true).catch(() => {});
    stopAll();

    if (!letters.length && refs.title) {
      letters = Array.from((refs.title as HTMLElement).querySelectorAll('.letter'));
    }

    const targets = [];
    
    for (let i = 0; i < letters.length; i++) {
      targets.push(animate(letters[i] as HTMLElement, { opacity: 0 }, { ...FADE, delay: i * 0.06 }));
    }
    
    if (refs.subtitle) targets.push(animate(refs.subtitle as HTMLElement, { opacity: 0 }, { ...FADE, delay: 0.2 }));
    if (refs.progressWrapper) targets.push(animate(refs.progressWrapper as HTMLElement, { opacity: 0 }, { ...FADE, delay: 0.1 }));
    if (refs.logoGlow) targets.push(animate(refs.logoGlow as HTMLElement, { opacity: 0 }, { ...FADE, delay: 0.1 }));
    if (refs.logoWrapper) targets.push(animate(refs.logoWrapper as HTMLElement, { opacity: 0 }, { ...FADE, delay: 0.1 }));
    if (refs.scanLine) targets.push(animate(refs.scanLine as HTMLElement, { opacity: 0 }, FADE));
    if (refs.overlay) targets.push(animate(refs.overlay as HTMLElement, { opacity: 0 }, { duration: 0.8, easing: 'ease-out' }));
    if (refs.splashContent) targets.push(animate(refs.splashContent as HTMLElement, { opacity: 0 }, FADE));

    await Promise.all(targets.map(a => a.finished));
    await new Promise(r => setTimeout(r, 300));
    await animate(refs.container as HTMLElement, { opacity: 0 }, { duration: 0.8, easing: 'ease-out' }).finished;
    await new Promise(r => setTimeout(r, 100));

    visible = false;
    onComplete();
  }

  function handleInput(e: Event) {
    e.preventDefault();
    e.stopPropagation();
    fadeOut();
  }

  onMount(() => {
    getCurrentWindow().setCursorVisible(false).catch(() => {
      document.body.style.cursor = 'none';
      document.documentElement.style.cursor = 'none';
    });

    if (refs.title) letters = Array.from((refs.title as HTMLElement).querySelectorAll('.letter'));

    // Вспышка за скобками — один элемент
if (refs.logoGlow) {
  // 1. Плавное появление до полной яркости
  const fadeIn = animate(refs.logoGlow as HTMLElement, 
    { opacity: [0, 1.0] }, 
    { duration: 2.0, easing: 'ease-out' }
  );
  
  // 2. После появления — пульсация opacity + scale
  fadeIn.finished.then(() => {
    if (!isFadingOut) {
      animationIds.push(
        animate(refs.logoGlow as HTMLElement, 
          { opacity: [1.0, 0.6, 1.0] }, 
          { duration: 2.5, repeat: Infinity, easing: 'ease-in-out' }
        ),
        animate(refs.logoGlow as HTMLElement, 
          { scale: [1, 1.3, 1] }, 
          { duration: 4, repeat: Infinity, easing: 'ease-in-out' }
        )
      );
    }
  });
  
  animationIds.push(fadeIn);
}
    setTimeout(() => {
      if (refs.scanLine && !isFadingOut) {
        animationIds.push(
          animate(refs.scanLine as HTMLElement, { top: ['-5%', '105%'] }, { duration: 5, repeat: Infinity, easing: 'linear' }),
          animate(refs.scanLine as HTMLElement, { opacity: [0, 0.3, 0] }, { duration: 5, repeat: Infinity, easing: 'linear' })
        );
      }
    }, 3500);

    async function runSequence() {
      if (!refs.bracketL || !refs.bracketR) return;

      animationIds.push(
        animate(refs.bracketL as SVGPathElement, { opacity: [0, 1] }, { duration: 0.6, easing: 'ease-out' }),
        animate(refs.bracketR as SVGPathElement, { opacity: [0, 1] }, { duration: 0.6, easing: 'ease-out' })
      );

      await Promise.all([
        animate(refs.bracketL as SVGPathElement, { strokeDashoffset: [150, 0] }, { duration: 3, easing: 'ease-in-out' }).finished,
        animate(refs.bracketR as SVGPathElement, { strokeDashoffset: [150, 0] }, { duration: 3, easing: 'ease-in-out' }).finished,
      ]);

      if (isFadingOut) return;

      if (refs.title && letters.length) {
        await animate(letters as HTMLElement[], { opacity: [0, 1] }, { 
          delay: stagger(0.08), duration: 0.8, easing: [0.34, 1.56, 0.64, 1] 
        }).finished;
      }

      if (isFadingOut || !refs.subtitle) return;
      await animate(refs.subtitle as HTMLElement, { opacity: [0, 1], transform: ['translateY(30px)', 'translateY(0)'] }, { 
        duration: 1.2, easing: [0.34, 1.56, 0.64, 1] 
      }).finished;

      if (isFadingOut || !refs.progressWrapper || !refs.progressBar) return;
      animationIds.push(animate(refs.progressWrapper as HTMLElement, { opacity: [0, 1] }, { duration: 0.8, easing: 'ease-out' }));
      await animate(refs.progressBar as HTMLElement, { transform: ['scaleX(0)', 'scaleX(1)'] }, { 
        duration: 3.5, easing: [0.4, 0, 0.2, 1] 
      }).finished;

      if (!isFadingOut) await new Promise(r => setTimeout(r, 2000));
      if (!isFadingOut) await fadeOut();
    }

    runSequence();

    return () => {
      stopAll();
      getCurrentWindow().setCursorVisible(true).catch(() => {
        document.body.style.cursor = '';
        document.documentElement.style.cursor = '';
      });
    };
  });
</script>

<svelte:window
  on:keydown={handleInput}
  on:mousedown={handleInput}
  on:touchstart={handleInput}
  on:wheel={handleInput}
  on:contextmenu|preventDefault={handleInput}
/>
{#if visible}
  <div bind:this={refs.container} class="splash-container">
    <div bind:this={refs.overlay} class="splash-overlay" />
    <div bind:this={refs.scanLine} class="splash-scanline" />

    <div bind:this={refs.splashContent} class="splash-content">
      <div bind:this={refs.logoWrapper} class="logo-wrapper">
        <svg class="logo-svg" viewBox="0 0 120 120">
          <defs>
            <linearGradient id="g" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#22d3ee"/>
              <stop offset="50%" stop-color="#c084fc"/>
              <stop offset="100%" stop-color="#f472b6"/>
            </linearGradient>
          </defs>
          <path bind:this={refs.bracketL} d="M48 22 Q36 22 36 34 L36 46 Q36 54 24 54 Q36 54 36 62 L36 74 Q36 86 48 86" fill="none" stroke="url(#g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="150" stroke-dashoffset="150" style="opacity:0"/>
          <path bind:this={refs.bracketR} d="M72 22 Q84 22 84 34 L84 46 Q84 54 96 54 Q84 54 84 62 L84 74 Q84 86 72 86" fill="none" stroke="url(#g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="150" stroke-dashoffset="150" style="opacity:0"/>
        </svg>
        <!-- Свечение теперь ПОСЛЕ SVG в DOM, но визуально за ним через z-index -->
        <div bind:this={refs.logoGlow} class="logo-glow" />
      </div>

      <h1 bind:this={refs.title} class="splash-title">
        <span class="letter">F</span>
        <span class="letter">O</span>
        <span class="letter">R</span>
        <span class="letter">M</span>
        <span class="letter">A</span>
        <span class="letter">T</span>
        <span class="letter">O</span>
      </h1>

      <div bind:this={refs.subtitle} class="splash-subtitle">
        <p class="subtitle-text">Universal Data Converter</p>
        <div class="divider divider-purple" />
        <span class="version-text">v0.1.0</span>
        <div class="divider divider-cyan" />
      </div>

      <div bind:this={refs.progressWrapper} class="progress-wrapper">
        <div bind:this={refs.progressBar} class="progress-bar" />
      </div>
    </div>
  </div>
{/if}