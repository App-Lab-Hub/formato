<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { ArrowLeft } from 'lucide-svelte';
  import { formats, type Format } from '$lib/data/formats';
  import { invoke } from '@tauri-apps/api/core';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import 'overlayscrollbars/overlayscrollbars.css';
  import { customScroll } from '$lib/actions/scroll';
  import TooltipProvider from '$lib/components/ui/tooltip/tooltip-provider.svelte';
  import SourceFormatHeader from '$lib/components/convert/SourceFormatHeader.svelte';
  import TargetFormatGrid from '$lib/components/convert/TargetFormatGrid.svelte';
  import FileDropZone from '$lib/components/convert/FileDropZone.svelte';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';


  const sourceFormatId: string = page.params.format!;
  const sourceFormat: Format | undefined = formats.find(f => f.id === sourceFormatId);
  const targetFormats = $derived(formats.filter(f => f.id !== sourceFormatId));

  let selectedTarget = $state<Format | null>(null);
  let files = $state<{ path: string; name: string; id: string }[]>([]);
  let convertingFiles = $state<Set<string>>(new Set());
  let convertedFiles = $state<Map<string, string>>(new Map());
  let counter = $state(0);

  function goBack() { goto('/'); }
  function selectTarget(format: Format) { selectedTarget = format; }
  function handleFilesChange(newFiles: typeof files) { files = newFiles; }

  async function convertOne(index: number) {
    const file = files[index];
    if (!selectedTarget || convertingFiles.has(file.id)) return;
    convertingFiles.add(file.id);
    try {
      const result = await invoke<{ success: boolean; content: string; error: string | null }>(
        'convert_file', { path: file.path, from: sourceFormatId, to: selectedTarget.id }
      );
      if (result.success) convertedFiles = new Map(convertedFiles.set(file.id, result.content));
    } catch (e) { console.error(`Conversion failed: ${file.name}`, e); }
    finally { convertingFiles.delete(file.id); }
  }

  async function convertAll() { for (let i = 0; i < files.length; i++) await convertOne(i); }
  function clearAll() { files = []; convertedFiles = new Map(); }

  async function downloadFile(fileId: string) {
    const savedPath = convertedFiles.get(fileId);
    
    if (!savedPath) {
      console.error('[Download] No path found for fileId:', fileId);
      return;
    }

    // Читаем содержимое
    const content = await invoke<string>('read_file_content', { path: savedPath });

    // Берём имя файла из сохранённого пути (сконвертированного)
    const convertedFileName = savedPath.split('/').pop() || 'file.txt';

    try {
      const filePath = await save({
        defaultPath: convertedFileName,
        title: 'Сохранить файл',
        filters: [{
          name: 'Все файлы',
          extensions: ['*']
        }]
      });

      if (filePath) {
        await writeTextFile(filePath, content);
        console.log('[Download] File saved to:', filePath);
      }
    } catch (e) {
      console.error('[Download] Failed:', e);
    }
  }


  function getMonacoLang(format: string): string {
    const map: Record<string, string> = {
      json: 'json', json5: 'json', yaml: 'yaml', yml: 'yaml',
      xml: 'xml', toml: 'ini', csv: 'plaintext', tsv: 'plaintext',
      ini: 'ini', properties: 'ini', markdown: 'markdown', md: 'markdown',
      html: 'html', hjson: 'json',
    };
    return map[format] ?? 'plaintext';
  }

  async function previewFileFn(fileId: string) {
    const savedPath = convertedFiles.get(fileId) ?? files.find(f => f.id === fileId)?.path;
    if (!savedPath) return;
    
    try {
      console.log('[Preview] Reading file...');
      const raw = await invoke<string>('read_file_content', { path: savedPath });
      const lang = getMonacoLang(selectedTarget?.id ?? sourceFormatId);
      const name = files.find(f => f.id === fileId)?.name ?? 'file';

      const windowId = `preview-${Date.now()}`;
      console.log('[Preview] Window ID:', windowId);

      // 1. Создаем объект окна с чистым коротким URL
      console.log('[Preview] Instantiating window...');
      const webview = new WebviewWindow(windowId, {
        url: `/preview?windowId=${windowId}&title=${encodeURIComponent(name)}`,
        title: name,
        width: 900,
        height: 700,
        resizable: true,
        center: true,
        maximizable: true,
        minimizable: true,
        closable: true,
        transparent: true,
        backgroundColor: { red: 30, green: 30, blue: 30, alpha: 1 },
        theme: 'dark',
        minWidth:400,
        minHeight:300
      });

      // 2. Используем локальный метод .once экземпляра окна.
      // Он сработает один раз, когда дочернее окно подаст сигнал 'preview-ready'
      console.log('[Preview] Registering once listener for preview-ready...');
      await webview.once('preview-ready', async () => {
        console.log('[Preview] Sub-window is ready! Sending 14k+ lines directly...');
        // Метод .emit на инстансе бьет целенаправленно внутрь этого вебвью
        await webview.emit('preview-data', { content: raw, lang, title: name });
        console.log('[Preview] Data sent via window channel');
      });

    } catch (e) {
      console.error('Preview failed:', e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }

  if (!sourceFormat) goto('/');
</script>

<svelte:window on:keydown={handleKeydown} />

<TooltipProvider>
<div class="flex flex-col bg-background text-foreground h-screen" use:customScroll>
  <main class="flex flex-col items-center gap-10 px-8 py-20 max-w-[1700px] mx-auto w-full">

    <button onclick={goBack} class="absolute top-6 left-6 flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors">
      <ArrowLeft class="h-5 w-5" />
      <span class="text-sm">Back</span>
    </button>

    <SourceFormatHeader format={sourceFormat!} />
    <TargetFormatGrid formats={targetFormats} {selectedTarget} onselect={selectTarget} />
    
    <FileDropZone
      sourceFormatId={sourceFormatId}
      sourceFormatName={sourceFormat?.name ?? ''}
      {selectedTarget}
      {files}
      {convertingFiles}
      {convertedFiles}
      {counter}
      onfileschange={handleFilesChange}
      onconvertone={convertOne}
      onconvertall={convertAll}
      onclearall={clearAll}
      onpreview={previewFileFn}
      ondownload={downloadFile}
    />

  </main>
</div>
</TooltipProvider>
