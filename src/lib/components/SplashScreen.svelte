<script lang="ts">
  import { onMount } from 'svelte';
  import { animate, stagger } from '@motionone/dom';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let { onComplete = () => {} } = $props();
  let visible = $state(true);
  let isFadingOut = false;
  
  // Используем refs вместо $ (зарезервировано)
  const refs: Record<string, Element | null> = {};
  
  const animationIds: any[] = [];
  let letters: Element[] = [];

  const FADE = { duration: 0.6, easing: 'ease-out' } as const;
  const GLOW_FADE = { duration: 0.8, easing: 'ease-out' } as const;

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
    if (refs.logoGlow?.parentElement) targets.push(animate(refs.logoGlow.parentElement, { opacity: 0 }, { ...FADE, delay: 0.1 }));
    if (refs.logoGlow) targets.push(animate(refs.logoGlow as HTMLElement, { opacity: 0 }, { ...FADE, delay: 0.1 }));
    if (refs.glow1) targets.push(animate(refs.glow1 as HTMLElement, { opacity: 0 }, GLOW_FADE));
    if (refs.glow2) targets.push(animate(refs.glow2 as HTMLElement, { opacity: 0 }, { ...GLOW_FADE, delay: 0.1 }));
    if (refs.scanLine) targets.push(animate(refs.scanLine as HTMLElement, { opacity: 0 }, FADE));
    if (refs.overlay) targets.push(animate(refs.overlay as HTMLElement, { opacity: 0 }, GLOW_FADE));
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

    if (refs.glow1) {
      animate(refs.glow1 as HTMLElement, { opacity: [0, 0.35] }, { duration: 2.5, easing: 'ease-out' })
        .finished.then(() => {
          if (!isFadingOut) animationIds.push(animate(refs.glow1 as HTMLElement, { opacity: [0.25, 0.45] }, { duration: 2, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' }));
        });
    }
    if (refs.glow2) {
      animate(refs.glow2 as HTMLElement, { opacity: [0, 0.3] }, { duration: 3, delay: 0.3, easing: 'ease-out' })
        .finished.then(() => {
          if (!isFadingOut) animationIds.push(animate(refs.glow2 as HTMLElement, { opacity: [0.15, 0.35] }, { duration: 2.5, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' }));
        });
    }

    if (refs.logoGlow) {
      animationIds.push(
        animate(refs.logoGlow as HTMLElement, { rotate: 360 }, { duration: 35, easing: 'linear', repeat: Infinity }),
        animate(refs.logoGlow as HTMLElement, { opacity: [0, 0.45] }, { duration: 2.5, easing: 'ease-out' }),
        animate(refs.logoGlow as HTMLElement, { scale: [1, 1.08] }, { duration: 4, repeat: Infinity, direction: 'alternate', easing: 'ease-in-out' })
      );
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
  <div bind:this={refs.container} class="fixed inset-0 z-[9999] flex items-center justify-center overflow-hidden select-none" style="background:#0a0a0f">
    <div bind:this={refs.overlay} class="absolute inset-0 opacity-90" style="background:radial-gradient(ellipse at center,transparent 30%,rgba(0,0,0,0.8) 100%)" />
    <div bind:this={refs.glow1} class="absolute w-[700px] h-[700px] opacity-0 pointer-events-none" style="background:rgba(6,182,212,0.1);filter:blur(140px);top:50%;left:50%;transform:translate(-50%,-50%)" />
    <div bind:this={refs.glow2} class="absolute w-[450px] h-[450px] opacity-0 pointer-events-none" style="background:rgba(217,70,239,0.1);filter:blur(140px);top:30%;left:20%" />
    <div bind:this={refs.scanLine} class="absolute left-0 w-full h-px pointer-events-none" style="background:linear-gradient(to right,transparent,rgba(34,211,238,0.2),transparent);top:-5%;opacity:0" />

    <div bind:this={refs.splashContent} class="relative z-10 flex flex-col items-center gap-12 select-none pointer-events-none">
      <div class="relative">
        <div bind:this={refs.logoGlow} class="absolute inset-[-10px] rounded-full opacity-0" style="background:linear-gradient(to bottom right,#22d3ee,#c084fc,#22d3ee);filter:blur(56px)" />
        <svg class="relative w-32 h-32" viewBox="0 0 120 120">
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
      </div>

      <h1 bind:this={refs.title} class="text-5xl font-bold tracking-[0.35em] uppercase select-none">
        <span class="letter inline-block" style="opacity:0;background:linear-gradient(to right,#22d3ee,#c084fc,#f472b6);-webkit-background-clip:text;-webkit-text-fill-color:transparent">F</span>
        <span class="letter inline-block" style="opacity:0;background:linear-gradient(to right,#22d3ee,#c084fc,#f472b6);-webkit-background-clip:text;-webkit-text-fill-color:transparent">O</span>
        <span class="letter inline-block" style="opacity:0;background:linear-gradient(to right,#22d3ee,#c084fc,#f472b6);-webkit-background-clip:text;-webkit-text-fill-color:transparent">R</span>
        <span class="letter inline-block" style="opacity:0;background:linear-gradient(to right,#22d3ee,#c084fc,#f472b6);-webkit-background-clip:text;-webkit-text-fill-color:transparent">M</span>
        <span class="letter inline-block" style="opacity:0;background:linear-gradient(to right,#22d3ee,#c084fc,#f472b6);-webkit-background-clip:text;-webkit-text-fill-color:transparent">A</span>
        <span class="letter inline-block" style="opacity:0;background:linear-gradient(to right,#22d3ee,#c084fc,#f472b6);-webkit-background-clip:text;-webkit-text-fill-color:transparent">T</span>
        <span class="letter inline-block" style="opacity:0;background:linear-gradient(to right,#22d3ee,#c084fc,#f472b6);-webkit-background-clip:text;-webkit-text-fill-color:transparent">O</span>
      </h1>

      <div bind:this={refs.subtitle} class="flex flex-col items-center gap-4 text-center opacity-0 select-none">
        <p class="text-sm text-white/70 tracking-[0.5em] uppercase font-light">Universal Data Converter</p>
        <div class="w-48 h-px" style="background:linear-gradient(to right,transparent,rgba(192,132,252,0.4),transparent)"/>
        <span class="text-xs text-white/40 font-mono tracking-[0.4em]">v0.1.0</span>
        <div class="w-32 h-px" style="background:linear-gradient(to right,transparent,rgba(34,211,238,0.2),transparent)"/>
      </div>

      <div bind:this={refs.progressWrapper} class="w-72 h-px rounded-full overflow-hidden opacity-0 select-none" style="background:rgba(255,255,255,0.05)">
        <div bind:this={refs.progressBar} class="h-full rounded-full origin-left" style="transform:scaleX(0);background:linear-gradient(to right,#22d3ee,#c084fc,#f472b6)"/>
      </div>
    </div>
  </div>
{/if}