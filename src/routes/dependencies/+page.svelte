<!-- src/routes/dependencies/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Package, Cpu, BookOpen, ChevronDown } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import type { DependenciesData } from '$lib/services/dependencies';
  import { onMount, tick } from 'svelte';
  import { animate, stagger } from '@motionone/dom';

  let { data }: { data: { deps: DependenciesData } } = $props();
  let deps = $state(data.deps);

  let npmOpen = $state(true);
  let cargoOpen = $state(true);
  let npmContentVisible = $state(true);
  let cargoContentVisible = $state(true);
  let npmContent: HTMLElement;
  let cargoContent: HTMLElement;
  let npmAnimating = $state(false);
  let cargoAnimating = $state(false);

  async function toggleNpm() {
    if (npmAnimating) return;
    
    if (!npmOpen) {
      npmOpen = true;
      npmContentVisible = true;
      await tick();
      
      if (!npmContent) return;
      npmAnimating = true;
      const content = npmContent;
      
      animate(content, 
        { height: [0, content.scrollHeight], opacity: [0, 1] }, 
        { duration: 0.5, easing: [0.4, 0, 0.2, 1] }
      ).finished.then(() => {
        npmAnimating = false;
      });
    } else {
      if (!npmContent) return;
      npmAnimating = true;
      const content = npmContent;
      
      npmOpen = false;
      animate(content, 
        { height: [content.scrollHeight, 0], opacity: [1, 0] }, 
        { duration: 0.5, easing: [0.4, 0, 0.2, 1] }
      ).finished.then(() => {
        npmContentVisible = false;
        npmAnimating = false;
      });
    }
  }

  async function toggleCargo() {
    if (cargoAnimating) return;
    
    if (!cargoOpen) {
      cargoOpen = true;
      cargoContentVisible = true;
      await tick();
      
      if (!cargoContent) return;
      cargoAnimating = true;
      const content = cargoContent;
      
      animate(content, 
        { height: [0, content.scrollHeight], opacity: [0, 1] }, 
        { duration: 0.5, easing: [0.4, 0, 0.2, 1] }
      ).finished.then(() => {
        cargoAnimating = false;
      });
    } else {
      if (!cargoContent) return;
      cargoAnimating = true;
      const content = cargoContent;
      
      cargoOpen = false;
      animate(content, 
        { height: [content.scrollHeight, 0], opacity: [1, 0] }, 
        { duration: 0.5, easing: [0.4, 0, 0.2, 1] }
      ).finished.then(() => {
        cargoContentVisible = false;
        cargoAnimating = false;
      });
    }
  }

  function goBack() { goto('/'); }

  function hasDependencies(group: any[]): boolean { return group && group.length > 0; }

  function getNpmGroups(deps: DependenciesData) {
    const groups: { key: string; label: string; data: any[] }[] = [];
    const npm = deps.npm;
    if (hasDependencies(npm.dependencies)) groups.push({ key: 'dependencies', label: 'Основные', data: npm.dependencies });
    if (hasDependencies(npm.devDependencies)) groups.push({ key: 'devDependencies', label: 'Dev-зависимости', data: npm.devDependencies });
    if (hasDependencies(npm.optionalDependencies)) groups.push({ key: 'optionalDependencies', label: 'Опциональные', data: npm.optionalDependencies });
    if (hasDependencies(npm.peerDependencies)) groups.push({ key: 'peerDependencies', label: 'Peer-зависимости', data: npm.peerDependencies });
    if (hasDependencies(npm.bundleDependencies)) groups.push({ key: 'bundleDependencies', label: 'В сборке', data: npm.bundleDependencies });
    return groups;
  }

  function getCargoGroups(deps: DependenciesData) {
    const groups: { key: string; label: string; data: any[] }[] = [];
    const cargo = deps.cargo;
    if (hasDependencies(cargo.dependencies)) groups.push({ key: 'dependencies', label: 'Основные', data: cargo.dependencies });
    if (hasDependencies(cargo.devDependencies)) groups.push({ key: 'dev-dependencies', label: 'Dev-зависимости', data: cargo.devDependencies });
    if (hasDependencies(cargo.buildDependencies)) groups.push({ key: 'build-dependencies', label: 'Build-зависимости', data: cargo.buildDependencies });
    if (hasDependencies(cargo.targetDependencies)) groups.push({ key: 'target-dependencies', label: 'Платформенные', data: cargo.targetDependencies });
    return groups;
  }

  function getTotalCount(deps: DependenciesData): number {
    let count = 0;
    count += deps.npm.dependencies?.length || 0;
    count += deps.npm.devDependencies?.length || 0;
    count += deps.npm.optionalDependencies?.length || 0;
    count += deps.npm.peerDependencies?.length || 0;
    count += deps.npm.bundleDependencies?.length || 0;
    count += deps.cargo.dependencies?.length || 0;
    count += deps.cargo.devDependencies?.length || 0;
    count += deps.cargo.buildDependencies?.length || 0;
    count += deps.cargo.targetDependencies?.length || 0;
    return count;
  }

  onMount(() => {
    const title = document.querySelector('.deps-title');
    if (title) animate(title as HTMLElement, { opacity: [0, 1], transform: ['translateY(-20px)', 'translateY(0)'] }, { duration: 0.6, easing: 'ease-out' });

    const sections = document.querySelectorAll('.dep-section');
    if (sections.length) animate(sections as NodeListOf<HTMLElement>, { opacity: [0, 1], transform: ['translateY(20px)', 'translateY(0)'] }, { delay: stagger(0.15), duration: 0.5, easing: 'ease-out' });
  });
</script>

<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-screen">
    <main class="flex flex-col items-center px-8 py-16 w-full">
      
      <div class="w-full max-w-[1700px] flex justify-start mb-8">
        <button onclick={goBack} class="cursor-pointer flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors">
          <ArrowLeft class="h-5 w-5" />
          <span class="text-sm">На главную</span>
        </button>
      </div>

      <div class="w-full max-w-[1700px]">
        <div class="text-center mb-12 deps-title" style="opacity: 0">
          <h1 class="text-3xl sm:text-4xl font-bold mb-3">
            <span class="bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
              Зависимости проекта
            </span>
          </h1>
          <div class="mt-4 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-border to-transparent"></div>
          <p class="text-muted-foreground/60 text-sm mt-2">
            Все NPM и Cargo пакеты, используемые в Formato
          </p>
        </div>

        <div class="max-w-4xl mx-auto space-y-6">
          {#if deps}
            {#if getNpmGroups(deps).length > 0}
              <div class="dep-section bg-card/50 backdrop-blur-sm rounded-2xl border border-border overflow-hidden" style="opacity: 0">
                <button
                  onclick={toggleNpm}
                  class="w-full flex items-center justify-between p-6 hover:bg-card/30 transition-colors cursor-pointer"
                >
                  <div class="flex items-center gap-3">
                    <Package class="h-5 w-5 text-yellow-400" />
                    <h2 class="text-lg font-semibold">NPM зависимости</h2>
                    <span class="text-xs text-muted-foreground/60 bg-muted-foreground/10 px-2 py-1 rounded-full">
                      {getNpmGroups(deps).reduce((acc, g) => acc + g.data.length, 0)} пакетов
                    </span>
                  </div>
                  <ChevronDown class="chevron-npm h-5 w-5 text-muted-foreground transition-transform duration-400" style="transform: rotate({npmOpen ? 180 : 0}deg)" />
                </button>
                
                {#if npmContentVisible}
                  <div bind:this={npmContent} class="border-t border-border overflow-hidden">
                    <div class="p-6 space-y-6">
                      {#each getNpmGroups(deps) as group}
                        <div>
                          <h3 class="text-sm font-medium text-muted-foreground mb-3">
                            {group.label}
                            <span class="text-xs text-muted-foreground/40 ml-1">({group.data.length})</span>
                          </h3>
                          <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                            {#each group.data as dep}
                              <div class="flex items-center justify-between px-4 py-2 bg-background/50 rounded-lg border border-border/50 hover:border-primary/30 hover:bg-background/80 transition-all duration-200">
                                <span class="text-sm font-mono">{dep.name}</span>
                                <span class="text-xs text-muted-foreground/60 font-mono">{dep.version}</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {/if}

            {#if getCargoGroups(deps).length > 0}
              <div class="dep-section bg-card/50 backdrop-blur-sm rounded-2xl border border-border overflow-hidden" style="opacity: 0">
                <button
                  onclick={toggleCargo}
                  class="w-full flex items-center justify-between p-6 hover:bg-card/30 transition-colors cursor-pointer"
                >
                  <div class="flex items-center gap-3">
                    <Cpu class="h-5 w-5 text-cyan-400" />
                    <h2 class="text-lg font-semibold">Cargo зависимости</h2>
                    <span class="text-xs text-muted-foreground/60 bg-muted-foreground/10 px-2 py-1 rounded-full">
                      {getCargoGroups(deps).reduce((acc, g) => acc + g.data.length, 0)} пакетов
                    </span>
                  </div>
                  <ChevronDown class="chevron-cargo h-5 w-5 text-muted-foreground transition-transform duration-400" style="transform: rotate({cargoOpen ? 180 : 0}deg)" />
                </button>
                
                {#if cargoContentVisible}
                  <div bind:this={cargoContent} class="border-t border-border overflow-hidden">
                    <div class="p-6 space-y-6">
                      {#each getCargoGroups(deps) as group}
                        <div>
                          <h3 class="text-sm font-medium text-muted-foreground mb-3">
                            {group.label}
                            <span class="text-xs text-muted-foreground/40 ml-1">({group.data.length})</span>
                          </h3>
                          <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                            {#each group.data as dep}
                              <div class="flex items-center justify-between px-4 py-2 bg-background/50 rounded-lg border border-border/50 hover:border-primary/30 hover:bg-background/80 transition-all duration-200">
                                <span class="text-sm font-mono">{dep.name}</span>
                                <span class="text-xs text-muted-foreground/60 font-mono">{dep.version}</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {/if}

            <div class="dep-section bg-card/30 backdrop-blur-sm rounded-2xl border border-border p-6 hover:border-primary/20 transition-all duration-300" style="opacity: 0">
              <div class="flex items-center gap-3">
                <BookOpen class="h-5 w-5 text-primary" />
                <div>
                  <h3 class="text-sm font-semibold">Где хранятся данные?</h3>
                  <p class="text-xs text-muted-foreground/60 mt-1">
                    Файлы <span class="font-mono">package.json</span> и <span class="font-mono">Cargo.toml</span> копируются 
                    при сборке и доступны через <span class="font-mono">/static</span>.
                  </p>
                </div>
              </div>
            </div>

            <div class="text-center text-xs text-muted-foreground/40 pb-8">
              Всего зависимостей: {getTotalCount(deps)}
            </div>
          {/if}
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>