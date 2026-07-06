<script lang="ts">
  import { Splide, SplideSlide } from '@splidejs/svelte-splide';
  // @ts-ignore
  import '@splidejs/svelte-splide/css/sea-green';
  import '$lib/styles/splide.css';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import Header from '$lib/components/Header.svelte';
  import Footer from '$lib/components/Footer.svelte';
  import FormatoLogo from '$lib/components/FormatoLogo.svelte';
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';

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

  let formats = $derived<Format[]>(page.data.formats || []);
  let splideInstance: any = null;
  let isRestoring = false;

  let formatMap = $derived.by(() => {
    const map = new Map<number, Format>();
    formats.forEach((f, i) => map.set(i, f));
    return map;
  });

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
        <FormatoLogo/>
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
                      <p class="mt-1.5 text-sm text-muted-foreground truncate">
                       {(m as any)[`format_desc_${format.id}`]()}
                    </p>
                    </div>
                  </div>
                </div>
              </SplideSlide>
            {/each}
          </Splide>
        {:else}
        <div class="text-center text-muted-foreground py-10">
          {m.no_formats()}
        </div>
        {/if}
      </div>
    </main>

    <Footer />
  </div>
</ScrollContainer>