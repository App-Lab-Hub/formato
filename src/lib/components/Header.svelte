<script lang="ts">
  import { Settings, Info } from 'lucide-svelte';

  let isClicked = false;
  
  let currentGradient = 'linear-gradient(to right, #22d3ee 0%, #22d3ee 30%, #a855f7 30.1%, #a855f7 70%, transparent 100%)';
  let shadowColor = 'rgba(168, 85, 247, 0.15)';

  const neonPalette = [
    { hex: '#7dd3fc', rgba: '125, 211, 252' },
    { hex: '#f9a8d4', rgba: '249, 168, 212' },
    { hex: '#86efac', rgba: '134, 239, 172' },
    { hex: '#fde68a', rgba: '253, 230, 138' },
    { hex: '#c4b5fd', rgba: '196, 181, 253' },
    { hex: '#93c5fd', rgba: '147, 197, 253' },
    { hex: '#fca5a5', rgba: '252, 165, 165' },
    { hex: '#a5b4fc', rgba: '165, 180, 252' },
    { hex: '#d8b4fe', rgba: '216, 180, 254' },
    { hex: '#6ee7b7', rgba: '110, 231, 183' },
    { hex: '#fdba74', rgba: '253, 186, 116' },
    { hex: '#67e8f9', rgba: '103, 232, 249' }
  ];

  // Все возможные раскладки ширин блоков
  const layouts = [
    // 1 цвет + фиолетовый
    { accent: 1, stops: [{ start: 0, end: 30 }], purpleStart: 30 },
    { accent: 1, stops: [{ start: 0, end: 40 }], purpleStart: 40 },
    { accent: 1, stops: [{ start: 0, end: 50 }], purpleStart: 50 },
    { accent: 1, stops: [{ start: 0, end: 20 }], purpleStart: 20 },
    // 2 цвета + фиолетовый
    { accent: 2, stops: [{ start: 0, end: 20 }, { start: 20, end: 35 }], purpleStart: 35 },
    { accent: 2, stops: [{ start: 0, end: 25 }, { start: 25, end: 45 }], purpleStart: 45 },
    { accent: 2, stops: [{ start: 0, end: 30 }, { start: 30, end: 50 }], purpleStart: 50 },
    { accent: 2, stops: [{ start: 0, end: 15 }, { start: 15, end: 40 }], purpleStart: 40 },
    { accent: 2, stops: [{ start: 0, end: 35 }, { start: 35, end: 55 }], purpleStart: 55 },
    { accent: 2, stops: [{ start: 0, end: 10 }, { start: 10, end: 25 }], purpleStart: 25 },
  ];

  // Дополнительные смещения для фиолетового блока
  const purpleEnds = [75, 80, 70, 85];

  function handleLogoClick(e: MouseEvent) {
    // Случайная раскладка
    const layout = layouts[Math.floor(Math.random() * layouts.length)];
    
    // Случайные цвета
    const shuffled = [...neonPalette].sort(() => Math.random() - 0.5);
    const selected = shuffled.slice(0, layout.accent);
    
    // Случайный конец фиолетового блока
    const purpleEnd = purpleEnds[Math.floor(Math.random() * purpleEnds.length)];
    
    let gradientSteps: string[] = [];

    // Строим блоки по раскладке
    layout.stops.forEach((stop, i) => {
      gradientSteps.push(`${selected[i].hex} ${stop.start}%`);
      gradientSteps.push(`${selected[i].hex} ${stop.end}%`);
    });

    // Фиолетовый блок
    gradientSteps.push(`#a855f7 ${layout.purpleStart}.1%`);
    gradientSteps.push(`#a855f7 ${purpleEnd}%`);
    
    // Прозрачность
    gradientSteps.push('transparent 100%');

    currentGradient = `linear-gradient(to right, ${gradientSteps.join(', ')})`;
    shadowColor = `rgba(${selected[0].rgba}, 0.2)`;

    isClicked = true;
    setTimeout(() => isClicked = false, 600);
  }
</script>
<header 
  class="w-full border-b dark:border-purple-500/10 light:border-purple-300/20 dark:bg-background/80 light:bg-white/80 backdrop-blur-xl relative transition-all duration-500 ease-out z-10 overflow-visible
  {isClicked ? 'dark:border-purple-500/30 light:border-purple-400/30' : 'dark:border-purple-500/10 light:border-purple-300/20'}"
  style={isClicked ? `box-shadow: 0 8px 25px ${shadowColor};` : ''}
>
  <div class="absolute bottom-0 left-0 right-0 h-[2px] overflow-hidden pointer-events-none z-0">
    <div 
      class="h-full opacity-0 transition-all duration-500
      {isClicked ? 'w-full opacity-80' : 'w-0'}"
      style="background: {currentGradient};"
    ></div>
  </div>

  <div class="flex justify-between items-center px-5 sm:px-8 py-3.5 max-w-[1700px] mx-auto relative z-10">
    <a href="/" class="flex items-center gap-2 group select-none" on:click={handleLogoClick}>
      <div class="relative flex items-center justify-center" style="padding: 4px;">
        <img
          src="/favicon.svg"
          alt="Formato"
          class="w-8 h-8 sm:w-9 sm:h-9 transition-all duration-500 group-hover:scale-110 group-hover:drop-shadow-[0_0_12px_rgba(168,85,247,0.5)]
          {isClicked ? 'rotate-[360deg]' : ''}"
        />
      </div>
      <div class="h-5 w-px dark:bg-purple-500/20 light:bg-purple-400/20 mx-1 hidden sm:block"></div>
      <span 
        class="hidden sm:block text-lg font-bold dark:bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 light:bg-gradient-to-r light:from-cyan-600 light:via-purple-600 light:to-pink-600 bg-clip-text text-transparent transition-all duration-300 group-hover:tracking-wider"
      >
        Formato
      </span>
    </a>

    <nav class="flex items-center gap-1">
      <a 
        href="/about" 
        class="flex items-center gap-2 px-3.5 py-2 rounded-lg text-base font-medium dark:text-purple-300/60 light:text-purple-600/60 dark:hover:text-purple-200 light:hover:text-purple-700 dark:hover:bg-purple-500/10 light:hover:bg-purple-400/10 transition-all duration-300"
      >
        <Info class="h-4 w-4" />
        <span class="hidden sm:inline">О нас</span>
      </a>
      <a 
        href="/settings" 
        class="flex items-center gap-2 px-3.5 py-2 rounded-lg text-base font-medium dark:text-purple-300/60 light:text-purple-600/60 dark:hover:text-purple-200 light:hover:text-purple-700 dark:hover:bg-purple-500/10 light:hover:bg-purple-400/10 transition-all duration-300"
      >
        <Settings class="h-4 w-4" />
        <span class="hidden sm:inline">Настройки</span>
      </a>
    </nav>
  </div>
</header>