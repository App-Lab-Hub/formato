<script lang="ts">
  import { onMount } from 'svelte';
  import { animate, stagger } from '@motionone/dom';

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

  onMount(async () => {
    if (glow1) animate(glow1, { opacity: [0.03, 0.12, 0.03] }, { duration: 6, easing: 'ease-in-out', repeat: Infinity });
    if (glow2) animate(glow2, { opacity: [0.03, 0.12, 0.03] }, { duration: 7, easing: 'ease-in-out', delay: 0.5, repeat: Infinity });
    if (logoGlow) {
      animate(logoGlow, { rotate: 360 }, { duration: 35, easing: 'linear', repeat: Infinity });
      animate(logoGlow, { opacity: [0.05, 0.15, 0.05] }, { duration: 6, easing: 'ease-in-out', repeat: Infinity });
    }

    if (bracketL && bracketR) {
      animate(bracketL, { opacity: [0, 1] }, { duration: 0.4, easing: 'ease-out' });
      animate(bracketR, { opacity: [0, 1] }, { duration: 0.4, easing: 'ease-out' });
      await Promise.all([
        animate(bracketL, { strokeDashoffset: [150, 0] }, { duration: 3, easing: 'linear' }).finished,
        animate(bracketR, { strokeDashoffset: [150, 0] }, { duration: 3, easing: 'linear' }).finished,
      ]);
    }

    if (title) {
      const letters = title.querySelectorAll('.letter');
      await animate(letters,
        { opacity: [0, 1], filter: ['blur(10px)', 'blur(0px)'], transform: ['translateY(30px)', 'translateY(0)'] },
        { delay: stagger(0.1), duration: 1.2, easing: [0.22, 0.61, 0.36, 1] }
      ).finished;
    }

  if (subtitle) {
    await animate(subtitle,
      { opacity: [0, 1], transform: ['translateY(24px)', 'translateY(0)'] },
      { duration: 1, easing: [0.22, 0.61, 0.36, 1] }
    ).finished;
  }

    if (progressWrapper && progressBar) {
      animate(progressWrapper, { opacity: [0, 1] }, { duration: 0.6, easing: 'ease-out' });
      await animate(progressBar,
        { transform: ['scaleX(0)', 'scaleX(1)'] },
        { duration: 3, easing: [0.4, 0, 0.2, 1] }
      ).finished;
    }

    await new Promise(r => setTimeout(r, 2000)); // 2 секунды

    if (container) {
      await animate(container,
        { opacity: [1, 0], filter: ['blur(0px)', 'blur(40px)'], scale: [1, 1.1] },
        { duration: 1, easing: [0.4, 0, 0.2, 1] }
      ).finished;
    }

    visible = false;
    onComplete();
  });
</script>

{#if visible}
  <div bind:this={container} class="fixed inset-0 z-[9999] flex items-center justify-center bg-[#060608] overflow-hidden select-none">
    <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_35%,rgba(0,0,0,0.7)_100%)] opacity-90" />
    <div bind:this={glow1} class="absolute w-[700px] h-[700px] bg-blue-500/[0.03] rounded-full blur-[140px] opacity-0" />
    <div bind:this={glow2} class="absolute w-[450px] h-[450px] bg-purple-500/[0.05] rounded-full blur-[140px] top-[30%] left-[20%] opacity-0" />

    <div class="relative z-10 flex flex-col items-center gap-12">
      <div class="relative">
        <div bind:this={logoGlow} class="absolute inset-[-10px] rounded-full bg-gradient-to-br from-blue-500 via-purple-500 to-blue-500 blur-[56px] opacity-0" />
        <svg class="relative w-32 h-32" viewBox="0 0 120 120">
          <defs>
            <linearGradient id="g" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#60a5fa" />
              <stop offset="50%" stop-color="#a78bfa" />
              <stop offset="100%" stop-color="#a855f7" />
            </linearGradient>
          </defs>
          <path bind:this={bracketL} d="M48 22 Q36 22 36 34 L36 46 Q36 54 24 54 Q36 54 36 62 L36 74 Q36 86 48 86" fill="none" stroke="url(#g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="150" stroke-dashoffset="150" style="opacity:0" />
          <path bind:this={bracketR} d="M72 22 Q84 22 84 34 L84 46 Q84 54 96 54 Q84 54 84 62 L84 74 Q84 86 72 86" fill="none" stroke="url(#g)" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="150" stroke-dashoffset="150" style="opacity:0" />
        </svg>
      </div>

      <h1 bind:this={title} class="text-5xl font-bold tracking-[0.35em] uppercase">
        <span class="letter inline-block bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent" style="opacity:0;filter:blur(8px)">F</span>
        <span class="letter inline-block bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent" style="opacity:0;filter:blur(8px)">O</span>
        <span class="letter inline-block bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent" style="opacity:0;filter:blur(8px)">R</span>
        <span class="letter inline-block bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent" style="opacity:0;filter:blur(8px)">M</span>
        <span class="letter inline-block bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent" style="opacity:0;filter:blur(8px)">A</span>
        <span class="letter inline-block bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent" style="opacity:0;filter:blur(8px)">T</span>
        <span class="letter inline-block bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent" style="opacity:0;filter:blur(8px)">O</span>
      </h1>

      <div bind:this={subtitle} class="flex flex-col items-center gap-2 text-center opacity-0">
        <p class="text-sm text-white/40 tracking-[0.35em] uppercase">Universal Data Converter</p>
        <span class="text-xs text-white/20 font-mono">v0.1.0</span>
      </div>

      <div bind:this={progressWrapper} class="w-72 h-px bg-white/5 rounded-full overflow-hidden opacity-0">
        <div bind:this={progressBar} class="h-full bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 rounded-full origin-left" style="transform:scaleX(0)" />
      </div>
    </div>
  </div>
{/if}