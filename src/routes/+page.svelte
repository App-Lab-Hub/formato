<!-- src/routes/+page.svelte -->
<script lang="ts">
  import { Splide, SplideSlide } from '@splidejs/svelte-splide';
  // @ts-ignore
  import '@splidejs/svelte-splide/css/sea-green';
  import '$lib/styles/splide.css';
  import { getFormats } from '$lib/data/formats';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { Settings, Info } from 'lucide-svelte';

  const SPLIDE_INDEX_KEY = 'splide_active_index';

  const splideOptions = {
    type: 'loop' as const,
    perPage: 3,
    perMove: 1,
    gap: '2rem',
    pagination: true,
    arrows: false,
    wheel: true,
    waitForTransition: false,
    wheelSleep: 100,
    wheelMinThreshold: 1,
    focus: 'center' as const,
    trimSpace: true,
    omitEnd: true,
    speed: 300,
    breakpoints: {
      1200: { perPage: 3, gap: '1.75rem' },
      900:  { perPage: 2, gap: '1.5rem' },
      600:  { perPage: 1, gap: '1.25rem' },
    }
  };

  let formats = getFormats();
  let splideInstance: any = null;
  let isRestoring = false;

  const formatMap = new Map<number, any>();
  formats.forEach((f, i) => formatMap.set(i, f));

  function normalizeIndex(index: number): number {
    if (index < 0) {
      return (index % formats.length + formats.length) % formats.length;
    } else {
      return index % formats.length;
    }
  }

  function goToConvert(formatId: string, index: number) {
    if (browser && index >= 0) {
      sessionStorage.setItem(SPLIDE_INDEX_KEY, String(index));
    }
    goto(`/convert/${formatId}`);
  }

  function restoreSplidePosition() {
    if (!splideInstance || isRestoring) return;
    
    try {
      const savedIndex = sessionStorage.getItem(SPLIDE_INDEX_KEY);
      if (savedIndex) {
        const index = parseInt(savedIndex);
        if (index >= 0 && index < formats.length) {
          isRestoring = true;
          splideInstance.go(index, 0);
          isRestoring = false;
        }
      }
    } catch (e) {
      isRestoring = false;
    }
  }

  onMount(() => {
    const navigationType = performance?.navigation?.type;
    if (navigationType === 1) {
      sessionStorage.removeItem(SPLIDE_INDEX_KEY);
    }
  });
</script>

<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-full">
    <main class="flex flex-col items-center gap-3 px-8 py-16">

      <!-- Шапка с навигацией -->
      <div class="w-full max-w-[1700px] flex justify-end gap-4 px-4 mb-4">
        <button 
          onclick={() => goto('/about')}
          class="flex items-center gap-2 text-muted-foreground/60 hover:text-primary transition-colors text-sm"
        >
          <Info class="h-4 w-4" />
          <span>О нас</span>
        </button>
        <button 
          onclick={() => goto('/settings')}
          class="flex items-center gap-2 text-muted-foreground/60 hover:text-primary transition-colors text-sm"
        >
          <Settings class="h-4 w-4" />
          <span>Настройки</span>
        </button>
      </div>

      <img
        src="/favicon.svg"
        alt="Formato logo"
        class="w-24 h-24 sm:w-28 sm:h-28"
      />

      <div class="text-center max-w-2xl">
        <p class="text-sm sm:text-base text-muted-foreground/60 tracking-wide">
          Универсальный конвертер данных — быстро, локально, без лишнего
        </p>
        <div class="mt-5 mb-2 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-border to-transparent"></div>
        <h2 class="text-lg sm:text-xl lg:text-2xl font-light tracking-[0.3em] uppercase bg-gradient-to-r from-primary to-primary/50 bg-clip-text text-transparent">
          Convert from
        </h2>
      </div>

      {#if formats.length > 0}
        <Splide
          options={splideOptions}
          aria-label="Выбор формата"
          class="w-full max-w-[1700px] mx-auto"
          on:mounted={(e) => {
            splideInstance = e.detail.splide;
            
            if (splideInstance) {
              splideInstance.on('click', (Slide: any) => {
                const slideIndex = Slide.index;
                const realIndex = normalizeIndex(slideIndex);
                const format = formatMap.get(realIndex);
                
                if (format) {
                  goToConvert(format.id, realIndex);
                }
              });
            }
            
            restoreSplidePosition();
          }}
        >
          {#each formats as format, index}
            {@const Icon = format.icon}
            <SplideSlide>
              <div
                class="group block rounded-2xl border-2 border-border bg-card p-10 transition-all duration-300 hover:scale-[1.05] {format.borderHover} {format.glow} hover:shadow-2xl hover:-translate-y-2 mx-auto cursor-pointer"
                style="max-width: 320px;"
              >
                <div class="flex flex-col items-center gap-6 text-center">
                  <div class="relative rounded-3xl bg-gradient-to-br p-8 {format.color}">
                    <div class="absolute inset-0 rounded-3xl bg-gradient-to-br opacity-30 blur-2xl {format.color}"></div>
                    <Icon class="relative h-16 w-16 {format.textColor}" />
                  </div>
                  <div class="w-full min-w-0">
                    <h3 class="text-xl lg:text-2xl font-bold">{format.name}</h3>
                    <p class="mt-2 text-sm lg:text-base text-muted-foreground truncate">{format.description}</p>
                  </div>
                </div>
              </div>
            </SplideSlide>
          {/each}
        </Splide>
      {:else}
        <div class="text-center text-muted-foreground py-10">
          Нет доступных форматов
        </div>
      {/if}

      <!-- Нижняя навигация -->
      <div class="mt-8 flex gap-6 text-xs text-muted-foreground/40">
        <button 
          onclick={() => goto('/about')}
          class="hover:text-primary/60 transition-colors"
        >
          О приложении
        </button>
        <button 
          onclick={() => goto('/settings')}
          class="hover:text-primary/60 transition-colors"
        >
          Настройки
        </button>
        <span>v0.1.0</span>
      </div>

    </main>
  </div>
</ScrollContainer>