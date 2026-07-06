<!-- src/routes/preview/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { formatFileSize, formatSize } from '$lib/utils/format';
  import { FileWarning, Settings, ExternalLink } from 'lucide-svelte';
  import ScrollContainer from '$lib/components/ScrollContainer.svelte';
  import type { PageProps } from './$types';
  import { invoke } from '@tauri-apps/api/core';
  import { m } from '$lib/paraglide/messages';

  let { data }: PageProps = $props();

  let monacoContainer = $state<HTMLElement>();
  let errorMessage = $state<string | null>(null);
  let fileSize = $state<number>(0);
  let maxSizeMB = $state<number>(5);

  async function openSettings() {
    const mainWindow = await WebviewWindow.getByLabel('main');
    if (mainWindow) {
      mainWindow.emit('navigate', '/settings');
    }
    WebviewWindow.getCurrent().close();
  }

  function fixFindWidgetHeight() {
    const findWidget = document.querySelector('.monaco-editor .find-widget') as HTMLElement;
    if (!findWidget) return;
    const isReplaceToggled = findWidget.classList.contains('replaceToggled');
    const currentHeight = parseInt(findWidget.style.height);
    if (isReplaceToggled) {
      if (currentHeight <= 70) findWidget.style.height = '67px';
    } else {
      if (currentHeight <= 41) findWidget.style.height = '40px';
    }
  }

  function processPreviewData(content: string, lang: string) {
    if (monacoContainer && content) {
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
        cursorBlinking: 'solid',
        cursorSmoothCaretAnimation: 'off',
        multiCursorModifier: 'alt',
        autoClosingBrackets: 'always',
        autoClosingQuotes: 'always',
        formatOnPaste: false,
        hover: { enabled: true, delay: 300 },
        suggest: { showWords: false, showSnippets: false, showClasses: false, showColors: false },
        links: true,
      });

      editor.addAction({
        id: 'find-with-fix',
        label: 'Find',
        keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyF],
        run: () => {
          editor.getAction('actions.find')?.run();
          setTimeout(fixFindWidgetHeight, 10);
        }
      });

      const findContainer = document.querySelector('.monaco-editor .overlayWidgets');
      if (findContainer) {
        const observer = new MutationObserver(() => fixFindWidgetHeight());
        observer.observe(findContainer, { attributes: true, attributeFilter: ['class', 'style'], subtree: true });
      }

      let isMouseDown = false;
      let activeTextarea: HTMLTextAreaElement | null = null;
      let scrollInterval: number | null = null;
      let scrollXDirection = 0;
      let scrollYDirection = 0;

      const startAutoscroll = () => {
        if (scrollInterval) return;
        scrollInterval = window.setInterval(() => {
          if (!activeTextarea) return;
          if (scrollXDirection !== 0) activeTextarea.scrollLeft += scrollXDirection * 8;
          if (scrollYDirection !== 0) activeTextarea.scrollTop += scrollYDirection * 6;
        }, 16);
      };

      const stopAutoscroll = () => {
        if (scrollInterval) { clearInterval(scrollInterval); scrollInterval = null; }
        scrollXDirection = 0;
        scrollYDirection = 0;
      };

      if (findContainer) {
        findContainer.addEventListener('mousedown', (e) => {
          const target = e.target as HTMLElement;
          if (target?.classList.contains('input') && target.tagName === 'TEXTAREA') {
            isMouseDown = true;
            activeTextarea = target as HTMLTextAreaElement;
          }
        });

        window.addEventListener('mousemove', (e) => {
          if (!isMouseDown || !activeTextarea) return;
          const rect = activeTextarea.getBoundingClientRect();
          let needsScroll = false;
          if (e.clientX > rect.right) { scrollXDirection = 1; needsScroll = true; }
          else if (e.clientX < rect.left) { scrollXDirection = -1; needsScroll = true; }
          else { scrollXDirection = 0; }
          if (e.clientY > rect.bottom) { scrollYDirection = 1; needsScroll = true; }
          else if (e.clientY < rect.top) { scrollYDirection = -1; needsScroll = true; }
          else { scrollYDirection = 0; }
          if (needsScroll) startAutoscroll(); else stopAutoscroll();
        });

        window.addEventListener('mouseup', () => {
          isMouseDown = false;
          activeTextarea = null;
          stopAutoscroll();
        });
      }
    }
  }

  onMount(async () => {
    maxSizeMB = data.maxSize;
    document.title = data.title;
    fileSize = data.size;

    const maxSizeBytes = maxSizeMB === 0 ? Infinity : maxSizeMB * 1024 * 1024;
    
    if (fileSize > maxSizeBytes) {
      const limitText = maxSizeMB === 0 ? m.preview_unlimited() : formatSize(maxSizeMB);
      errorMessage = m.preview_too_large() + ` (${formatFileSize(fileSize)}). ${m.preview_max_size()}: ${limitText}.`;
      return;
    }

    if (data.path) {
      const content = await invoke<string>('read_file_content', { path: decodeURIComponent(data.path) });
      processPreviewData(content, data.lang);
    }
  });
</script>

<div bind:this={monacoContainer} style="width: 100vw; height: 100vh;" class="preview-page">
  {#if errorMessage}
    <ScrollContainer>
      <div class="flex flex-col items-center justify-center min-h-full w-full bg-gradient-to-br from-[#0a0a0f] via-[#1a1025] to-[#0f0a1a] text-white p-6 sm:p-8 md:p-12">
        <div class="max-w-lg w-full text-center my-auto">
          <div class="relative inline-flex mb-6 sm:mb-8">
            <div class="absolute inset-0 blur-2xl bg-gradient-to-r from-red-500/20 via-purple-500/20 to-pink-500/20 rounded-full"></div>
            <div class="relative p-4 sm:p-5 rounded-2xl bg-gradient-to-br from-red-500/10 to-purple-500/10 border border-red-500/20">
              <FileWarning class="h-12 w-12 sm:h-16 sm:w-16 text-red-400" />
            </div>
          </div>

          <h2 class="text-xl sm:text-2xl font-bold mb-3 bg-gradient-to-r from-red-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
            {m.preview_too_large_title()}
          </h2>

          <div class="bg-white/[0.03] backdrop-blur-sm rounded-2xl border border-white/[0.06] p-4 sm:p-6 mb-4 sm:mb-6">
            <p class="text-gray-300 text-xs sm:text-sm leading-relaxed">{errorMessage}</p>
          </div>

          <div class="space-y-2 sm:space-y-3">
            {#if maxSizeMB === 0}
              <p class="text-xs text-gray-500 px-2">{m.preview_unlimited_desc()}</p>
            {:else}
              <button
                onclick={openSettings}
                class="cursor-pointer w-full flex items-center justify-center gap-2 px-4 sm:px-5 py-2.5 sm:py-3 rounded-xl bg-gradient-to-r from-purple-500/20 to-pink-500/20 border border-purple-500/30 text-purple-300 hover:from-purple-500/30 hover:to-pink-500/30 hover:text-purple-200 transition-all duration-300 text-xs sm:text-sm font-medium"
              >
                <Settings class="h-3.5 w-3.5 sm:h-4 sm:w-4" />
                {m.preview_open_settings()}
                <ExternalLink class="h-2.5 w-2.5 sm:h-3 sm:w-3 opacity-60" />
              </button>
              
              <p class="text-xs text-gray-500">{m.preview_increase_limit()}</p>
            {/if}
          </div>

          <div class="mt-6 sm:mt-8 h-px w-24 sm:w-32 mx-auto bg-gradient-to-r from-transparent via-red-500/20 to-transparent"></div>
        </div>
      </div>
    </ScrollContainer>
  {/if}
</div>