<script lang="ts">
  import "../app.css";
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  import { onMount } from 'svelte';

  let { children } = $props();
  let splashDone = $state(false);

  onMount(() => {
    // SvelteKit полностью загрузился — показываем окно
    import('@tauri-apps/api/core').then(({ invoke }) => {
      invoke('app_ready');
    });
  });
</script>

{#if !splashDone}
  <SplashScreen onComplete={() => splashDone = true} />
{/if}

<div class="bg-background text-foreground">
  {@render children?.()}
</div>