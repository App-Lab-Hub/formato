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
  import Header from '$lib/components/Header.svelte';
  import Footer from '$lib/components/Footer.svelte';

  const SPLIDE_INDEX_KEY = 'splide_active_index';

  const splideOptions = {
    type: 'loop' as const,
    perPage: 3,
    perMove: 1,
    gap: '2rem',
    padding: '2rem',
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
      1200: { perPage: 3, gap: '1.75rem', padding: '1.5rem' },
      900:  { perPage: 2, gap: '1.75rem', padding: '1.25rem' },
      650:  { perPage: 1, gap: '1.25rem', padding: '1rem' },
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
  <div class="min-h-full flex flex-col bg-background text-foreground">
    <Header />
    
    <main class="flex-1 flex flex-col items-center justify-start w-full max-w-[1700px] mx-auto px-4 sm:px-6 py-4 sm:py-8">
      <div class="w-full">
        <div class="text-center mb-8 sm:mb-10">
          <div class="relative inline-block mb-4">
            <div class="absolute inset-0 blur-2xl bg-gradient-to-r from-cyan-400/20 via-purple-400/20 to-pink-400/20 rounded-full"></div>
            <img
              src="/favicon.svg"
              alt="Formato logo"
              class="relative w-24 h-24 sm:w-32 sm:h-32 mx-auto transition-transform hover:scale-105 duration-300"
            />
          </div>
                  
        <h1 class="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight mb-3">
          <span class="bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
            Formato
          </span>
        </h1>
          
          <div class="mt-3 h-px w-20 sm:w-28 mx-auto bg-gradient-to-r from-transparent via-purple-400/40 to-transparent"></div>

          <p class="mt-3 text-sm sm:text-base text-purple-300/50 max-w-md mx-auto font-light tracking-wide">
            Универсальный конвертер данных
          </p>
        </div>
        {#if formats.length > 0}
          <Splide
            options={splideOptions}
            aria-label="Выбор формата"
            class="w-full"
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
                  class="group block rounded-2xl border-2 border-border bg-card/50 backdrop-blur-sm p-10 transition-all duration-300 hover:scale-[1.05] {format.borderHover} {format.glow} hover:shadow-2xl hover:-translate-y-2 mx-auto cursor-pointer"
                  style="max-width: 320px;"
                >
                  <div class="flex flex-col items-center gap-6 text-center">
                    <div class="relative rounded-3xl bg-gradient-to-br p-8 {format.color}">
                      <div class="absolute inset-0 rounded-3xl bg-gradient-to-br opacity-30 blur-2xl {format.color}"></div>
                      <Icon class="relative h-16 w-16 {format.textColor}" />
                    </div>
                    <div class="w-full min-w-0">
                      <h3 class="text-xl lg:text-2xl font-bold">{format.name}</h3>
                      <p class="mt-1.5 text-sm text-muted-foreground truncate">{format.description}</p>
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
      </div>
    </main>

    <Footer />
  </div>
</ScrollContainer>