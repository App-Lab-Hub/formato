<!-- src/lib/components/convert/FormatCard.svelte -->
<script lang="ts">
  import type { Format } from '$lib/types/format';
  import { m } from '$lib/paraglide/messages';
  import { Lock, Sparkles, AlertCircle, Download, Check } from 'lucide-svelte';

  let {
    format,
    status,
    isSelected,
    onselect,
  }: {
    format: Format;
    status: string;
    isSelected: boolean;
    onselect: (format: Format) => void;
  } = $props();

  const Icon = format.icon;
  const isAvailable = status === 'available' || status === 'available_with_ai';
  const isNotAvailable = status === 'not_available';
  const isAvailableWithAI = status === 'available_with_ai';

  function getStatusInfo() {
    switch (status) {
      case 'available':
        return { 
          label: 'Доступно', 
          color: 'text-green-400 border-green-400/30 bg-green-500/10',
          icon: Check,
        };
      case 'available_with_ai':
        return { 
          label: 'Требуется AI', 
          color: 'text-yellow-400 border-yellow-400/30 bg-yellow-500/10',
          icon: Sparkles,
        };
      case 'not_available':
        return { 
          label: 'Недоступно', 
          color: 'text-red-400 border-red-400/30 bg-red-500/10',
          icon: Lock,
        };
      default:
        return { 
          label: 'Неизвестно', 
          color: 'text-gray-400 border-gray-400/30 bg-gray-500/10',
          icon: AlertCircle,
        };
    }
  }

  const statusInfo = getStatusInfo();

  function handleDownloadAI(e: MouseEvent) {
    e.stopPropagation();
    console.log('📦 Скачивание AI модели для формата:', format.name);
  }

  function handleCardClick() {
    if (isAvailable) {
      onselect(format);
    }
  }
</script>

<!-- ДЕФОЛТНАЯ КАРТОЧКА -->
{#if !isAvailableWithAI && !isNotAvailable}
<button
  onclick={handleCardClick}
  class="group relative flex flex-col items-center justify-center gap-4 rounded-2xl border-2 p-5 w-full aspect-[4/5] transition-all duration-500 overflow-hidden text-center select-none backdrop-blur-sm
         {isSelected 
           ? 'border-primary bg-primary/5 scale-[1.04]' 
           : 'dark:border-border light:border-neutral-200 dark:bg-card light:bg-neutral-50/40 hover:scale-[1.01]'}
         {isSelected ? format.glow : ''} 
         {format.borderHover}"
>
  <!-- Мягкий контур при наведении (подстраивается под цвет иконки) -->
  <div class="absolute -inset-px bg-gradient-to-b from-current to-transparent rounded-2xl opacity-0 group-hover:opacity-10 transition-opacity duration-500 -z-10 {format.textColor}"></div>

  <!-- Центрированная иконка с динамическим градиентом и свечением из БД -->
  <div class="relative rounded-2xl bg-gradient-to-br p-5 transition-all duration-500 group-hover:scale-110 group-hover:shadow-md {format.color} {format.glow}">
    <!-- Размытое фоновое облако (Blur) под иконкой -->
    <div class="absolute inset-0 rounded-2xl bg-gradient-to-br opacity-20 blur-xl transition-opacity group-hover:opacity-40 {format.color}"></div>
    
    <div class="flex-shrink-0 h-11 w-11 relative z-10 flex items-center justify-center">
      <!-- Использование компонента Svelte 5 из интерфейса Format -->
      <format.icon class="w-full h-full transition-transform duration-500 group-hover:rotate-12 {format.textColor}" />
    </div>
  </div>

  <!-- Блок текстов (Защищен от "выжигания" глаз в светлой теме) -->
  <div class="flex flex-col items-center gap-1.5 z-10 px-2 w-full mt-1">
    <span class="text-base font-extrabold tracking-wide dark:text-foreground light:text-neutral-800 transition-colors duration-300">
      {format.name}
    </span>
    <span class="text-xs font-medium leading-relaxed dark:text-muted-foreground light:text-neutral-500/90 line-clamp-3 max-w-[160px]">
      {(m as any)[`format_desc_${format.id}`]()}
    </span>
  </div>

  <!-- Мягкий нейтральный градиентный наплыв внизу для объема -->
  <div class="absolute bottom-0 left-0 right-0 h-1/4 pointer-events-none rounded-b-2xl bg-gradient-to-t dark:from-neutral-950/20 light:from-neutral-200/20 to-transparent z-10"></div>
</button>


{/if}

<!-- КАРТОЧКА С AI - СТРУКТУРА ОДИН В ОДИН КАК У ISNOTAVAILABLE -->
{#if isAvailableWithAI}
  <div
    class="group relative flex flex-col items-center justify-center gap-3 sm:gap-4 rounded-2xl border-2 p-4 xs:p-5 w-full aspect-[4/5] overflow-hidden text-center select-none transition-all duration-500
           dark:border-border/60 light:border-neutral-200/80 dark:bg-card/40 light:bg-neutral-50/40 cursor-default"
  >
    <!-- Бейдж статуса в углу (Минималистичный AI с пульсацией) -->
    <div class="absolute top-2.5 right-2.5 xs:top-3 xs:right-3 z-20">
      <div class="flex items-center gap-1 xs:gap-1.5 px-2 py-0.5 rounded-full text-[8px] xs:text-[9px] font-extrabold tracking-wider uppercase border border-current/20 bg-current/5 {format.textColor}">
        <span class="w-1.5 h-1.5 rounded-full bg-current animate-pulse"></span>
        <span>AI</span>
      </div>
    </div>

    <!-- Центрированная приглушенная иконка -->
    <div class="relative rounded-2xl bg-gradient-to-br from-neutral-200/50 via-neutral-100/40 to-neutral-200/20 dark:from-neutral-900/60 dark:to-neutral-950/40 p-3.5 xs:p-5 grayscale opacity-40 border dark:border-neutral-800/50 light:border-neutral-200/40">
      <!-- Размытое фоновое облако (Blur) под иконкой (использует цвет формата из БД) -->
      <div class="absolute inset-0 rounded-2xl bg-gradient-to-br opacity-20 blur-xl {format.color}"></div>
      
      <div class="flex-shrink-0 h-9 w-9 xs:h-11 xs:w-11 relative z-10 flex items-center justify-center">
        <format.icon class="w-full h-full dark:text-neutral-500 light:text-neutral-400" />
      </div>
    </div>

    <!-- Блок текстов с мягким пастельным fade-out эффектом маски -->
    <div class="flex flex-col items-center gap-1 xs:gap-1.5 z-10 px-1 xs:px-2 w-full mt-0.5 xs:mt-1
                [mask-image:linear-gradient(to_bottom,rgba(0,0,0,1)_50%,rgba(0,0,0,0.6)_75%,rgba(0,0,0,0)_100%)]">
      <span class="text-sm xs:text-base font-extrabold tracking-wide dark:text-neutral-500 light:text-neutral-400">
        {format.name}
      </span>
      <span class="text-[11px] xs:text-xs font-medium leading-relaxed dark:text-neutral-600 light:text-neutral-400/80 line-clamp-3 max-w-[140px] xs:max-w-[160px]">
        {(m as any)[`format_desc_${format.id}`]()}
      </span>
    </div>

    <!-- Матовое глухое перекрытие поверх нижней части (один в один как в оригинале) -->
    <div class="absolute bottom-0 left-0 right-0 h-1/3 pointer-events-none rounded-b-2xl bg-gradient-to-t dark:from-neutral-950/80 dark:via-neutral-950/30 light:from-neutral-100/80 light:via-neutral-100/20 to-transparent z-10"></div>
    
    <!-- Интерактивная кнопка "Скачать AI" по центру в самом низу -->
    <div 
      role="button"
      tabindex="0"
      onclick={(e) => { 
        e.stopPropagation(); 
        handleDownloadAI(e); 
      }}
      onkeydown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.stopPropagation();
          // @ts-ignore
          handleDownloadAI(e);
        }
      }}
      class="absolute bottom-4 left-1/2 -translate-x-1/2 z-20 flex items-center justify-center gap-1.5 px-3.5 xs:px-4 py-1.5 xs:py-2 rounded-xl border text-[9px] xs:text-[10px] font-black tracking-widest uppercase whitespace-nowrap cursor-pointer transform-gpu will-change-transform select-none max-w-[calc(100%-32px)]
             hover:scale-[1.02] active:scale-[0.98] active:duration-75 origin-center
             bg-white dark:bg-neutral-900 text-neutral-800 dark:text-neutral-200 border-neutral-200 dark:border-neutral-800
             group-hover:bg-gradient-to-br group-hover:border-transparent group-hover:shadow-md {format.color} {format.textColor}"
    >
      <div class="flex items-center justify-center h-3.5 w-3.5 flex-shrink-0">
        <Download class="h-full w-full stroke-[2.5]" />
      </div>
      <span class="leading-none pt-[0.5px] antialiased subpixel-antialiased max-w-[max-content]">Скачать AI</span>
    </div>
  </div>
{/if}






<!-- КАРТОЧКА НЕДОСТУПНО - ПРЕМИАЛЬНАЯ РУБИНОВАЯ ТЕМА -->
{#if isNotAvailable}
  <div
    class="group relative flex flex-col items-center justify-center gap-3 sm:gap-4 rounded-2xl border-2 p-4 xs:p-5 w-full aspect-[4/5] overflow-hidden text-center select-none
           dark:border-red-950/40 light:border-red-100 dark:bg-neutral-950/40 light:bg-neutral-50/20 
           cursor-not-allowed transition-all duration-500"
  >
    <!-- Тонкое рубиновое свечение по контуру карточки в темной теме -->
    <div class="absolute -inset-px bg-gradient-to-b dark:from-red-500/10 light:from-red-500/5 to-transparent rounded-2xl -z-10"></div>

    <!-- Бейдж статуса (Минималистичный замок) -->
    <div class="absolute top-2.5 right-2.5 xs:top-3 xs:right-3 z-20">
      <div class="flex items-center gap-1 xs:gap-1.5 px-2 py-0.5 rounded-full text-[8px] xs:text-[9px] font-extrabold tracking-wider uppercase border border-red-500/20 bg-red-500/5 dark:text-red-400 light:text-red-600">
        <Lock class="h-2.5 w-2.5" />
        <span>Locked</span>
      </div>
    </div>

    <!-- Центрированная приглушенная иконка -->
    <div class="relative rounded-2xl bg-gradient-to-br from-neutral-200/50 via-neutral-100/40 to-neutral-200/20 dark:from-neutral-900/60 dark:to-neutral-950/40 p-3.5 xs:p-5 grayscale opacity-40 border dark:border-neutral-800/50 light:border-neutral-200/40">
      <!-- Едва заметное багровое облако (Blur) глубоко под иконкой -->
      <div class="absolute inset-0 rounded-2xl bg-gradient-to-br from-red-500/10 to-transparent opacity-20 blur-xl"></div>
      
      <div class="flex-shrink-0 h-9 w-9 xs:h-11 xs:w-11 relative z-10 flex items-center justify-center">
        <!-- Берем родную иконку формата, но обесцвечиваем её для эффекта блокировки -->
        <format.icon class="w-full h-full dark:text-neutral-500 light:text-neutral-400" />
      </div>
    </div>

    <!-- Блок текстов с мягким пастельным fade-out эффектом маски -->
    <div class="flex flex-col items-center gap-1 xs:gap-1.5 z-10 px-1 xs:px-2 w-full mt-0.5 xs:mt-1
                [mask-image:linear-gradient(to_bottom,rgba(0,0,0,1)_50%,rgba(0,0,0,0.6)_75%,rgba(0,0,0,0)_100%)]">
      <span class="text-sm xs:text-base font-extrabold tracking-wide dark:text-neutral-500 light:text-neutral-400">
        {format.name}
      </span>
      <span class="text-[11px] xs:text-xs font-medium leading-relaxed dark:text-neutral-600 light:text-neutral-400/80 line-clamp-3 max-w-[140px] xs:max-w-[160px]">
        {(m as any)[`format_desc_${format.id}`]()}
      </span>
    </div>

    <!-- Матовое глухое перекрытие поверх нижней части (вместо уродливого черного слоя по центру) -->
    <div class="absolute bottom-0 left-0 right-0 h-1/3 pointer-events-none rounded-b-2xl bg-gradient-to-t dark:from-neutral-950/80 dark:via-neutral-950/30 light:from-neutral-100/80 light:via-neutral-100/20 to-transparent z-10"></div>
    
    <!-- Деликатная плашка по центру в самом низу — Стили ОДИН В ОДИН как у кнопки AI выше -->
    <div class="absolute bottom-4 left-1/2 -translate-x-1/2 z-20 flex items-center justify-center gap-1.5 px-3.5 xs:px-4 py-1.5 xs:py-2 rounded-xl border text-[9px] xs:text-[10px] font-black tracking-widest uppercase whitespace-nowrap max-w-[calc(100%-32px)]
                dark:border-red-950 dark:bg-red-950/30 light:border-red-100 light:bg-red-50/60 dark:text-red-400/80 light:text-red-600/80">
      <div class="flex items-center justify-center h-3.5 w-3.5 flex-shrink-0">
        <Lock class="h-full w-full stroke-[2.5]" />
      </div>
      <span class="leading-none pt-[0.5px] antialiased subpixel-antialiased max-w-[calc(100%-16px)] overflow-hidden text-ellipsis">Формат недоступен</span>
    </div>
  </div>
{/if}
