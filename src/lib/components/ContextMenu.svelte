<!-- src/lib/components/ContextMenu.svelte -->
<script lang="ts">
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';

  let { 
    items, 
    onAction 
  }: { 
    items: { label: string; icon?: string; action: string; danger?: boolean }[];
    onAction: (action: string, context: any) => void;
  } = $props();

  let isOpen = $state(false);
  let position = $state({ x: 0, y: 0 });
  let context = $state<any>(null);
  let menuElement = $state<HTMLElement | null>(null);

  // Проверяем, что мы в браузере и окно активно
  function openMenu(event: MouseEvent, ctx: any) {
    if (!browser) return;
    
    event.preventDefault();
    event.stopPropagation();

    console.log('🖱️ Открываем контекстное меню:', event.clientX, event.clientY);
    
    context = ctx;
    position = { 
      x: Math.min(event.clientX, window.innerWidth - 200), 
      y: Math.min(event.clientY, window.innerHeight - 300) 
    };
    isOpen = true;
  }

  function closeMenu() {
    isOpen = false;
    context = null;
  }

  function handleAction(action: string) {
    onAction(action, context);
    closeMenu();
  }

  function handleGlobalClick(event: MouseEvent) {
    if (!isOpen) return;
    if (event.button === 0 && menuElement && !menuElement.contains(event.target as Node)) {
      closeMenu();
    }
  }

  // Экспортируем функции для использования в других компонентах
  export function getContextMenuHandlers() {
    return {
      onContextMenu: (event: MouseEvent, ctx: any) => openMenu(event, ctx),
    };
  }

  // Экспортируем openMenu для глобального использования
  export function openGlobalMenu(event: MouseEvent, ctx: any) {
    openMenu(event, ctx);
  }

  onMount(() => {
    if (!browser) return;

    console.log('[OK] ContextMenu mounted');
    
    // Отключаем системное контекстное меню
    const preventContextMenu = (e: Event) => {
      e.preventDefault();
    };
    
    document.addEventListener('contextmenu', preventContextMenu, { capture: true });
    
    return () => {
      document.removeEventListener('contextmenu', preventContextMenu, { capture: true });
    };
  });
</script>

<svelte:window on:mousedown={handleGlobalClick} />


{#if isOpen}
  <div
    bind:this={menuElement}
    transition:fade={{ duration: 150 }}
    role="menu"
    tabindex="-1"
    class="fixed z-[9999] bg-[#1e1e2e] border border-[#45475a] rounded-lg p-1 min-w-[180px] shadow-lg"
    style="top: {position.y}px; left: {position.x}px;"
    oncontextmenu={(e) => e.preventDefault()}
  >
    {#each items as item}
      <button
        role="menuitem"
        onclick={() => handleAction(item.action)}
        class="w-full text-left px-3 py-2 text-sm bg-transparent border-none rounded-md hover:bg-[#313244] cursor-pointer
               {item.danger ? 'text-[#f38ba8]' : 'text-[#cdd6f4]'}"
      >
        {#if item.icon}
          <span class="mr-2" aria-hidden="true">{item.icon}</span>
        {/if}
        {item.label}
      </button>
    {/each}
  </div>
{/if}