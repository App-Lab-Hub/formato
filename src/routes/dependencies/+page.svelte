<!-- src/routes/dependencies/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Package, Cpu, BookOpen, ChevronRight, ChevronDown, Wrench, Target, Users, Globe } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import type { DependenciesData } from '$lib/services/dependencies';

  // Используем $props() вместо export let
  let { data }: { data: { deps: DependenciesData } } = $props();

  let deps = $state(data.deps);
  let error = $state<string | null>(null);
  let expandedSections = $state<Set<string>>(new Set(['npm', 'cargo']));

  function goBack() {
    goto('/');
  }

  function toggleSection(section: string) {
    if (expandedSections.has(section)) {
      expandedSections.delete(section);
    } else {
      expandedSections.add(section);
    }
    expandedSections = new Set(expandedSections);
  }

  const iconMap: Record<string, any> = {
    'dependencies': Package,
    'devDependencies': Wrench,
    'optionalDependencies': Target,
    'peerDependencies': Users,
    'bundleDependencies': Globe,
    'dev-dependencies': Wrench,
    'build-dependencies': Globe,
  };

  function getDependencyIcon(type: string) {
    return iconMap[type] || Package;
  }

  function getDependencyLabel(type: string): string {
    const labels: Record<string, string> = {
      'dependencies': 'Основные',
      'devDependencies': 'Dev-зависимости',
      'optionalDependencies': 'Опциональные',
      'peerDependencies': 'Peer-зависимости',
      'bundleDependencies': 'В сборке',
      'dev-dependencies': 'Dev-зависимости',
      'build-dependencies': 'Build-зависимости',
    };
    return labels[type] || type;
  }

  function hasDependencies(group: any[]): boolean {
    return group && group.length > 0;
  }

  function getNpmGroups(deps: DependenciesData) {
    const groups: { key: string; label: string; data: any[] }[] = [];
    const npm = deps.npm;
    
    if (hasDependencies(npm.dependencies)) {
      groups.push({ key: 'dependencies', label: 'Основные', data: npm.dependencies });
    }
    if (hasDependencies(npm.devDependencies)) {
      groups.push({ key: 'devDependencies', label: 'Dev-зависимости', data: npm.devDependencies });
    }
    if (hasDependencies(npm.optionalDependencies)) {
      groups.push({ key: 'optionalDependencies', label: 'Опциональные', data: npm.optionalDependencies });
    }
    if (hasDependencies(npm.peerDependencies)) {
      groups.push({ key: 'peerDependencies', label: 'Peer-зависимости', data: npm.peerDependencies });
    }
    if (hasDependencies(npm.bundleDependencies)) {
      groups.push({ key: 'bundleDependencies', label: 'В сборке', data: npm.bundleDependencies });
    }
    
    return groups;
  }

  function getCargoGroups(deps: DependenciesData) {
    const groups: { key: string; label: string; data: any[] }[] = [];
    const cargo = deps.cargo;
    
    if (hasDependencies(cargo.dependencies)) {
      groups.push({ key: 'dependencies', label: 'Основные', data: cargo.dependencies });
    }
    if (hasDependencies(cargo.devDependencies)) {
      groups.push({ key: 'dev-dependencies', label: 'Dev-зависимости', data: cargo.devDependencies });
    }
    if (hasDependencies(cargo.buildDependencies)) {
      groups.push({ key: 'build-dependencies', label: 'Build-зависимости', data: cargo.buildDependencies });
    }
    if (hasDependencies(cargo.targetDependencies)) {
      groups.push({ key: 'target-dependencies', label: 'Платформенные', data: cargo.targetDependencies });
    }
    
    return groups;
  }

  function getTotalCount(deps: DependenciesData): number {
    let count = 0;
    const npm = deps.npm;
    const cargo = deps.cargo;
    
    count += npm.dependencies?.length || 0;
    count += npm.devDependencies?.length || 0;
    count += npm.optionalDependencies?.length || 0;
    count += npm.peerDependencies?.length || 0;
    count += npm.bundleDependencies?.length || 0;
    
    count += cargo.dependencies?.length || 0;
    count += cargo.devDependencies?.length || 0;
    count += cargo.buildDependencies?.length || 0;
    count += cargo.targetDependencies?.length || 0;
    
    return count;
  }
</script>

<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-full">
    <main class="flex flex-col items-center px-8 py-16">
      
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

        <div class="max-w-4xl mx-auto">
          {#if error}
            <div class="text-center text-red-400 py-10">
              <p>{error}</p>
            </div>
          {:else if deps}
            <div class="space-y-6">
              <!-- NPM Dependencies -->
              {#if getNpmGroups(deps).length > 0}
                <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border overflow-hidden">
                  <button 
                    onclick={() => toggleSection('npm')}
                    class="w-full flex items-center justify-between p-6 hover:bg-card/30 transition-colors"
                  >
                    <div class="flex items-center gap-3">
                      <Package class="h-5 w-5 text-yellow-400" />
                      <h2 class="text-lg font-semibold">NPM зависимости</h2>
                      <span class="text-xs text-muted-foreground/60 bg-muted-foreground/10 px-2 py-1 rounded-full">
                        {getNpmGroups(deps).reduce((acc, g) => acc + g.data.length, 0)} пакетов
                      </span>
                    </div>
                    {#if expandedSections.has('npm')}
                      <ChevronDown class="h-5 w-5 text-muted-foreground" />
                    {:else}
                      <ChevronRight class="h-5 w-5 text-muted-foreground" />
                    {/if}
                  </button>
                  
                  {#if expandedSections.has('npm')}
                    <div class="border-t border-border p-6 space-y-6">
                      {#each getNpmGroups(deps) as group}
                        {@const Icon = getDependencyIcon(group.key)}
                        <div>
                          <h3 class="text-sm font-medium text-muted-foreground mb-3 flex items-center gap-2">
                            <Icon class="h-4 w-4" />
                            {group.label}
                            <span class="text-xs text-muted-foreground/40">({group.data.length})</span>
                          </h3>
                          <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                            {#each group.data as dep}
                              <div class="flex items-center justify-between px-4 py-2 bg-background/50 rounded-lg border border-border/50">
                                <span class="text-sm font-mono">{dep.name}</span>
                                <span class="text-xs text-muted-foreground/60 font-mono">{dep.version}</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}

              <!-- Cargo Dependencies -->
              {#if getCargoGroups(deps).length > 0}
                <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border overflow-hidden">
                  <button 
                    onclick={() => toggleSection('cargo')}
                    class="w-full flex items-center justify-between p-6 hover:bg-card/30 transition-colors"
                  >
                    <div class="flex items-center gap-3">
                      <Cpu class="h-5 w-5 text-cyan-400" />
                      <h2 class="text-lg font-semibold">Cargo зависимости</h2>
                      <span class="text-xs text-muted-foreground/60 bg-muted-foreground/10 px-2 py-1 rounded-full">
                        {getCargoGroups(deps).reduce((acc, g) => acc + g.data.length, 0)} пакетов
                      </span>
                    </div>
                    {#if expandedSections.has('cargo')}
                      <ChevronDown class="h-5 w-5 text-muted-foreground" />
                    {:else}
                      <ChevronRight class="h-5 w-5 text-muted-foreground" />
                    {/if}
                  </button>
                  
                  {#if expandedSections.has('cargo')}
                    <div class="border-t border-border p-6 space-y-6">
                      {#each getCargoGroups(deps) as group}
                        {@const Icon = getDependencyIcon(group.key)}
                        <div>
                          <h3 class="text-sm font-medium text-muted-foreground mb-3 flex items-center gap-2">
                            <Icon class="h-4 w-4" />
                            {group.label}
                            <span class="text-xs text-muted-foreground/40">({group.data.length})</span>
                          </h3>
                          <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                            {#each group.data as dep}
                              <div class="flex items-center justify-between px-4 py-2 bg-background/50 rounded-lg border border-border/50">
                                <span class="text-sm font-mono">{dep.name}</span>
                                <span class="text-xs text-muted-foreground/60 font-mono">{dep.version}</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}

              <div class="bg-card/30 backdrop-blur-sm rounded-2xl border border-border p-6">
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

              <div class="text-center text-xs text-muted-foreground/40">
                Всего зависимостей: {getTotalCount(deps)}
              </div>
            </div>
          {/if}
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>