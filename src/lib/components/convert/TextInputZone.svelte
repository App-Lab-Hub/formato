<!-- src/lib/components/convert/TextInputZone.svelte -->
<script lang="ts">
  import { m } from '$lib/paraglide/messages';
  import { onMount } from 'svelte';
  
  let { 
    sourceFormatId, 
    sourceFormatName, 
    selectedTarget = null, 
    isConverting = false, 
    onConvert 
  } = $props<{
    sourceFormatId: string;
    sourceFormatName: string;
    selectedTarget?: { id: string; name: string } | null;
    isConverting?: boolean;
    onConvert: (content: string, fileName: string) => Promise<void>;
  }>();
  
  let textContent = $state('');
  let fileName = $state('');
  let error = $state<string | null>(null);
  let isTextMode = $state(false);
  
  onMount(() => {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
    fileName = `input.${sourceFormatId}`;
  });
  
  function handleFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      const file = input.files[0];
      const reader = new FileReader();
      reader.onload = (event) => {
        const result = event.target?.result;
        if (typeof result === 'string') {
          textContent = result;
          const nameParts = file.name.split('.');
          nameParts.pop();
          fileName = nameParts.join('.') || 'input';
          error = null;
        }
      };
      reader.onerror = () => {
        error = m.file_read_error();
      };
      reader.readAsText(file);
    }
  }
  
  async function handleConvert() {
    if (!selectedTarget) {
      error = m.text_select_format();
      return;
    }
    if (!textContent.trim()) {
      error = m.enter_text_or_file();
      return;
    }
    error = null;
    const finalFileName = fileName || `input.${sourceFormatId}`;
    await onConvert(textContent, finalFileName);
  }
  
  function clearText() {
    textContent = '';
    error = null;
  }
  
  function toggleMode() {
    isTextMode = !isTextMode;
    if (isTextMode) {
      error = null;
    }
  }
</script>

<div class="w-full max-w-4xl mx-auto">
  <div class="flex items-center gap-4 mb-4">
    <button
      onclick={toggleMode}
      class={[
        'px-4 py-2 rounded-lg text-sm font-medium transition-colors text-primary',
        isTextMode ? 'bg-primary/40' : 'bg-primary/20'
      ]}
    >
      📁 {isTextMode ? m.switch_to_file() : m.switch_to_text()}
    </button>
    
    {#if isTextMode}
      <button
        onclick={clearText}
        class="px-3 py-1 text-xs rounded bg-destructive/20 text-destructive hover:bg-destructive/30 transition-colors"
      >
        {m.clear_text()}
      </button>
    {/if}
  </div>
  
  {#if !isTextMode}
    <div class="border-2 border-dashed border-muted-foreground/30 rounded-xl p-8 text-center hover:border-primary/50 transition-colors">
      <div class="flex flex-col items-center gap-4">
        <span class="text-4xl">📄</span>
        <p class="text-sm text-muted-foreground">
          {m.drag_file_here()} <label class="text-primary hover:underline cursor-pointer">{m.select_file()}</label>
        </p>
        <input type="file" accept=".txt,.json,.xml,.csv" onchange={handleFileChange} class="hidden" />
        {#if textContent}
          <div class="w-full max-h-60 overflow-auto bg-muted/30 rounded-lg p-4 text-left text-sm font-mono">
            <div class="text-xs text-muted-foreground mb-2">
              📄 {fileName || m.file()} ({textContent.length} {m.characters()})
            </div>
            <pre class="whitespace-pre-wrap break-all">{textContent.slice(0, 500)}{textContent.length > 500 ? '...' : ''}</pre>
          </div>
        {/if}
      </div>
    </div>
  {:else}
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
  {/if}
  
  <div class="mt-4 flex justify-end">
    <button
      onclick={handleConvert}
      disabled={!textContent.trim() || !selectedTarget || isConverting}
      class="px-6 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
    >
      {#if isConverting}
        <span class="animate-spin">⏳</span> {m.text_converting()}
      {:else}
        🔄 {m.text_convert_to({ format: selectedTarget?.name || '...' })}
      {/if}
    </button>
  </div>
</div>