<script lang="ts">
  import { onMount } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

  let monacoContainer = $state<HTMLElement>();

function fixFindWidgetHeight() {
  const findWidget = document.querySelector('.monaco-editor .find-widget') as HTMLElement;
  if (!findWidget) return;
  
  const isReplaceToggled = findWidget.classList.contains('replaceToggled');
  const currentHeight = parseInt(findWidget.style.height);
  
  if (isReplaceToggled) {
    // Replace открыт
    if (currentHeight <= 70) findWidget.style.height = '67px';
  } else {
    if (currentHeight <= 41) findWidget.style.height = '40px';
  }
}

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

    const currentWindow = WebviewWindow.getCurrent();

    console.log('[Preview Page] Registering once listener for preview-data...');
    
    await currentWindow.once<{ content: string; lang: string; title: string }>('preview-data', (event) => {
      console.log('[Preview Page] Data package arrived! Length:', event.payload.content?.length);
      const { content, lang } = event.payload;

      if (monacoContainer && content) {
        console.log('[Preview Page] Initializing Monaco editor...');
        const editor = monaco.editor.create(monacoContainer, {
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
            loop: true,
            cursorMoveOnType: true,
          },
          fixedOverflowWidgets: true,
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

        // Ctrl+F — открыть поиск + исправить высоту
        editor.addAction({
          id: 'find-with-fix',
          label: 'Find',
          keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyF],
          run: () => {
            editor.getAction('actions.find')?.run();
            setTimeout(fixFindWidgetHeight, 10);
          }
        });

        // MutationObserver — авто-коррекция при toggle Replace
        const findContainer = document.querySelector('.monaco-editor .overlayWidgets');
        if (findContainer) {
          const observer = new MutationObserver(() => {
            fixFindWidgetHeight();
          });
          observer.observe(findContainer, {
            attributes: true,
            attributeFilter: ['class', 'style'],
            subtree: true
          });
        }

        // ==========================================================================
        // УЛУЧШЕННЫЙ АВТОСКРОЛЛ ПО ВСЕМ ОСЯМ (ВВЕРХ/ВНИЗ/ВЛЕВО/ВПРАВО) ПРИ ВЫДЕЛЕНИИ
        // ==========================================================================
        let isMouseDown = false;
        let activeTextarea: HTMLTextAreaElement | null = null;
        let scrollInterval: number | null = null;
        let scrollXDirection = 0; // -1 = влево, 1 = вправо, 0 = стоп
        let scrollYDirection = 0; // -1 = вверх, 1 = вниз, 0 = стоп

        const startAutoscroll = () => {
          if (scrollInterval) return;
          scrollInterval = window.setInterval(() => {
            if (!activeTextarea) return;
            // Двигаем горизонтальную и вертикальную оси независимо
            if (scrollXDirection !== 0) activeTextarea.scrollLeft += scrollXDirection * 8;
            if (scrollYDirection !== 0) activeTextarea.scrollTop += scrollYDirection * 6;
          }, 16); // ~60 FPS
        };

        const stopAutoscroll = () => {
          if (scrollInterval) {
            clearInterval(scrollInterval);
            scrollInterval = null;
          }
          scrollXDirection = 0;
          scrollYDirection = 0;
        };

        if (findContainer) {
          findContainer.addEventListener('mousedown', (e) => {
            const target = e.target as HTMLElement;
            if (target && target.classList.contains('input') && target.tagName === 'TEXTAREA') {
              isMouseDown = true;
              activeTextarea = target as HTMLTextAreaElement;
            }
          });

          window.addEventListener('mousemove', (e) => {
            if (!isMouseDown || !activeTextarea) return;

            const rect = activeTextarea.getBoundingClientRect();
            let needsScroll = false;

            // 1. Проверяем горизонтальные границы (Влево / Вправо)
            if (e.clientX > rect.right) {
              scrollXDirection = 1;
              needsScroll = true;
            } else if (e.clientX < rect.left) {
              scrollXDirection = -1;
              needsScroll = true;
            } else {
              scrollXDirection = 0;
            }

            // 2. Проверяем вертикальные границы (Вверх / Вниз)
            if (e.clientY > rect.bottom) {
              scrollYDirection = 1;
              needsScroll = true;
            } else if (e.clientY < rect.top) {
              scrollYDirection = -1;
              needsScroll = true;
            } else {
              scrollYDirection = 0;
            }

            // Запускаем или останавливаем таймер в зависимости от координат мыши
            if (needsScroll) {
              startAutoscroll();
            } else {
              stopAutoscroll();
            }
          });

          window.addEventListener('mouseup', () => {
            isMouseDown = false;
            activeTextarea = null;
            stopAutoscroll();
          });
        }
        // ==========================================================================

        console.log('[Preview Page] Monaco editor rendered successfully');
      }
    });

    console.log('[Preview Page] Signaling parent window...');
    await currentWindow.emit('preview-ready');
    console.log('[Preview Page] Signal preview-ready fired!');
  });
</script>

<div bind:this={monacoContainer} style="width: 100vw; height: 100vh;" class="preview-page"></div>
