<!-- src/routes/settings/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { ArrowLeft, Sun, Moon, Monitor, Bell, BellOff, Shield, Languages, Palette, Volume2, VolumeX, Save } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';

  let theme = $state('system');
  let notifications = $state(true);
  let language = $state('ru');
  let sound = $state(true);
  let autoSave = $state(true);

  function goBack() {
    goto('/');
  }

  function saveSettings() {
    if (browser) {
      localStorage.setItem('settings', JSON.stringify({
        theme,
        notifications,
        language,
        sound,
        autoSave
      }));
    }
    console.log('Settings saved');
  }

  onMount(() => {
    document.documentElement.style.backgroundColor = '#0a0a0f';
    
    if (browser) {
      const saved = localStorage.getItem('settings');
      if (saved) {
        try {
          const settings = JSON.parse(saved);
          theme = settings.theme || 'system';
          notifications = settings.notifications ?? true;
          language = settings.language || 'ru';
          sound = settings.sound ?? true;
          autoSave = settings.autoSave ?? true;
        } catch (e) {}
      }
    }

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
            Настройки
          </h1>
          <p class="text-muted-foreground/60 text-sm mt-2">
            Настройте приложение под себя
          </p>
          <div class="mt-4 h-px w-32 mx-auto bg-gradient-to-r from-transparent via-border to-transparent"></div>
        </div>

        <div class="space-y-6 max-w-3xl mx-auto">
          <!-- Тема -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-4">
              <Palette class="h-5 w-5 text-primary" />
              <h2 class="text-lg font-semibold">Тема</h2>
            </div>
            <div class="grid grid-cols-3 gap-3">
              <button 
                onclick={() => theme = 'light'}
                class={`px-4 py-3 rounded-xl border transition-all flex items-center justify-center gap-2 ${
                  theme === 'light' 
                    ? 'border-primary bg-primary/10 text-primary' 
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <Sun class="h-4 w-4" />
                <span class="text-sm">Светлая</span>
              </button>
              <button 
                onclick={() => theme = 'dark'}
                class={`px-4 py-3 rounded-xl border transition-all flex items-center justify-center gap-2 ${
                  theme === 'dark' 
                    ? 'border-primary bg-primary/10 text-primary' 
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <Moon class="h-4 w-4" />
                <span class="text-sm">Тёмная</span>
              </button>
              <button 
                onclick={() => theme = 'system'}
                class={`px-4 py-3 rounded-xl border transition-all flex items-center justify-center gap-2 ${
                  theme === 'system' 
                    ? 'border-primary bg-primary/10 text-primary' 
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <Monitor class="h-4 w-4" />
                <span class="text-sm">Системная</span>
              </button>
            </div>
          </div>

          <!-- Язык -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-4">
              <Languages class="h-5 w-5 text-primary" />
              <h2 class="text-lg font-semibold">Язык</h2>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <button 
                onclick={() => language = 'ru'}
                class={`px-4 py-3 rounded-xl border transition-all ${
                  language === 'ru' 
                    ? 'border-primary bg-primary/10 text-primary' 
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <span class="text-sm">Русский</span>
              </button>
              <button 
                onclick={() => language = 'en'}
                class={`px-4 py-3 rounded-xl border transition-all ${
                  language === 'en' 
                    ? 'border-primary bg-primary/10 text-primary' 
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <span class="text-sm">English</span>
              </button>
            </div>
          </div>

          <!-- Уведомления -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                {#if notifications}
                  <Bell class="h-5 w-5 text-primary" />
                {:else}
                  <BellOff class="h-5 w-5 text-muted-foreground" />
                {/if}
                <div>
                  <h2 class="text-lg font-semibold">Уведомления</h2>
                  <p class="text-sm text-muted-foreground">Получать уведомления о завершении конвертации</p>
                </div>
              </div>
              <button 
                onclick={() => notifications = !notifications}
                class={`relative w-12 h-6 rounded-full transition-colors ${
                  notifications ? 'bg-primary' : 'bg-muted-foreground/20'
                }`}
              >
                <span 
                  class={`absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all ${
                    notifications ? 'left-6' : 'left-0.5'
                  }`}
                />
              </button>
            </div>
          </div>

          <!-- Звук -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                {#if sound}
                  <Volume2 class="h-5 w-5 text-primary" />
                {:else}
                  <VolumeX class="h-5 w-5 text-muted-foreground" />
                {/if}
                <div>
                  <h2 class="text-lg font-semibold">Звук</h2>
                  <p class="text-sm text-muted-foreground">Воспроизводить звук при завершении операций</p>
                </div>
              </div>
              <button 
                onclick={() => sound = !sound}
                class={`relative w-12 h-6 rounded-full transition-colors ${
                  sound ? 'bg-primary' : 'bg-muted-foreground/20'
                }`}
              >
                <span 
                  class={`absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all ${
                    sound ? 'left-6' : 'left-0.5'
                  }`}
                />
              </button>
            </div>
          </div>

          <!-- Автосохранение -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <Save class="h-5 w-5 text-primary" />
                <div>
                  <h2 class="text-lg font-semibold">Автосохранение</h2>
                  <p class="text-sm text-muted-foreground">Автоматически сохранять настройки</p>
                </div>
              </div>
              <button 
                onclick={() => autoSave = !autoSave}
                class={`relative w-12 h-6 rounded-full transition-colors ${
                  autoSave ? 'bg-primary' : 'bg-muted-foreground/20'
                }`}
              >
                <span 
                  class={`absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-all ${
                    autoSave ? 'left-6' : 'left-0.5'
                  }`}
                />
              </button>
            </div>
          </div>

          <!-- Безопасность -->
          <div class="bg-card/50 backdrop-blur-sm rounded-2xl border border-border p-6">
            <div class="flex items-center gap-3 mb-4">
              <Shield class="h-5 w-5 text-primary" />
              <h2 class="text-lg font-semibold">Безопасность</h2>
            </div>
            <p class="text-sm text-muted-foreground mb-3">
              Все данные обрабатываются локально. Никакие данные не передаются на сервер.
            </p>
            <div class="flex items-center gap-2 text-xs text-muted-foreground/60">
              <span class="px-2 py-1 bg-emerald-500/10 rounded border border-emerald-500/20 text-emerald-400">● Защищено</span>
              <span>Локальная обработка</span>
            </div>
          </div>

          <!-- Кнопка сохранения -->
          <button 
            onclick={saveSettings}
            class="w-full px-6 py-4 bg-primary/10 hover:bg-primary/20 rounded-2xl border border-primary/20 text-primary font-semibold transition-all flex items-center justify-center gap-2"
          >
            <Save class="h-5 w-5" />
            Сохранить настройки
          </button>
        </div>

        <div class="text-center mt-8 text-xs text-muted-foreground/40">
          Настройки сохраняются локально на вашем устройстве
        </div>
      </div>

    </main>
  </div>
</ScrollContainer>