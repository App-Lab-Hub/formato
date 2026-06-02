<script lang="ts">
  import "../app.css";
  // import { initMonacoWorkers } from "$lib/monaco-workers";

  let { children } = $props();

  // Выполняется только на клиенте (в браузере / WebView Tauri)
  $effect(() => {
    // initMonacoWorkers();
  });
</script>

<div class="bg-background text-foreground">
  {@render children?.()}
</div>
