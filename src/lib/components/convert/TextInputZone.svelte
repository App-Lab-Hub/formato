<!-- src/lib/components/convert/TextInputZone.svelte -->
<script lang="ts">
  import { m } from '$lib/paraglide/messages';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { FileText, Plus, LoaderCircle, Type, Trash2, X } from 'lucide-svelte';
  
  let { 
    sourceFormatId, 
    sourceFormatName, 
    onfilesadd,
  } = $props<{
    sourceFormatId: string;
    sourceFormatName: string;
    onfilesadd: (files: { path: string; name: string }[], suppressToast?: boolean) => void;
  }>();
  
  let textContent = $state('');
  let fileName = $state('');
  let error = $state<string | null>(null);
  let isProcessing = $state(false);
  
  onMount(() => {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
    fileName = `input`;
  });
  
  async function handleAdd() {
    if (!textContent.trim()) {
      error = m.enter_text_or_file();
      return;
    }
    if (isProcessing) return;
    
    error = null;
    isProcessing = true;
    
    try {
      const fullFileName = fileName.trim() || 'input';
      const tempPath = await invoke<string>('create_temp_file', {
        content: textContent,
        extension: sourceFormatId,
        name: fullFileName
      });
      
      onfilesadd([{
        path: tempPath,
        name: `${fullFileName}.${sourceFormatId}`,
      }], false);
      
      textContent = '';
      fileName = 'input';
      error = null;
    } catch (e) {
      console.error('Failed to create text file:', e);
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
  <div class="border-2 border-dashed border-muted rounded-2xl p-6 bg-card/30 hover:border-primary/50 transition-colors duration-300">
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-3">
        <div class="p-2 rounded-lg bg-primary/10 text-primary">
          <Type class="h-5 w-5" />
        </div>
        <label class="text-sm font-medium text-muted-foreground">{m.file_name()}:</label>
        <div class="flex items-center bg-background border border-muted rounded-lg px-3 py-1.5 focus-within:ring-2 focus-within:ring-primary transition-shadow">
          <input
            type="text"
            bind:value={fileName}
            placeholder="input"
            class="bg-transparent outline-none text-sm w-48"
          />
          <span class="text-sm text-muted-foreground ml-1">.{sourceFormatId}</span>
        </div>
      </div>
      <div class="flex items-center gap-2 text-xs text-muted-foreground bg-background/50 px-3 py-1 rounded-full">
        <FileText class="h-3.5 w-3.5" />
        <span>{textContent.length}</span>
        <span>{m.characters()}</span>
      </div>
    </div>
    
    <div class="relative">
      <textarea
        bind:value={textContent}
        placeholder={m.enter_text_prompt({ format: sourceFormatName })}
        class="w-full h-80 min-h-[200px] bg-background border border-muted rounded-xl p-4 font-mono text-sm resize-y focus:outline-none focus:ring-2 focus:ring-primary transition-shadow placeholder:text-muted-foreground/50"
        spellcheck="false"
      />
    </div>
    
    {#if error}
      <div class="mt-3 text-sm text-destructive bg-destructive/10 px-4 py-2 rounded-lg border border-destructive/20 flex items-center gap-2">
        <X class="h-4 w-4" />
        {error}
      </div>
    {/if}
  </div>
  
  <div class="mt-4 flex justify-end gap-3">
    <button
      onclick={clearText}
      disabled={isProcessing || !textContent}
      class="px-4 py-2 bg-muted text-muted-foreground rounded-lg hover:bg-muted/80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
    >
      <Trash2 class="h-4 w-4" />
      {m.clear_text()}
    </button>
    <button
      onclick={handleAdd}
      disabled={!textContent.trim() || isProcessing}
      class="px-6 py-2 bg-primary text-white rounded-lg hover:bg-violet-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 shadow-sm shadow-violet-500/20"
    >
      {#if isProcessing}
        <LoaderCircle class="h-4 w-4 animate-spin" />
        {m.adding_text()}
      {:else}
        <Plus class="h-4 w-4" />
        {m.add_to_list()}
      {/if}
    </button>
  </div>
</div>