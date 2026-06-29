<!-- src/routes/about/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Info, Zap, Shield, Users, Code, Sparkles, Package } from 'lucide-svelte';
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { onMount } from 'svelte';
  import { getFormats } from '$lib/data/formats';

  let formats = getFormats();

  function goBack() {
    goto('/');
  }

  onMount(() => {
    document.documentElement.style.backgroundColor = '#0a0a0f';
    return () => {
      document.documentElement.style.backgroundColor = '';
    };
  });
</script>

<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-full">
    <main class="flex flex-col items-center px-8 py-16">
      
      <!-- Кнопка назад -->
      <div class="w-full max-w-[1700px] flex justify-start mb-8">
        <button 
          onclick={goBack} 
          class="flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors"
        >
          <ArrowLeft class="h-5 w-5" />
          <span class="text-sm">На главную</span>
        </button>
      </div>

      <div class="w-full max-w-[1700px]">
        <div class="text-center mb-12">
          <img
            src="/favicon.svg"
            alt="Formato logo"
            class="w-20 h-20 mx-auto mb-4"
          />
          <h1 class="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
            Formato
          </h1>
          <p class="text-muted-foreground/60 text-lg mt-2">
            Универсальный конвертер данных
          </p>
          <div class="mt-4 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-border to-transparent"></div>
        </div>

        <div class="max-w-4xl mx-auto">
          <!-- О приложении -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-8 mb-8">
            <div class="flex items-center gap-3 mb-4">
              <Info class="h-5 w-5 text-primary" />
              <h2 class="text-xl font-semibold">О приложении</h2>
            </div>
            <p class="text-muted-foreground leading-relaxed">
              <span class="font-semibold text-foreground">Formato</span> — это мощный и простой в использовании конвертер данных, 
              который работает полностью локально на вашем устройстве. 
              Никаких загрузок на сервер, никаких ограничений по размеру файлов.
            </p>
            <p class="text-muted-foreground leading-relaxed mt-3">
              Поддерживает <span class="font-semibold text-foreground">{formats.length}</span> форматов данных, 
              включая JSON, YAML, XML, CSV, TOML, INI, Markdown и HTML.
            </p>
          </div>

          <!-- Преимущества -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
            <div class="bg-card/30 backdrop-blur-sm rounded-xl border border-border p-6">
              <div class="flex items-center gap-3 mb-3">
                <Zap class="h-5 w-5 text-yellow-400" />
                <h3 class="font-semibold">Быстро</h3>
              </div>
              <p class="text-sm text-muted-foreground">
                Мгновенная конвертация благодаря нативной обработке данных на Rust
              </p>
            </div>
            
            <div class="bg-card/30 backdrop-blur-sm rounded-xl border border-border p-6">
              <div class="flex items-center gap-3 mb-3">
                <Shield class="h-5 w-5 text-emerald-400" />
                <h3 class="font-semibold">Безопасно</h3>
              </div>
              <p class="text-sm text-muted-foreground">
                Все данные обрабатываются локально на вашем устройстве. Ничего не отправляется в интернет
              </p>
            </div>
            
            <div class="bg-card/30 backdrop-blur-sm rounded-xl border border-border p-6">
              <div class="flex items-center gap-3 mb-3">
                <Sparkles class="h-5 w-5 text-purple-400" />
                <h3 class="font-semibold">Умно</h3>
              </div>
              <p class="text-sm text-muted-foreground">
                Интеллектуальный парсинг и преобразование данных с сохранением структуры
              </p>
            </div>
            
            <div class="bg-card/30 backdrop-blur-sm rounded-xl border border-border p-6">
              <div class="flex items-center gap-3 mb-3">
                <Users class="h-5 w-5 text-cyan-400" />
                <h3 class="font-semibold">Для всех</h3>
              </div>
              <p class="text-sm text-muted-foreground">
                Простой интерфейс для разработчиков, аналитиков и всех, кто работает с данными
              </p>
            </div>
          </div>

        <!-- Стек технологий -->
        <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-8 mb-8">
          <div class="flex items-center gap-3 mb-4">
            <Code class="h-5 w-5 text-primary" />
            <h2 class="text-xl font-semibold">Стек технологий</h2>
          </div>
          <div class="flex flex-wrap gap-3">
            <span class="px-4 py-2 bg-primary/10 rounded-full text-sm text-primary border border-primary/20">Tauri</span>
            <span class="px-4 py-2 bg-cyan-500/10 rounded-full text-sm text-cyan-400 border border-cyan-400/20">Rust</span>
            <span class="px-4 py-2 bg-yellow-500/10 rounded-full text-sm text-yellow-400 border border-yellow-400/20">SvelteKit</span>
            <span class="px-4 py-2 bg-blue-500/10 rounded-full text-sm text-blue-400 border border-blue-400/20">TypeScript</span>
            <span class="px-4 py-2 bg-purple-500/10 rounded-full text-sm text-purple-400 border border-purple-400/20">Tailwind CSS</span>
            <span class="px-4 py-2 bg-pink-500/10 rounded-full text-sm text-pink-400 border border-pink-400/20">OverlayScrollbars</span>
            <span class="px-4 py-2 bg-orange-500/10 rounded-full text-sm text-orange-400 border border-orange-400/20">Monaco Editor</span>
            <span class="px-4 py-2 bg-green-500/10 rounded-full text-sm text-green-400 border border-green-400/20">Lucide Icons</span>
            <span class="px-4 py-2 bg-red-500/10 rounded-full text-sm text-red-400 border border-red-400/20">Splide</span>
          </div>
        </div>

          <!-- Технические детали / Зависимости -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-8 mb-8">
            <div class="flex items-center gap-3 mb-4">
              <Package class="h-5 w-5 text-primary" />
              <h2 class="text-xl font-semibold">Технические детали</h2>
            </div>
            <p class="text-muted-foreground leading-relaxed mb-4">
              Посмотрите полное дерево зависимостей проекта — все NPM и Cargo пакеты, используемые в Formato.
            </p>
            <a 
              href="/dependencies" 
              class="inline-flex items-center gap-2 px-6 py-3 bg-primary/10 hover:bg-primary/20 rounded-lg text-primary transition-colors border border-primary/20"
            >
              <Package class="h-4 w-4" />
              <span>Посмотреть зависимости</span>
            </a>
          </div>

          <!-- GitHub -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-8">
            <div class="flex items-center gap-3 mb-4">
              <div class="h-5 w-5 text-primary">
                <FaGithub />
              </div>
              <h2 class="text-xl font-semibold">Открытый исходный код</h2>
            </div>
            <p class="text-muted-foreground leading-relaxed mb-4">
              Formato — это проект с открытым исходным кодом. Вы можете внести свой вклад, сообщить о проблемах 
              или просто посмотреть код на GitHub.
            </p>
            <a 
              href="https://github.com/yourusername/formato" 
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center gap-2 px-6 py-3 bg-primary/10 hover:bg-primary/20 rounded-lg text-primary transition-colors border border-primary/20"
            >
              <div class="h-4 w-4">
                <FaGithub />
              </div>
              <span>Посмотреть на GitHub</span>
            </a>
          </div>

          <div class="text-center mt-8 text-xs text-muted-foreground/40">
            v0.1.0 • Сделано с ❤️
          </div>
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>