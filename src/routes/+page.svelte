<!-- src/routes/+page.svelte -->
<script lang="ts">
  import { Splide, SplideSlide } from '@splidejs/svelte-splide';
  // @ts-ignore
  import '@splidejs/svelte-splide/css/sea-green';
  import '$lib/styles/splide.css';
  import { getFormats } from '$lib/data/formats';
  import { goto } from '$app/navigation';
  import { customScroll } from '$lib/actions/scroll';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';

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

  // Храним соответствие между индексом и форматом
  const formatMap = new Map<number, any>();
  formats.forEach((f, i) => formatMap.set(i, f));

  // Функция для нормализации индекса (работает с отрицательными и большими числами)
  function normalizeIndex(index: number): number {
    if (index < 0) {
      // Для отрицательных индексов (клон слева)
      return (index % formats.length + formats.length) % formats.length;
    } else {
      // Для положительных индексов (включая клоны справа)
      return index % formats.length;
    }
  }

  function goToConvert(formatId: string, index: number) {
    console.log('[goToConvert] Переход:', formatId, 'индекс:', index);
    
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
          console.log('[restoreSplidePosition] Восстанавливаем индекс:', index);
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

<div class="h-screen w-screen" use:customScroll>
  <div class="flex flex-col bg-background text-foreground min-h-full">
    <main class="flex flex-col items-center gap-8 px-8 py-16">

      <img
        src="/logo.svg"
        alt="Formato logo"
        class="w-16 h-16 sm:w-20 sm:h-20"
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
            
            // Подписываемся на событие click от Splide
            if (splideInstance) {
              splideInstance.on('click', (Slide: any, event: MouseEvent) => {
                // Получаем индекс слайда (может быть отрицательным для клонов слева)
                const slideIndex = Slide.index;
                // Нормализуем индекс
                const realIndex = normalizeIndex(slideIndex);
                const format = formatMap.get(realIndex);
                
                console.log('[Splide click] Слайд индекс:', slideIndex, 'реальный:', realIndex, 'формат:', format?.id);
                
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
    </main>
  </div>
</div>