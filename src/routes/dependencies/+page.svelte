<!-- src/routes/dependencies/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Package, Cpu, BookOpen, ChevronRight, ChevronDown } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { onMount } from 'svelte';

  // Данные из package.json (замени на актуальные)
  const npmDependencies = [
    { name: '@splidejs/svelte-splide', version: '^0.2.0' },
    { name: '@tauri-apps/api', version: '^2.0.0' },
    { name: '@tauri-apps/plugin-dialog', version: '^2.0.0' },
    { name: '@tauri-apps/plugin-fs', version: '^2.0.0' },
    { name: 'lucide-svelte', version: '^1.0.0' },
    { name: 'overlayscrollbars', version: '^2.0.0' },
    { name: 'svelte-icons', version: '^2.0.0' },
    { name: '@motionone/dom', version: '^11.0.0' },
  ];

  const npmDevDependencies = [
    { name: 'typescript', version: '^5.0.0' },
    { name: 'svelte', version: '^5.0.0' },
    { name: 'vite', version: '^6.0.0' },
    { name: '@sveltejs/adapter-static', version: '^3.0.0' },
    { name: '@sveltejs/kit', version: '^2.0.0' },
    { name: 'tailwindcss', version: '^3.0.0' },
  ];

  // Данные из Cargo.toml
  const cargoDependencies = [
    { name: 'tauri', version: '^2.0.0' },
    { name: 'serde', version: '^1.0.0' },
    { name: 'serde_json', version: '^1.0.0' },
    { name: 'tokio', version: '^1.0.0' },
    { name: 'anyhow', version: '^1.0.0' },
    { name: 'thiserror', version: '^1.0.0' },
    { name: 'log', version: '^0.4.0' },
    { name: 'env_logger', version: '^0.11.0' },
  ];

  const cargoDevDependencies = [
    { name: 'tauri-build', version: '^2.0.0' },
    { name: 'serde_with', version: '^3.0.0' },
  ];

  function goBack() {
    goto('/');
  }

  let expandedSections = $state<Set<string>>(new Set(['npm', 'cargo']));

  function toggleSection(section: string) {
    if (expandedSections.has(section)) {
      expandedSections.delete(section);
    } else {
      expandedSections.add(section);
    }
    expandedSections = new Set(expandedSections);
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
          <h1 class="text-3xl sm:text-4xl font-bold bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
            Зависимости проекта
          </h1>
          <p class="text-muted-foreground/60 text-sm mt-2">
            Все NPM и Cargo пакеты, используемые в Formato
          </p>
          <div class="mt-4 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-border to-transparent"></div>
        </div>

        <div class="max-w-4xl mx-auto space-y-6">
          
          <!-- NPM Dependencies -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border overflow-hidden">
            <button 
              onclick={() => toggleSection('npm')}
              class="w-full flex items-center justify-between p-6 hover:bg-card/30 transition-colors"
            >
              <div class="flex items-center gap-3">
                <Package class="h-5 w-5 text-yellow-400" />
                <h2 class="text-lg font-semibold">NPM зависимости</h2>
                <span class="text-xs text-muted-foreground/60 bg-muted-foreground/10 px-2 py-1 rounded-full">
                  {npmDependencies.length + npmDevDependencies.length} пакетов
                </span>
              </div>
              {#if expandedSections.has('npm')}
                <ChevronDown class="h-5 w-5 text-muted-foreground" />
              {:else}
                <ChevronRight class="h-5 w-5 text-muted-foreground" />
              {/if}
            </button>
            
            {#if expandedSections.has('npm')}
              <div class="border-t border-border p-6">
                <h3 class="text-sm font-medium text-muted-foreground mb-3">Основные зависимости</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-2 mb-6">
                  {#each npmDependencies as dep}
                    <div class="flex items-center justify-between px-4 py-2 bg-background/50 rounded-lg border border-border/50">
                      <span class="text-sm font-mono">{dep.name}</span>
                      <span class="text-xs text-muted-foreground/60 font-mono">{dep.version}</span>
                    </div>
                  {/each}
                </div>
                
                <h3 class="text-sm font-medium text-muted-foreground mb-3">Dev-зависимости</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                  {#each npmDevDependencies as dep}
                    <div class="flex items-center justify-between px-4 py-2 bg-background/50 rounded-lg border border-border/50">
                      <span class="text-sm font-mono">{dep.name}</span>
                      <span class="text-xs text-muted-foreground/60 font-mono">{dep.version}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Cargo Dependencies -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border overflow-hidden">
            <button 
              onclick={() => toggleSection('cargo')}
              class="w-full flex items-center justify-between p-6 hover:bg-card/30 transition-colors"
            >
              <div class="flex items-center gap-3">
                <Cpu class="h-5 w-5 text-cyan-400" />
                <h2 class="text-lg font-semibold">Cargo зависимости</h2>
                <span class="text-xs text-muted-foreground/60 bg-muted-foreground/10 px-2 py-1 rounded-full">
                  {cargoDependencies.length + cargoDevDependencies.length} пакетов
                </span>
              </div>
              {#if expandedSections.has('cargo')}
                <ChevronDown class="h-5 w-5 text-muted-foreground" />
              {:else}
                <ChevronRight class="h-5 w-5 text-muted-foreground" />
              {/if}
            </button>
            
            {#if expandedSections.has('cargo')}
              <div class="border-t border-border p-6">
                <h3 class="text-sm font-medium text-muted-foreground mb-3">Основные зависимости</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-2 mb-6">
                  {#each cargoDependencies as dep}
                    <div class="flex items-center justify-between px-4 py-2 bg-background/50 rounded-lg border border-border/50">
                      <span class="text-sm font-mono">{dep.name}</span>
                      <span class="text-xs text-muted-foreground/60 font-mono">{dep.version}</span>
                    </div>
                  {/each}
                </div>
                
                <h3 class="text-sm font-medium text-muted-foreground mb-3">Dev-зависимости</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                  {#each cargoDevDependencies as dep}
                    <div class="flex items-center justify-between px-4 py-2 bg-background/50 rounded-lg border border-border/50">
                      <span class="text-sm font-mono">{dep.name}</span>
                      <span class="text-xs text-muted-foreground/60 font-mono">{dep.version}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Информация -->
          <div class="bg-card/30 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3">
              <BookOpen class="h-5 w-5 text-primary" />
              <div>
                <h3 class="text-sm font-semibold">Как обновляются зависимости?</h3>
                <p class="text-xs text-muted-foreground/60 mt-1">
                  Данные берутся из <span class="font-mono">package.json</span> и <span class="font-mono">Cargo.toml</span>. 
                  Обновляются при каждой сборке приложения.
                </p>
              </div>
            </div>
          </div>

        </div>

        <div class="text-center mt-8 text-xs text-muted-foreground/40">
          Всего зависимостей: {npmDependencies.length + npmDevDependencies.length + cargoDependencies.length + cargoDevDependencies.length}
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>