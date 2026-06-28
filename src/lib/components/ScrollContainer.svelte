<!-- src/lib/components/ScrollContainer.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { OverlayScrollbars } from "overlayscrollbars";
  import "overlayscrollbars/overlayscrollbars.css";

  let container: HTMLElement;
  let instance: any;
  let saveTimeout: number | null = null;
  let isRestoring = false;

  const STORAGE_KEY = 'scroll_positions';

  function getSavedPosition(): number {
    try {
      const all = JSON.parse(sessionStorage.getItem(STORAGE_KEY) || '{}');
      const path = window.location.pathname;
      const key = path.replace(/\//g, '_') || 'root';
      return all[key] ?? 0;
    } catch {
      return 0;
    }
  }

  function savePosition(y: number) {
    try {
      const all = JSON.parse(sessionStorage.getItem(STORAGE_KEY) || '{}');
      const path = window.location.pathname;
      const key = path.replace(/\//g, '_') || 'root';
      all[key] = y;
      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(all));
    } catch {}
  }

  function getCurrentPosition(): number {
    if (!instance || instance.state().destroyed) return 0;
    try {
      const el = instance.elements().scrollOffsetElement;
      return el?.scrollTop || 0;
    } catch {
      return 0;
    }
  }

  function getMaxScroll(): number {
    if (!instance || instance.state().destroyed) return 0;
    try {
      const el = instance.elements().scrollOffsetElement;
      if (!el) return 0;
      return Math.max(0, el.scrollHeight - el.clientHeight);
    } catch {
      return 0;
    }
  }

  function restorePosition() {
    const y = getSavedPosition();
    const maxY = getMaxScroll();
    
    if (instance && !instance.state().destroyed) {
      try {
        const el = instance.elements().scrollOffsetElement;
        if (el && y >= 0 && maxY > 0) {
          const targetY = Math.min(y, maxY);
          if (Math.abs(el.scrollTop - targetY) > 5) {
            isRestoring = true;
            el.scrollTop = targetY;
            setTimeout(() => {
              isRestoring = false;
            }, 100);
          }
        }
      } catch {}
    }
  }

  onMount(() => {
    instance = OverlayScrollbars(container, {
      scrollbars: {
        theme: "os-theme-dark",
        autoHide: "leave",
        clickScroll: "instant",
        dragScroll: true,
      },
    });

    instance?.on("scroll", () => {
      if (saveTimeout) clearTimeout(saveTimeout);
      saveTimeout = window.setTimeout(() => {
        const y = getCurrentPosition();
        // Сохраняем любую позицию, если это не процесс восстановления
        if (!isRestoring) {
          savePosition(y);
        }
      }, 200);
    });

    requestAnimationFrame(() => {
      restorePosition();
    });
  });

  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
    instance?.destroy();
  });
</script>

<div 
  bind:this={container} 
  class="h-screen w-screen overflow-hidden"
>
  <slot />
</div>