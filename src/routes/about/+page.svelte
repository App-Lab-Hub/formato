<script lang="ts">
  import { goto } from '$app/navigation';
  import { Info, Zap, Shield, Users, Code, Sparkles, Package, Heart } from 'lucide-svelte';
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { getFormats } from '$lib/data/formats';
  import FormatoLogo from '$lib/components/FormatoLogo.svelte';
  import { m } from '$lib/paraglide/messages';
  import BackButton from '$lib/components/BackButton.svelte';
  import {
    getFormatCount,
    getTechStack,
    getTechColorClasses,
    getVersion,
    getGithubUrl,
  } from '$lib/utils/about';

  let formats = getFormats();

  function goBack() {
    goto('/');
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<ScrollContainer>
<div class="min-h-screen flex flex-col">
  <div class="flex-1 bg-background text-foreground px-6 pt-6 sm:pt-8 sm:px-8 pb-3">
      
      <BackButton onClick={goBack} />

      <div class="w-full max-w-[1700px] mx-auto">
        <div class="max-w-4xl mx-auto">
          <FormatoLogo />
        </div>
        
        <div class="max-w-4xl mx-auto">
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-8 mb-8">
            <div class="flex items-center gap-3 mb-4">
              <Info class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <h2 class="text-xl font-semibold dark:text-foreground light:text-purple-800">{m.about_title()}</h2>
            </div>
            <p class="dark:text-muted-foreground light:text-purple-700/70 leading-relaxed">
              {m.about_desc_1()}
            </p>
            <p class="dark:text-muted-foreground light:text-purple-700/70 leading-relaxed mt-3">
              {m.about_desc_2()} <span class="font-semibold dark:text-foreground light:text-purple-800">{getFormatCount(formats)}</span> {m.about_desc_3()}
            </p>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
            <div class="dark:bg-card/30 light:bg-purple-200/50 backdrop-blur-sm rounded-xl border dark:border-border light:border-purple-300/50 p-6">
              <div class="flex items-center gap-3 mb-3">
                <Zap class="h-5 w-5 text-yellow-400" />
                <h3 class="font-semibold dark:text-foreground light:text-purple-800">{m.about_fast()}</h3>
              </div>
              <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.about_fast_desc()}</p>
            </div>
            
            <div class="dark:bg-card/30 light:bg-purple-200/50 backdrop-blur-sm rounded-xl border dark:border-border light:border-purple-300/50 p-6">
              <div class="flex items-center gap-3 mb-3">
                <Shield class="h-5 w-5 text-emerald-400" />
                <h3 class="font-semibold dark:text-foreground light:text-purple-800">{m.about_secure()}</h3>
              </div>
              <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.about_secure_desc()}</p>
            </div>
            
            <div class="dark:bg-card/30 light:bg-purple-200/50 backdrop-blur-sm rounded-xl border dark:border-border light:border-purple-300/50 p-6">
              <div class="flex items-center gap-3 mb-3">
                <Sparkles class="h-5 w-5 text-purple-400" />
                <h3 class="font-semibold dark:text-foreground light:text-purple-800">{m.about_smart()}</h3>
              </div>
              <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.about_smart_desc()}</p>
            </div>
            
            <div class="dark:bg-card/30 light:bg-purple-200/50 backdrop-blur-sm rounded-xl border dark:border-border light:border-purple-300/50 p-6">
              <div class="flex items-center gap-3 mb-3">
                <Users class="h-5 w-5 text-cyan-400" />
                <h3 class="font-semibold dark:text-foreground light:text-purple-800">{m.about_for_everyone()}</h3>
              </div>
              <p class="text-sm dark:text-muted-foreground light:text-purple-700/70">{m.about_for_everyone_desc()}</p>
            </div>
          </div>

          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-8 mb-8">
            <div class="flex items-center gap-3 mb-4">
              <Code class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <h2 class="text-xl font-semibold dark:text-foreground light:text-purple-800">{m.about_tech_stack()}</h2>
            </div>
            <div class="flex flex-wrap gap-3">
              {#each getTechStack() as tech}
                <span class="px-4 py-2 rounded-full text-sm border {getTechColorClasses(tech.name)}">
                  {tech.name}
                </span>
              {/each}
            </div>
          </div>

          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-8 mb-8">
            <div class="flex items-center gap-3 mb-4">
              <Package class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <h2 class="text-xl font-semibold dark:text-foreground light:text-purple-800">{m.about_tech_details()}</h2>
            </div>
            <p class="dark:text-muted-foreground light:text-purple-700/70 leading-relaxed mb-4">
              {m.about_tech_details_desc()}
            </p>
            <a 
              href="/dependencies" 
              class="inline-flex items-center gap-2 px-6 py-3 dark:bg-primary/10 light:bg-purple-300/50 dark:hover:bg-primary/20 light:hover:bg-purple-400/60 rounded-lg text-primary transition-colors border dark:border-primary/20 light:border-purple-300/50"
            >
              <Package class="h-4 w-4" />
              <span>{m.about_view_deps()}</span>
            </a>
          </div>

          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-8">
            <div class="flex items-center gap-3 mb-4">
              <div class="h-5 w-5 text-purple-600 dark:text-purple-400">
                <FaGithub />
              </div>
              <h2 class="text-xl font-semibold dark:text-foreground light:text-purple-800">{m.about_open_source()}</h2>
            </div>
            <p class="dark:text-muted-foreground light:text-purple-700/70 leading-relaxed mb-4">
              {m.about_open_source_desc()}
            </p>
            <div class="flex flex-wrap gap-3">
              <a 
                href={getGithubUrl()} 
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center gap-2 px-6 py-3 dark:bg-primary/10 light:bg-purple-300/50 dark:hover:bg-primary/20 light:hover:bg-purple-400/60 rounded-lg text-primary transition-colors border dark:border-primary/20 light:border-purple-300/50"
              >
                <div class="h-4 w-4">
                  <FaGithub />
                </div>
                <span>{m.about_view_github()}</span>
              </a>
              
              <a 
                href="https://boosty.to/yourusername" 
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center gap-2 px-6 py-3 dark:bg-amber-500/10 light:bg-amber-200/50 dark:hover:bg-amber-500/20 light:hover:bg-amber-300/60 rounded-lg text-amber-600 dark:text-amber-400 transition-colors border border-amber-500/30 dark:border-amber-500/20 light:border-amber-300/50"
              >
                <img src="/boosty.svg" alt="Boosty" class="h-4 w-4" />
                <span>{m.about_support_boosty()}</span>
              </a>
            </div>
          </div>

          <div class="text-center mt-8 text-xs dark:text-muted-foreground/40 light:text-purple-700/40">
            {getVersion()}
          </div>
        </div>
      </div> 

    </div>
  </div>
</ScrollContainer>