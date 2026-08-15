<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Package, Cpu, BookOpen, ChevronDown } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import type { DependenciesData } from '$lib/services/dependencies';
  import { slide } from 'svelte/transition';
  import { cubicIn, cubicOut } from 'svelte/easing';
  import { m } from '$lib/paraglide/messages';
  import BackButton from '$lib/components/BackButton.svelte';
  import {
    hasDependencies,
    getNpmGroups,
    getCargoGroups,
    getTotalCount,
    getGroupCount,
    getGroupLabel,
  } from '$lib/utils/dependencies';

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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<ScrollContainer>
  <div class="flex flex-col bg-background text-foreground min-h-screen">
    <main class="flex flex-col items-center px-8 py-16 w-full">
      
      <BackButton onClick={goBack} />

      <div class="w-full max-w-[1700px] pt-6">
        <div class="text-center mb-12">
          <h1 class="text-3xl sm:text-4xl font-bold mb-3">
            <span class="bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 dark:from-cyan-400 dark:via-purple-400 dark:to-pink-400 light:from-cyan-600 light:via-purple-600 light:to-pink-600 bg-clip-text text-transparent">
              {m.deps_title()}
            </span>
          </h1>
          <div class="mt-4 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-purple-400/50 to-transparent"></div>
          <p class="dark:text-muted-foreground/60 light:text-purple-800/60 text-sm mt-2">
            {m.deps_subtitle()}
          </p>
        </div>

        <div class="max-w-4xl mx-auto space-y-6">
          {#if data.deps}
            {#if getNpmGroups(data.deps).length > 0}
              <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 overflow-hidden">
                <button
                  onclick={toggleNpm}
                  class="w-full flex items-center justify-between p-6 dark:hover:bg-card/30 light:hover:bg-purple-300/50 transition-colors cursor-pointer"
                >
                  <div class="flex items-center gap-3">
                    <Package class="h-5 w-5 text-yellow-400" />
                    <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.deps_npm()}</h2>
                    <span class="text-xs dark:text-muted-foreground/60 light:text-purple-700/60 dark:bg-muted-foreground/10 light:bg-purple-300/50 px-2 py-1 rounded-full">
                      {getGroupCount(data.deps, 'npm')} {m.deps_packages()}
                    </span>
                  </div>
                  <ChevronDown class="h-5 w-5 dark:text-muted-foreground light:text-purple-600/60 transition-transform duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)]" style="transform: rotate({npmOpen ? 180 : 0}deg)" />
                </button>
                
                {#if npmOpen}
                  <div 
                    class="dark:border-t border-border light:border-t border-purple-300/40 p-6 space-y-6" 
                    in:slide|local={{ duration: 600, easing: cubicIn }}
                    out:slide|local={{ duration: 600, easing: cubicOut }}
                  >                
                    {#each getNpmGroups(data.deps) as group}
                      <div>
                        <h3 class="text-sm font-medium dark:text-muted-foreground light:text-purple-700/70 mb-3">
                          {getGroupLabel(group.key)}
                          <span class="text-xs dark:text-muted-foreground/40 light:text-purple-600/40 ml-1">({group.data.length})</span>
                        </h3>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                          {#each group.data as dep}
                            <div class="flex items-center justify-between px-4 py-2 dark:bg-background/50 light:bg-purple-100/60 rounded-lg border dark:border-border/50 light:border-purple-300/40 dark:hover:border-primary/30 light:hover:border-purple-400/60 dark:hover:bg-background/80 light:hover:bg-purple-200/70 transition-all duration-200">
                              <span class="text-sm font-mono dark:text-foreground light:text-purple-800/90">{dep.name}</span>
                              <span class="text-xs dark:text-muted-foreground/60 light:text-purple-600/50 font-mono">{dep.version}</span>
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
              <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 overflow-hidden">
                <button
                  onclick={toggleCargo}
                  class="w-full flex items-center justify-between p-6 dark:hover:bg-card/30 light:hover:bg-purple-300/50 transition-colors cursor-pointer"
                >
                  <div class="flex items-center gap-3">
                    <Cpu class="h-5 w-5 text-cyan-400" />
                    <h2 class="text-lg font-semibold dark:text-foreground light:text-purple-800">{m.deps_cargo()}</h2>
                    <span class="text-xs dark:text-muted-foreground/60 light:text-purple-700/60 dark:bg-muted-foreground/10 light:bg-purple-300/50 px-2 py-1 rounded-full">
                      {getGroupCount(data.deps, 'cargo')} {m.deps_packages()}
                    </span>
                  </div>
                  <ChevronDown class="h-5 w-5 dark:text-muted-foreground light:text-purple-600/60 transition-transform duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)]" style="transform: rotate({cargoOpen ? 180 : 0}deg)" />
                </button>
                
                {#if cargoOpen}
                  <div 
                    class="dark:border-t border-border light:border-t border-purple-300/40 p-6 space-y-6" 
                    in:slide|local={{ duration: 600, easing: cubicIn }}
                    out:slide|local={{ duration: 600, easing: cubicOut }}
                  >   
                    {#each getCargoGroups(data.deps) as group}
                      <div>
                        <h3 class="text-sm font-medium dark:text-muted-foreground light:text-purple-700/70 mb-3">
                          {getGroupLabel(group.key)}
                          <span class="text-xs dark:text-muted-foreground/40 light:text-purple-600/40 ml-1">({group.data.length})</span>
                        </h3>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                          {#each group.data as dep}
                            <div class="flex items-center justify-between px-4 py-2 dark:bg-background/50 light:bg-purple-100/60 rounded-lg border dark:border-border/50 light:border-purple-300/40 dark:hover:border-primary/30 light:hover:border-purple-400/60 dark:hover:bg-background/80 light:hover:bg-purple-200/70 transition-all duration-200">
                              <span class="text-sm font-mono dark:text-foreground light:text-purple-800/90">{dep.name}</span>
                              <span class="text-xs dark:text-muted-foreground/60 light:text-purple-600/50 font-mono">{dep.version}</span>
                            </div>
                          {/each}
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}

            <div class="dark:bg-card/30 light:bg-purple-200/40 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-6 dark:hover:border-primary/20 light:hover:border-purple-400/60 transition-all duration-300">
              <div class="flex items-center gap-3">
                <BookOpen class="h-5 w-5 text-purple-600 dark:text-purple-400" />
                <div>
                  <h3 class="text-sm font-semibold dark:text-foreground light:text-purple-800">{m.deps_where()}</h3>
                  <p class="text-xs dark:text-muted-foreground/60 light:text-purple-700/60 mt-1">
                    {m.deps_where_desc()}
                  </p>
                </div>
              </div>
            </div>

            <div class="text-center text-xs dark:text-muted-foreground/40 light:text-purple-700/40 pb-8">
              {m.deps_total()}: {getTotalCount(data.deps)}
            </div>
          {/if}
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>