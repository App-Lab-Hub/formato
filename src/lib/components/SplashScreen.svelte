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

    if (refs.logoGlow) {
      const fadeIn = animate(refs.logoGlow as HTMLElement, 
        { opacity: [0, 1.0] }, 
        { duration: 2.0, easing: 'ease-out' }
      );
      
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
          animate(refs.scanLine as HTMLElement, 
            { top: ['-5%', '105%'] }, 
            { duration: 6, repeat: Infinity, easing: 'linear' }
          ),
          animate(refs.scanLine as HTMLElement, 
            { opacity: [0, 0.9, 0] }, 
            { duration: 6, repeat: Infinity, easing: 'linear' }
          )
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

      // Стрелки: сначала запускаем вращение (невидимые), потом показываем одновременно
      const arrowParts = document.querySelectorAll('.arrow-part');
      if (arrowParts.length) {
        arrowParts.forEach(el => {
          (el as HTMLElement).style.transformOrigin = '60px 54px';
        });

        // Сначала запускаем вращение (стрелки еще невидемые)
        if (!isFadingOut) {
          animationIds.push(
            animate(arrowParts[0] as SVGElement, { 
              rotate: [360, 0]
            }, { 
              duration: 10, 
              repeat: Infinity, 
              easing: 'linear'
            })
          );
          animationIds.push(
            animate(arrowParts[1] as SVGElement, { 
              rotate: [360, 0]
            }, { 
              duration: 10, 
              repeat: Infinity, 
              easing: 'linear'
            })
          );
        }

        // Показываем стрелки одновременно (без stagger)
        await animate(arrowParts, { opacity: [0, 1] }, { 
          duration: 0.8, 
          easing: 'ease-out' 
        }).finished;
      }

      if (isFadingOut || !refs.title || !letters.length) return;
      await animate(letters as HTMLElement[], { opacity: [0, 1] }, { 
        delay: stagger(0.08), duration: 0.8, easing: [0.34, 1.56, 0.64, 1] 
      }).finished;

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
    <div bind:this={refs.overlay} class="splash-overlay" ></div>
    <div bind:this={refs.scanLine} class="splash-scanline" ></div>

    <div bind:this={refs.splashContent} class="splash-content">
      <div bind:this={refs.logoWrapper} class="logo-wrapper">
        <svg class="logo-svg" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <linearGradient id="g" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#22d3ee"/>
              <stop offset="50%" stop-color="#c084fc"/>
              <stop offset="100%" stop-color="#f472b6"/>
            </linearGradient>
          </defs>
          
          <!-- Левая скобка -->
          <path bind:this={refs.bracketL} d="M48 22 Q36 22 36 34 L36 46 Q36 54 24 54 Q36 54 36 62 L36 74 Q36 86 48 86" fill="none" stroke="url(#g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="150" stroke-dashoffset="150" style="opacity:0"/>
          
          <!-- Правая скобка -->
          <path bind:this={refs.bracketR} d="M72 22 Q84 22 84 34 L84 46 Q84 54 96 54 Q84 54 84 62 L84 74 Q84 86 72 86" fill="none" stroke="url(#g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="150" stroke-dashoffset="150" style="opacity:0"/>
          
          <!-- Первая стрелка -->
          <g class="arrow-part" style="opacity:0; transform-origin: 60px 54px;">
            <path fill="url(#g)" d="M59.3755 66.45C54.5752 65.9956 50.4829 62.8543 48.7786 58.3158C48.2365 56.8746 47.9533 55.1647 48.007 53.6258C48.0314 52.8734 48.0998 52.6487 48.3391 52.4728C48.4807 52.3702 49.9799 52.0429 50.3022 52.0429C50.5464 52.0429 50.771 52.1748 50.9029 52.3946C50.981 52.5314 50.9908 52.6194 50.9761 52.9467C50.9322 53.6404 50.9566 54.9399 51.0201 55.3161C51.3521 57.4266 52.28 59.2586 53.7596 60.7389C55.2393 62.2192 57.0315 63.1279 59.1802 63.4845C59.6636 63.5627 61.1872 63.5627 61.7439 63.4796C63.2138 63.2647 64.6886 62.6882 65.8704 61.8674C66.2366 61.6134 66.8959 61.0809 66.8959 61.0418C66.8959 61.0272 66.6859 60.8024 66.432 60.5435C66.178 60.2846 65.9485 60.011 65.9192 59.9377C65.7971 59.6104 65.9095 59.2489 66.1878 59.0779C66.2708 59.029 66.7982 58.8434 67.3598 58.6626C67.9263 58.4867 69.0153 58.1301 69.7868 57.8761C70.5535 57.622 71.247 57.4168 71.3251 57.4168C71.6572 57.4168 71.9746 57.7735 71.9746 58.1497C71.9746 58.433 70.9637 63.4943 70.8661 63.7044C70.7831 63.8802 70.5682 64.0317 70.3338 64.0805C70.0847 64.1343 69.8357 63.9926 69.4011 63.5578C69.266 63.4259 69.1308 63.294 68.9957 63.1621C68.9274 63.2272 68.859 63.2924 68.7906 63.3575C68.0679 64.0512 66.7445 64.9111 65.6751 65.3801C63.6533 66.2741 61.4607 66.6503 59.3755 66.45Z"/>
          </g>
          
          <!-- Вторая стрелка -->
          <g class="arrow-part" style="opacity:0; transform-origin: 60px 54px;">
            <path fill="url(#g)" d="M70.4754 55.9219C70.0701 55.7851 69.9529 55.4871 70.031 54.7885C70.0896 54.212 70.0505 53.103 69.948 52.507C69.5866 50.4503 68.6881 48.7013 67.2377 47.2552C65.7581 45.7749 63.9659 44.8662 61.8172 44.5096C61.3337 44.4314 59.8101 44.4314 59.2534 44.5145C57.8958 44.7099 56.5139 45.2277 55.3956 45.9557C54.9707 46.2293 54.1015 46.9034 54.1015 46.9572C54.1015 46.967 54.3115 47.1917 54.5654 47.4506C54.8193 47.7095 55.0488 47.9831 55.0781 48.0564C55.2002 48.3837 55.083 48.7599 54.7998 48.9211C54.7265 48.9651 54.1698 49.1605 53.5643 49.351C52.9588 49.5464 51.86 49.9031 51.1177 50.1474C49.6381 50.6359 49.5599 50.6457 49.2621 50.3965C49.0814 50.2402 49.013 50.0838 49.0179 49.82C49.0179 49.5464 50.0385 44.5291 50.1313 44.3386C50.2143 44.1627 50.4292 44.0113 50.6587 43.9624C50.9322 43.9038 51.1226 44.0113 51.5621 44.4461C51.7021 44.5845 51.8421 44.7229 51.9821 44.8613C52.1351 44.7294 52.2881 44.5975 52.4411 44.4656C55.2295 42.0864 59.019 41.0507 62.6522 41.6858C67.4379 42.5163 71.3984 46.1902 72.5997 50.8948C72.9415 52.2383 73.0782 53.787 72.9562 54.8715C72.8829 55.4724 72.7413 55.5848 71.7744 55.7802C70.8075 55.9805 70.6854 55.9903 70.4754 55.9219Z"/>
          </g>
        </svg>
        <div bind:this={refs.logoGlow} class="logo-glow" ></div>
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
        <div class="divider divider-purple" ></div>
        <span class="version-text">v0.1.0</span>
        <div class="divider divider-cyan" ></div>
      </div>

      <div bind:this={refs.progressWrapper} class="progress-wrapper">
        <div bind:this={refs.progressBar} class="progress-bar" ></div>
      </div>
    </div>
  </div>
{/if}