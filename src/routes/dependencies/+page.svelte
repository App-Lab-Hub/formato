<!-- src/routes/dependencies/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Package, Cpu, BookOpen, ChevronDown } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import type { DependenciesData } from '$lib/services/dependencies';
  import { slide } from 'svelte/transition';
  import { cubicIn, cubicOut } from 'svelte/easing';
  import { m } from '$lib/paraglide/messages';

  let { data }: { data: { deps: DependenciesData } } = $props();

  let npmOpen = $state(true);
  let cargoOpen = $state(true);
  let npmAnimating = $state(false);
  let cargoAnimating = $state(false);

  function toggleNpm() {
    if (npmAnimating) return;
    npmAnimating = true;
    npmOpen = !npmOpen;
    setTimeout(() => npmAnimating = false, 600);
  }

  function toggleCargo() {
    if (cargoAnimating) return;
    cargoAnimating = true;
    cargoOpen = !cargoOpen;
    setTimeout(() => cargoAnimating = false, 600);
  }

  function goBack() { goto('/'); }

  function hasDependencies(group: any[]): boolean { return group && group.length > 0; }

  function getNpmGroups(deps: DependenciesData) {
    const groups: { key: string; label: string; data: any[] }[] = [];
    const npm = deps.npm;
    if (hasDependencies(npm.dependencies)) groups.push({ key: 'dependencies', label: m.deps_main(), data: npm.dependencies });
    if (hasDependencies(npm.devDependencies)) groups.push({ key: 'devDependencies', label: m.deps_dev(), data: npm.devDependencies });
    if (hasDependencies(npm.optionalDependencies)) groups.push({ key: 'optionalDependencies', label: m.deps_optional(), data: npm.optionalDependencies });
    if (hasDependencies(npm.peerDependencies)) groups.push({ key: 'peerDependencies', label: m.deps_peer(), data: npm.peerDependencies });
    if (hasDependencies(npm.bundleDependencies)) groups.push({ key: 'bundleDependencies', label: m.deps_bundle(), data: npm.bundleDependencies });
    return groups;
  }

  function getCargoGroups(deps: DependenciesData) {
    const groups: { key: string; label: string; data: any[] }[] = [];
    const cargo = deps.cargo;
    if (hasDependencies(cargo.dependencies)) groups.push({ key: 'dependencies', label: m.deps_main(), data: cargo.dependencies });
    if (hasDependencies(cargo.devDependencies)) groups.push({ key: 'dev-dependencies', label: m.deps_dev(), data: cargo.devDependencies });
    if (hasDependencies(cargo.buildDependencies)) groups.push({ key: 'build-dependencies', label: m.deps_build(), data: cargo.buildDependencies });
    if (hasDependencies(cargo.targetDependencies)) groups.push({ key: 'target-dependencies', label: m.deps_platform(), data: cargo.targetDependencies });
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
</script>

<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-screen">
    <main class="flex flex-col items-center px-8 py-16 w-full">
      
      <div class="w-full max-w-[1700px] flex justify-start mb-8">
        <button onclick={goBack} class="cursor-pointer flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors">
          <ArrowLeft class="h-5 w-5" />
          <span class="text-sm">{m.settings_back()}</span>
        </button>
      </div>

      <div class="w-full max-w-[1700px]">
        <div class="text-center mb-12">
          <h1 class="text-3xl sm:text-4xl font-bold mb-3">
            <span class="bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
              {m.deps_title()}
            </span>
          </h1>
          <div class="mt-4 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-border to-transparent"></div>
          <p class="text-muted-foreground/60 text-sm mt-2">
            {m.deps_subtitle()}
          </p>
        </div>

        <div class="max-w-4xl mx-auto space-y-6">
          {#if data.deps}
            {#if getNpmGroups(data.deps).length > 0}
              <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border overflow-hidden">
                <button
                  onclick={toggleNpm}
                  class="w-full flex items-center justify-between p-6 hover:bg-card/30 transition-colors cursor-pointer"
                >
                  <div class="flex items-center gap-3">
                    <Package class="h-5 w-5 text-yellow-400" />
                    <h2 class="text-lg font-semibold">{m.deps_npm()}</h2>
                    <span class="text-xs text-muted-foreground/60 bg-muted-foreground/10 px-2 py-1 rounded-full">
                      {getNpmGroups(data.deps).reduce((acc, g) => acc + g.data.length, 0)} {m.deps_packages()}
                    </span>
                  </div>
                  <ChevronDown class="h-5 w-5 text-muted-foreground transition-transform duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)]" style="transform: rotate({npmOpen ? 180 : 0}deg)" />
                </button>
                
                {#if npmOpen}
                  <div 
                    class="border-t border-border p-6 space-y-6" 
                    in:slide|local={{ duration: 600, easing: cubicIn }}
                    out:slide|local={{ duration: 600, easing: cubicOut }}
                  >                
                    {#each getNpmGroups(data.deps) as group}
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
                {/if}
              </div>
            {/if}

            {#if getCargoGroups(data.deps).length > 0}
              <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border overflow-hidden">
                <button
                  onclick={toggleCargo}
                  class="w-full flex items-center justify-between p-6 hover:bg-card/30 transition-colors cursor-pointer"
                >
                  <div class="flex items-center gap-3">
                    <Cpu class="h-5 w-5 text-cyan-400" />
                    <h2 class="text-lg font-semibold">{m.deps_cargo()}</h2>
                    <span class="text-xs text-muted-foreground/60 bg-muted-foreground/10 px-2 py-1 rounded-full">
                      {getCargoGroups(data.deps).reduce((acc, g) => acc + g.data.length, 0)} {m.deps_packages()}
                    </span>
                  </div>
                  <ChevronDown class="h-5 w-5 text-muted-foreground transition-transform duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)]" style="transform: rotate({cargoOpen ? 180 : 0}deg)" />
                </button>
                
                {#if cargoOpen}
                  <div 
                    class="border-t border-border p-6 space-y-6" 
                    in:slide|local={{ duration: 600, easing: cubicIn }}
                    out:slide|local={{ duration: 600, easing: cubicOut }}
                  >   
                    {#each getCargoGroups(data.deps) as group}
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
                {/if}
              </div>
            {/if}

            <div class="bg-card/30 backdrop-blur-sm rounded-2xl border border-border p-6 hover:border-primary/20 transition-all duration-300">
              <div class="flex items-center gap-3">
                <BookOpen class="h-5 w-5 text-primary" />
                <div>
                  <h3 class="text-sm font-semibold">{m.deps_where()}</h3>
                  <p class="text-xs text-muted-foreground/60 mt-1">
                    {m.deps_where_desc()}
                  </p>
                </div>
              </div>
            </div>

            <div class="text-center text-xs text-muted-foreground/40 pb-8">
              {m.deps_total()}: {getTotalCount(data.deps)}
            </div>
          {/if}
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>