<!-- src/lib/components/ScrollContainer.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { OverlayScrollbars, type OverlayScrollbars as OverlayScrollbarsType } from "overlayscrollbars";
  import "overlayscrollbars/overlayscrollbars.css";

  const SCROLL_STORAGE_KEY = "scroll_positions";

  let container: HTMLElement;
  let instance: OverlayScrollbarsType | undefined;
  let saveTimeout: number | null = null;
  
  // Определяем ключ для текущей страницы
  const currentPath = window.location.pathname;
  const pageKey = currentPath === '/' ? 'main' : currentPath.replace(/\//g, '_');
  
  // Флаг, что мы уже восстанавливали позицию
  let hasRestored = false;

  // Получаем сохраненную позицию
  function getSavedPosition(): number {
    try {
      const allPositions = JSON.parse(sessionStorage.getItem(SCROLL_STORAGE_KEY) || '{}');
      return allPositions[pageKey] || 0;
    } catch {
      return 0;
    }
  }

  // Сохраняем позицию
  function savePosition(y: number) {
    try {
      const allPositions = JSON.parse(sessionStorage.getItem(SCROLL_STORAGE_KEY) || '{}');
      allPositions[pageKey] = y;
      sessionStorage.setItem(SCROLL_STORAGE_KEY, JSON.stringify(allPositions));
      console.log('[savePosition] Сохранено:', { pageKey, y });
    } catch (e) {
      console.warn('[savePosition] Ошибка:', e);
    }
  }

  // Получаем текущую позицию скролла
  function getCurrentPosition(): number {
    if (!instance || instance.state().destroyed) return 0;
    
    try {
      const scrollOffsetElement = instance.elements().scrollOffsetElement;
      if (scrollOffsetElement) {
        return scrollOffsetElement.scrollTop || 0;
      }
    } catch (e) {}
    
    return 0;
  }

  // Восстанавливаем позицию (принудительно)
  function restorePosition() {
    const savedY = getSavedPosition();
    console.log('[restorePosition] Попытка восстановления:', { pageKey, savedY, hasRestored });
    
    if (savedY > 0 && !hasRestored) {
      // Пробуем восстановить через scrollOffsetElement
      if (instance && !instance.state().destroyed) {
        try {
          const scrollOffsetElement = instance.elements().scrollOffsetElement;
          if (scrollOffsetElement) {
            scrollOffsetElement.scrollTo({
              top: savedY,
              behavior: 'smooth'
            });
            hasRestored = true;
            console.log('[restorePosition] Восстановлено через scrollOffsetElement:', savedY);
            return;
          }
        } catch (e) {}
      }
      
      // Пробуем через viewport
      const viewport = container?.querySelector('.os-viewport') as HTMLElement;
      if (viewport) {
        viewport.scrollTo({
          top: savedY,
          behavior: 'smooth'
        });
        hasRestored = true;
        console.log('[restorePosition] Восстановлено через viewport:', savedY);
        return;
      }
    }
  }

  // Сохраняем текущую позицию
  function saveCurrentPosition() {
    const y = getCurrentPosition();
    if (y > 0) {
      savePosition(y);
    }
  }

  onMount(() => {
    console.log('[ScrollContainer] Монтирование:', pageKey);
    
    // Инициализация
    instance = OverlayScrollbars(container, {
      scrollbars: {
        theme: "os-theme-dark",
        autoHide: "leave",
        clickScroll: "instant",
        dragScroll: true,
      },
    });

    // Событие скролла
    instance?.on("scroll", () => {
      if (saveTimeout) {
        clearTimeout(saveTimeout);
      }
      saveTimeout = window.setTimeout(saveCurrentPosition, 200);
    });

    // При обновлении контента
    instance?.on("updated", () => {
      if (saveTimeout) {
        clearTimeout(saveTimeout);
      }
      saveTimeout = window.setTimeout(saveCurrentPosition, 300);
    });

    // Добавляем обработчики на viewport
    setTimeout(() => {
      const viewport = container?.querySelector('.os-viewport') as HTMLElement;
      if (viewport) {
        viewport.addEventListener('scroll', () => {
          if (saveTimeout) {
            clearTimeout(saveTimeout);
          }
          saveTimeout = window.setTimeout(saveCurrentPosition, 200);
        });
      }
    }, 100);

    // Пытаемся восстановить позицию сразу
    setTimeout(restorePosition, 200);
    setTimeout(restorePosition, 500);
    setTimeout(restorePosition, 1000);
    
    // Для главной страницы даем больше времени (ждем карусель)
    if (currentPath === '/') {
      setTimeout(restorePosition, 2000);
      setTimeout(restorePosition, 3000);
    }
  });

  onDestroy(() => {
    console.log('[ScrollContainer] Уничтожение:', pageKey);
    saveCurrentPosition();
    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
    instance?.destroy();
  });
</script>

<div 
  bind:this={container} 
  class="h-screen w-screen"
  style="overflow: hidden;"
>
  <slot />
</div>