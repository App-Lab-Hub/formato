<!-- src/lib/components/ScrollContainer.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import 'simplebar/dist/simplebar.css';
  import '$lib/styles/simplebar.css';

  import SimpleBar from 'simplebar';
  import type SimpleBarCore from 'simplebar-core';

  let container: HTMLElement;
  let simpleBarInstance: SimpleBarCore;

  onMount(() => {
    if (!container) return;

    simpleBarInstance = new SimpleBar(container, {
      autoHide: true,
      forceVisible: false,
      clickOnTrack: true,
    });

    // Восстановление позиции
    const scrollEl = simpleBarInstance.getScrollElement();
    if (scrollEl) {
      const savedPos = sessionStorage.getItem('simplebarScrollPosition');
      if (savedPos) {
        const pos = parseInt(savedPos, 10);
        if (pos > 0) {
          setTimeout(() => {
            scrollEl.scrollTop = pos;
          }, 100);
        }
      }

      // Сохранение позиции
      const onScroll = () => {
        sessionStorage.setItem('simplebarScrollPosition', String(scrollEl.scrollTop || 0));
      };
      scrollEl.addEventListener('scroll', onScroll);

      return () => {
        scrollEl.removeEventListener('scroll', onScroll);
        if (simpleBarInstance) {
          simpleBarInstance.unMount();
        }
      };
    }
  });
</script>

<div bind:this={container} class="scroll-container">
  <slot />
</div>