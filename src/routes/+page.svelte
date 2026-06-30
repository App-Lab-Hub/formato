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
  import { Settings, Info, Heart } from 'lucide-svelte';
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';

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
    // 900:  { perPage: 2, gap: '2rem', padding: '1rem' },
    900:  { perPage: 2 },

    600:  { perPage: 1, gap: '1.25rem', padding: '0.75rem' },
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
  <div class="min-h-screen flex flex-col bg-background text-foreground">
    <!-- HEADER -->
    <header class="w-full border-b border-border/30 bg-background/50 backdrop-blur-sm">
      <div class="flex justify-between items-center px-4 sm:px-6 py-3 max-w-[1700px] mx-auto">
        <div class="flex items-center gap-2.5">
          <img
            src="/favicon.svg"
            alt="Formato"
            class="w-7 h-7 sm:w-9 sm:h-9 opacity-80"
          />
          <span class="text-sm sm:text-base font-medium text-muted-foreground/60">Formato</span>
        </div>
        <div class="flex items-center gap-1.5 sm:gap-2">
          <a 
            href="/about" 
            class="flex items-center gap-1.5 px-2.5 sm:px-3 py-1.5 rounded-lg text-xs sm:text-sm text-muted-foreground/60 hover:text-primary hover:bg-primary/10 transition-all duration-200"
          >
            <Info class="h-3.5 w-3.5" />
            <span class="hidden sm:inline">О нас</span>
          </a>
          <a 
            href="/settings" 
            class="flex items-center gap-1.5 px-2.5 sm:px-3 py-1.5 rounded-lg text-xs sm:text-sm text-muted-foreground/60 hover:text-primary hover:bg-primary/10 transition-all duration-200"
          >
            <Settings class="h-3.5 w-3.5" />
            <span class="hidden sm:inline">Настройки</span>
          </a>
        </div>
      </div>
    </header>
    
    <!-- MAIN CONTENT -->
    <main class="flex-1 flex flex-col items-center justify-center w-full max-w-[1700px] mx-auto px-4 sm:px-6 py-4 sm:py-8">
      <div class="w-full">
        <!-- Логотип и заголовок -->
        <div class="text-center mb-6 sm:mb-8">
          <div class="relative inline-block mb-3">
            <div class="absolute inset-0 blur-2xl bg-gradient-to-r from-cyan-400/20 via-purple-400/20 to-pink-400/20 rounded-full"></div>
            <img
              src="/favicon.svg"
              alt="Formato logo"
              class="relative w-20 h-20 sm:w-28 sm:h-28 mx-auto transition-transform hover:scale-105 duration-300"
            />
          </div>
          
          <h1 class="text-2xl sm:text-3xl lg:text-4xl font-bold bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent mb-2">
            Formato
          </h1>
          <p class="text-xs sm:text-sm text-muted-foreground/60 max-w-md mx-auto">
            Универсальный конвертер данных — быстро, локально, без лишнего
          </p>
          <div class="mt-3 h-px w-20 sm:w-28 mx-auto bg-gradient-to-r from-transparent via-border to-transparent"></div>
          <h2 class="mt-3 text-sm sm:text-base lg:text-lg font-light tracking-[0.3em] uppercase bg-gradient-to-r from-primary to-primary/50 bg-clip-text text-transparent">
            Convert from
          </h2>
        </div>

        <!-- Карусель -->
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
                  class="group block rounded-2xl border-2 border-border bg-card/50 backdrop-blur-sm p-6 sm:p-10 transition-all duration-300 hover:scale-[1.05] {format.borderHover} {format.glow} hover:shadow-2xl hover:-translate-y-2 mx-auto cursor-pointer"
                  style="max-width: 320px;"
                >
                  <div class="flex flex-col items-center gap-4 sm:gap-6 text-center">
                    <div class="relative rounded-3xl bg-gradient-to-br p-6 sm:p-8 {format.color}">
                      <div class="absolute inset-0 rounded-3xl bg-gradient-to-br opacity-30 blur-2xl {format.color}"></div>
                      <Icon class="relative h-14 w-14 sm:h-16 sm:w-16 {format.textColor}" />
                    </div>
                    <div class="w-full min-w-0">
                      <h3 class="text-lg sm:text-xl lg:text-2xl font-bold">{format.name}</h3>
                      <p class="mt-1.5 text-xs sm:text-sm text-muted-foreground truncate">{format.description}</p>
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

    <!-- FOOTER -->
    <footer class="w-full border-t border-border/30 bg-background/50 backdrop-blur-sm mt-auto">
      <div class="flex flex-col sm:flex-row items-center justify-between gap-2 px-4 sm:px-6 py-3 max-w-[1700px] mx-auto">
        <div class="flex items-center gap-3 text-xs sm:text-sm text-muted-foreground/40">
          <span>v0.1.0</span>
          <span class="hidden sm:inline">•</span>
          <span class="flex items-center gap-2">
            Сделано с <Heart class="h-3 w-3 text-red-400/60 fill-red-400/20" />
          </span>
        </div>
        <div class="flex items-center gap-2 text-xs sm:text-sm">
          <a 
            href="/about" 
            class="text-muted-foreground/40 hover:text-primary/70 transition-colors duration-200"
          >
            О нас
          </a>
          <span class="text-muted-foreground/20">|</span>
          <a 
            href="/settings" 
            class="text-muted-foreground/40 hover:text-primary/70 transition-colors duration-200"
          >
            Настройки
          </a>
          <span class="text-muted-foreground/20">|</span>
          <a 
            href="/dependencies" 
            class="text-muted-foreground/40 hover:text-primary/70 transition-colors duration-200"
          >
            Зависимости
          </a>
          <span class="text-muted-foreground/20">|</span>
          <a 
            href="https://github.com/yourusername/formato" 
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center gap-1 text-muted-foreground/40 hover:text-primary/70 transition-colors duration-200"
          >
            <div class="h-3.5 w-3.5">
              <FaGithub />
            </div>
            <span>GitHub</span>
          </a>
        </div>
      </div>
    </footer>
  </div>
</ScrollContainer> 