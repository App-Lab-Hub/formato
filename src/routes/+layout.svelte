<script lang="ts">
  import { invoke } from  "@tauri-apps/api/core";
  import "../app.css";
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(async () => {
    await invoke('set_complete', { task: 'frontend' });
  });
</script>

<div class="bg-background text-foreground">
  {@render children?.()}
</div>