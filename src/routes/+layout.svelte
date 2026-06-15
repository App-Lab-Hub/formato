<script lang="ts">
  import "../app.css";
  import SplashScreen from '$lib/components/SplashScreen.svelte';

  let { children } = $props();
  let splashDone = $state(false);
</script>

{#if !splashDone}
  <SplashScreen onComplete={() => splashDone = true} />
{/if}

<div class="bg-background text-foreground">
  {@render children?.()}
</div>