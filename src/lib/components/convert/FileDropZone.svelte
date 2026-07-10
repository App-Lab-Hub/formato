<!-- src/lib/components/convert/FileDropZone.svelte -->
<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { onMount } from 'svelte';
  import { Upload } from 'lucide-svelte';
  import { m } from '$lib/paraglide/messages';
  import { toast } from '$lib/utils/toast';

  let {
    sourceFormatId,
    sourceFormatName,
    sourceFormatExtensions = [sourceFormatId],
    onfilesadd,
  }: {
    sourceFormatId: string;
    sourceFormatName: string;
    sourceFormatExtensions?: string[];
    onfilesadd: (files: { path: string; name: string }[], suppressToast?: boolean) => void;
  } = $props();

  let isDragOver = $state(false);
  let dropzoneEl = $state<HTMLElement | null>(null);
  let pendingProcessed = false;

  function processAndAddPaths(paths: string[], suppressToast: boolean = false) {
    const validPaths = paths.filter(path => {
      const ext = path.split('.').pop()?.toLowerCase();
      return ext ? sourceFormatExtensions.includes(ext) : false;
    });

    if (validPaths.length === 0) {
      if (!suppressToast) {
        toast.warning(m.file_no_suitable());
      }
      clearPendingFiles();
      return;
    }

    const newFiles = validPaths.map(path => ({
      path,
      name: path.split('/').pop() || path.split('\\').pop() || path,
    }));

    // 👇 Передаём suppressToast дальше
    onfilesadd(newFiles, suppressToast);
  }

  function clearPendingFiles() {
    const storageKey = `pending_files_${sourceFormatId}`;
    sessionStorage.removeItem(storageKey);
  }



  async function pickFile() {
    const result = await open({
      multiple: true,
      filters: [{ name: `${sourceFormatName} files`, extensions: sourceFormatExtensions }],
    });

    if (result) {
      const paths = Array.isArray(result) ? result : [result];
      const storageKey = `pending_files_${sourceFormatId}`;
      sessionStorage.setItem(storageKey, JSON.stringify(paths));
      // 👇 Пользовательский выбор — показываем тосты
      processAndAddPaths(paths, false);
    }
  }

  function isOverDropzone(x: number, y: number): boolean {
    if (!dropzoneEl) return false;
    const rect = dropzoneEl.getBoundingClientRect();
    return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
  }

  onMount(() => {
    // addPendingFiles();
    
    const webview = getCurrentWebview();
    const unlisten = webview.onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        const pos = event.payload.position;
        const x = pos.x / window.devicePixelRatio;
        const y = pos.y / window.devicePixelRatio;
        const over = isOverDropzone(x, y);
        isDragOver = over;
        document.body.style.cursor = over ? 'copy' : 'default';
      } else if (event.payload.type === 'drop') {
        isDragOver = false;
        document.body.style.cursor = '';
        const pos = event.payload.position;
        const x = pos.x / window.devicePixelRatio;
        const y = pos.y / window.devicePixelRatio;
        if (isOverDropzone(x, y) && event.payload.paths?.length) {
          // 👇 Drag&drop — показываем тосты
          processAndAddPaths(event.payload.paths, false);
        }
      } else if (event.payload.type === 'leave') {
        isDragOver = false;
        document.body.style.cursor = '';
      }
    });

    return () => {
      document.body.style.cursor = '';
      unlisten.then(fn => fn());
    };
  });
</script>

<button
  bind:this={dropzoneEl}
  onclick={pickFile}
  class="group w-full max-w-4xl min-h-[180px] flex flex-col items-center justify-center gap-4 rounded-2xl border-2 border-dashed bg-card/30 duration-300 cursor-pointer transition-all {isDragOver
    ? 'border-primary bg-primary/10'
    : 'dark:border-border light:border-purple-300/40 dark:hover:border-primary/50 light:hover:border-purple-500/60 dark:hover:bg-primary/5 light:hover:bg-purple-200/40'}"
>
  <div class="rounded-full p-4 transition-all duration-500 ease-out group-hover:bg-primary/20 group-hover:text-primary {isDragOver ? 'bg-primary/30 text-primary scale-105' : 'bg-purple-200/60 text-purple-600 dark:bg-purple-500/20 dark:text-purple-300'}">
    <Upload class="h-8 w-8 transition-transform duration-500 ease-out group-hover:scale-110 {isDragOver ? 'scale-110' : ''}" />
  </div>
  <div class="text-center space-y-1">
    {#if isDragOver}
      <p class="text-base font-medium text-primary">{m.drop_files_here()}</p>
    {:else}
      <p class="text-base font-medium dark:text-foreground light:text-purple-800">
        {m.drop_files_text()} <span class="text-primary font-semibold">{sourceFormatName}</span> {m.drop_files_file()}
      </p>
      <p class="text-sm dark:text-muted-foreground light:text-purple-600/70">{m.or_click()}</p>
    {/if}
  </div>
</button>