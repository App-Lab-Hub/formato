<script lang="ts">
  import { onMount } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

  let monacoContainer = $state<HTMLElement>();

  onMount(async () => {
    console.log('[Preview Page] Mounted');
    
    const searchParams = new URLSearchParams(window.location.search);
    const windowId = searchParams.get('windowId');
    const title = decodeURIComponent(searchParams.get('title') ?? 'Preview');
    document.title = title;
    
    console.log('[Preview Page] Window ID:', windowId);

    if (!windowId) {
      console.error('[Preview Page] Error: windowId parameter missing');
      return;
    }

    // Получаем инстанс текущего открытого окна превью
    const currentWindow = WebviewWindow.getCurrent();

    console.log('[Preview Page] Registering once listener for preview-data...');
    
    // Подписываемся на событие на уровне текущего окна ровно один раз
    await currentWindow.once<{ content: string; lang: string; title: string }>('preview-data', (event) => {
      console.log('[Preview Page] Data package arrived! Length:', event.payload.content?.length);
      const { content, lang } = event.payload;

      if (monacoContainer && content) {
        console.log('[Preview Page] Initializing Monaco editor...');
        monaco.editor.create(monacoContainer, {
          value: content,
          language: lang,
          readOnly: false,
          theme: 'vs-dark',
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          wordWrap: 'on',
          lineNumbers: 'on',
          fontSize: 13,
          lineHeight: 22,
          automaticLayout: true,
          tabSize: 2,
          detectIndentation: true,
          insertSpaces: true,
          padding: { top: 16, bottom: 16 },
          lineNumbersMinChars: 4,
          lineDecorationsWidth: 0,
          renderLineHighlight: 'all',
          renderWhitespace: 'selection',
          matchBrackets: 'always',
          selectionHighlight: false,
          occurrencesHighlight: "off",
          roundedSelection: true,
          'semanticHighlighting.enabled': true,
          folding: true,
          showFoldingControls: 'mouseover',
          overviewRulerBorder: false,
          hideCursorInOverviewRuler: true,
          overviewRulerLanes: 0,
          glyphMargin: true,
          contextmenu: false,
          scrollbar: {
            vertical: 'visible',
            horizontal: 'auto',
            verticalScrollbarSize: 6,
            horizontalScrollbarSize: 6,
            arrowSize: 0,
            useShadows: false,
          },
          stickyScroll: { enabled: false },
          find: {
            addExtraSpaceOnTop: true,
            seedSearchStringFromSelection: 'always',
            autoFindInSelection: 'never',
          },
          largeFileOptimizations: true,
          maxTokenizationLineLength: 100000,
          smoothScrolling: true,
          colorDecorators: true,
          colorDecoratorsLimit: 100000,
          cursorBlinking: 'solid',
          cursorSmoothCaretAnimation: 'off',
          multiCursorModifier: 'alt',
          autoClosingBrackets: 'always',
          autoClosingQuotes: 'always',
          emptySelectionClipboard: true,
          formatOnPaste: false,
          hover: { enabled: true, delay: 300 },
          suggest: {
            showWords: false,
            showSnippets: false,
            showClasses: false,
            showColors: false,
          },
          links: true,
        });
        console.log('[Preview Page] Monaco editor rendered successfully');
      }
    });

    console.log('[Preview Page] Signaling parent window...');
    // Отправляем сигнал наверх текущему объекту окна. 
    // Поскольку родитель слушает именно этот инстанс, событие мгновенно поймается
    await currentWindow.emit('preview-ready');
    console.log('[Preview Page] Signal preview-ready fired!');
  });
</script>

<div bind:this={monacoContainer} style="width: 100vw; height: 100vh;" class="preview-page"></div>
