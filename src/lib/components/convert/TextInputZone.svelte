<!-- src/lib/components/convert/TextInputZone.svelte -->
<script lang="ts">
  import { m } from '$lib/paraglide/messages';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  let { 
    sourceFormatId, 
    sourceFormatName, 
    selectedTarget = null, 
    isConverting = false, 
    onfilesadd,
  } = $props<{
    sourceFormatId: string;
    sourceFormatName: string;
    selectedTarget?: { id: string; name: string } | null;
    isConverting?: boolean;
    onfilesadd: (files: { path: string; name: string }[], suppressToast?: boolean) => void;
  }>();
  
  let textContent = $state('');
  let fileName = $state('');
  let error = $state<string | null>(null);
  let isProcessing = $state(false);
  
  onMount(() => {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
    fileName = `input.${sourceFormatId}`;
  });
  
  async function handleConvert() {
    if (!selectedTarget) {
      error = m.text_select_format();
      return;
    }
    if (!textContent.trim()) {
      error = m.enter_text_or_file();
      return;
    }
    if (isProcessing) return;
    
    error = null;
    isProcessing = true;
    
    try {
      const tempPath = await invoke<string>('create_temp_file', {
        content: textContent,
        extension: sourceFormatId,
        name: fileName || 'input'
      });
      
      // 👇 Пользовательский ввод — показываем тосты
      onfilesadd([{
        path: tempPath,
        name: fileName || `input.${sourceFormatId}`,
      }], false);
      
      textContent = '';
      error = null;
    } catch (e) {
      console.error('Text conversion failed:', e);
      error = m.text_convert_error();
    } finally {
      isProcessing = false;
    }
  }
  
  function clearText() {
    textContent = '';
    error = null;
  }
</script>

<div class="w-full max-w-4xl mx-auto">
  <div class="border-2 border-muted rounded-xl p-4 bg-muted/10">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-3">
        <label class="text-sm font-medium">{m.file_name()}:</label>
        <input
          type="text"
          bind:value={fileName}
          placeholder="input"
          class="bg-background border border-muted rounded px-3 py-1 text-sm w-48 focus:outline-none focus:ring-2 focus:ring-primary"
        />
        <span class="text-sm text-muted-foreground">.{sourceFormatId}</span>
      </div>
      <div class="text-xs text-muted-foreground">
        {textContent.length} {m.characters()}
      </div>
    </div>
    
    <textarea
      bind:value={textContent}
      placeholder={m.enter_text_prompt({ format: sourceFormatName })}
      class="w-full h-80 min-h-[200px] bg-background border border-muted rounded-lg p-4 font-mono text-sm resize-y focus:outline-none focus:ring-2 focus:ring-primary transition-shadow"
      spellcheck="false"
    />
    
    {#if error}
      <div class="mt-3 text-sm text-destructive">{error}</div>
    {/if}
  </div>
  
  <div class="mt-4 flex justify-end gap-2">
    <button
      onclick={clearText}
      disabled={isProcessing}
      class="px-4 py-2 bg-muted text-muted-foreground rounded-lg hover:bg-muted/80 transition-colors disabled:opacity-50"
    >
      {m.clear_text()}
    </button>
    <button
      onclick={handleConvert}
      disabled={!textContent.trim() || !selectedTarget || isConverting || isProcessing}
      class="px-6 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
    >
      {#if isConverting || isProcessing}
        <span class="animate-spin">⏳</span> {m.text_converting()}
      {:else}
        🔄 {m.text_convert_to({ format: selectedTarget?.name || '...' })}
      {/if}
    </button>
  </div>
</div>