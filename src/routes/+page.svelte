<!-- src/routes/+page.svelte -->
<script lang="ts">
  import { Splide, SplideSlide } from '@splidejs/svelte-splide';
  // @ts-ignore
  import '@splidejs/svelte-splide/css/sea-green';
  import '$lib/styles/splide.css';
  import { getFormats, isFormatsLoaded } from '$lib/data/formats';
  import { customScroll } from '$lib/actions/scroll';
  import { goto } from '$app/navigation';
  import { LoaderCircle } from 'lucide-svelte';
  import { onMount } from 'svelte';

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

  let formats = $state(getFormats());
  let isLoading = $state(!isFormatsLoaded() && formats.length === 0);

  onMount(() => {
    // Если данные ещё не загружены — ждём
    if (!isFormatsLoaded() && formats.length === 0) {
      const checkFormats = setInterval(() => {
        const f = getFormats();
        if (f.length > 0) {
          formats = f;
          isLoading = false;
          clearInterval(checkFormats);
        }
      }, 100);
      
      return () => clearInterval(checkFormats);
    } else {
      isLoading = false;
    }
  });

  function goToConvert(format: any) {
    goto(`/convert/${format.id}`);
  }
</script>

<div class="h-screen w-screen overflow-hidden" use:customScroll>
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

      {#if isLoading}
        <div class="flex flex-col items-center justify-center gap-4 py-20">
          <LoaderCircle class="h-16 w-16 text-primary animate-spin" />
          <span class="text-sm text-muted-foreground">Загрузка форматов...</span>
        </div>
      {:else if formats.length > 0}
        <Splide
          options={splideOptions}
          aria-label="Выбор формата"
          class="w-full max-w-[1700px] mx-auto"
        >
          {#each formats as format}
            {@const Icon = format.icon}
            <SplideSlide>
              <div
                on:click={() => goToConvert(format)}
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