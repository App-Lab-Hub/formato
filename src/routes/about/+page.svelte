<!-- src/routes/about/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Info, Zap, Shield, Users, Code, Sparkles, Package } from 'lucide-svelte';
  // @ts-ignore
  import { FaGithub } from 'svelte-icons/fa';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { onMount } from 'svelte';
  import { getFormats } from '$lib/data/formats';
  import FormatoLogo from '$lib/components/FormatoLogo.svelte';
  import { m } from '$lib/paraglide/messages';

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
      
      <div class="w-full max-w-[1700px] flex justify-start mb-8">
        <button 
          onclick={goBack} 
          class="flex items-center gap-2 dark:text-muted-foreground light:text-purple-700/70 dark:hover:text-primary light:hover:text-purple-800 transition-colors"
        >
          <ArrowLeft class="h-5 w-5" />
          <span class="text-sm">{m.settings_back()}</span>
        </button>
      </div>

      <div class="w-full max-w-[1700px]">
        <FormatoLogo/>
        <div class="max-w-4xl mx-auto">
          <!-- О приложении -->
          <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-8 mb-8">
            <div class="flex items-center gap-3 mb-4">
              <Info class="h-5 w-5 text-purple-600 dark:text-purple-400" />
              <h2 class="text-xl font-semibold dark:text-foreground light:text-purple-800">{m.about_title()}</h2>
            </div>
            <p class="dark:text-muted-foreground light:text-purple-700/70 leading-relaxed">
              {m.about_desc_1()}
            </p>
            <p class="dark:text-muted-foreground light:text-purple-700/70 leading-relaxed mt-3">
              {m.about_desc_2()} <span class="font-semibold dark:text-foreground light:text-purple-800">{formats.length}</span> {m.about_desc_3()}
            </p>
          </div>

          <!-- Преимущества -->
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

        <!-- Стек технологий -->
        <div class="dark:bg-card/50 light:bg-purple-200/50 backdrop-blur-sm rounded-2xl border dark:border-border light:border-purple-300/50 p-8 mb-8">
          <div class="flex items-center gap-3 mb-4">
            <Code class="h-5 w-5 text-purple-600 dark:text-purple-400" />
            <h2 class="text-xl font-semibold dark:text-foreground light:text-purple-800">{m.about_tech_stack()}</h2>
          </div>
          <div class="flex flex-wrap gap-3">
            <span class="px-4 py-2 dark:bg-primary/10 light:bg-purple-300/50 rounded-full text-sm dark:text-primary light:text-purple-700 border dark:border-primary/20 light:border-purple-300/50">Tauri</span>
            <span class="px-4 py-2 dark:bg-cyan-500/10 light:bg-cyan-200/50 rounded-full text-sm dark:text-cyan-400 light:text-cyan-700 border dark:border-cyan-400/20 light:border-cyan-300/50">Rust</span>
            <span class="px-4 py-2 dark:bg-yellow-500/10 light:bg-yellow-200/50 rounded-full text-sm dark:text-yellow-400 light:text-yellow-700 border dark:border-yellow-400/20 light:border-yellow-300/50">SvelteKit</span>
            <span class="px-4 py-2 dark:bg-blue-500/10 light:bg-blue-200/50 rounded-full text-sm dark:text-blue-400 light:text-blue-700 border dark:border-blue-400/20 light:border-blue-300/50">TypeScript</span>
            <span class="px-4 py-2 dark:bg-purple-500/10 light:bg-purple-300/50 rounded-full text-sm dark:text-purple-400 light:text-purple-700 border dark:border-purple-400/20 light:border-purple-300/50">Tailwind CSS</span>
            <span class="px-4 py-2 dark:bg-pink-500/10 light:bg-pink-200/50 rounded-full text-sm dark:text-pink-400 light:text-pink-700 border dark:border-pink-400/20 light:border-pink-300/50">OverlayScrollbars</span>
            <span class="px-4 py-2 dark:bg-orange-500/10 light:bg-orange-200/50 rounded-full text-sm dark:text-orange-400 light:text-orange-700 border dark:border-orange-400/20 light:border-orange-300/50">Monaco Editor</span>
            <span class="px-4 py-2 dark:bg-green-500/10 light:bg-green-200/50 rounded-full text-sm dark:text-green-400 light:text-green-700 border dark:border-green-400/20 light:border-green-300/50">Lucide Icons</span>
            <span class="px-4 py-2 dark:bg-red-500/10 light:bg-red-200/50 rounded-full text-sm dark:text-red-400 light:text-red-700 border dark:border-red-400/20 light:border-red-300/50">Splide</span>
          </div>
        </div>

          <!-- Технические детали / Зависимости -->
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

          <!-- GitHub -->
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
            <a 
              href="https://github.com/yourusername/formato" 
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center gap-2 px-6 py-3 dark:bg-primary/10 light:bg-purple-300/50 dark:hover:bg-primary/20 light:hover:bg-purple-400/60 rounded-lg text-primary transition-colors border dark:border-primary/20 light:border-purple-300/50"
            >
              <div class="h-4 w-4">
                <FaGithub />
              </div>
              <span>{m.about_view_github()}</span>
            </a>
          </div>

          <div class="text-center mt-8 text-xs dark:text-muted-foreground/40 light:text-purple-700/40">
            v0.1.0
          </div>
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>