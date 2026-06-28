// src/lib/actions/scroll.ts
import {
  OverlayScrollbars,
  type OverlayScrollbars as OverlayScrollbarsType,
} from "overlayscrollbars";
import "overlayscrollbars/overlayscrollbars.css";

const SCROLL_STORAGE_KEY_PREFIX = "scroll_pos_";

export function customScroll(node: HTMLElement) {
  let instance: OverlayScrollbarsType | undefined;
  let saveTimeout: number | null = null;
  let currentScrollY = 0;
  let viewportElement: HTMLElement | null = null;

  // Флаг, что пользователь взаимодействует со скроллом
  let isUserInteracting = false;
  // ID таймера для восстановления
  let restoreTimeoutId: number | null = null;
  // Флаг, что восстановление уже было выполнено
  let hasRestored = false;

  // Определяем текущий путь и создаем уникальный ключ
  const currentPath = window.location.pathname;
  // Для главной страницы используем 'main', для остальных - путь без слешей
  const pageKey =
    currentPath === "/" ? "main" : currentPath.replace(/\//g, "_");
  const storageKey = `${SCROLL_STORAGE_KEY_PREFIX}${pageKey}`;

  // Инициализация
  instance = OverlayScrollbars(node, {
    scrollbars: {
      theme: "os-theme-dark",
      autoHide: "leave",
      clickScroll: "instant",
      dragScroll: true,
    },
  });

  // Функция для получения актуальной позиции скролла
  function getScrollPosition(): number {
    if (!viewportElement) {
      viewportElement = node.querySelector(".os-viewport") as HTMLElement;
    }

    if (instance && !instance.state().destroyed) {
      try {
        const scrollOffsetElement = instance.elements().scrollOffsetElement;
        if (
          scrollOffsetElement &&
          scrollOffsetElement.scrollTop !== undefined
        ) {
          return scrollOffsetElement.scrollTop;
        }
      } catch (e) {
        // Игнорируем
      }
    }

    if (viewportElement && viewportElement.scrollTop !== undefined) {
      return viewportElement.scrollTop;
    }

    if (node.scrollTop !== undefined) {
      return node.scrollTop;
    }

    if (instance && !instance.state().destroyed) {
      return instance.state().scrollCoordinates.start.y;
    }

    return 0;
  }

  // Сохранение позиции
  function saveScrollPosition() {
    try {
      const y = getScrollPosition();
      if (y >= 0) {
        sessionStorage.setItem(storageKey, String(y));
        currentScrollY = y;
      }
    } catch (e) {
      // Игнорируем
    }
  }

  // Прерываем восстановление при взаимодействии пользователя
  function cancelRestore() {
    if (restoreTimeoutId) {
      clearTimeout(restoreTimeoutId);
      restoreTimeoutId = null;
    }
    isUserInteracting = true;
  }

  // Восстановление позиции
  function restoreScrollPosition() {
    // Если пользователь уже взаимодействовал - не восстанавливаем
    if (isUserInteracting) {
      return;
    }

    // Если уже восстановили - не повторяем
    if (hasRestored) {
      return;
    }

    try {
      const saved = sessionStorage.getItem(storageKey);

      if (saved) {
        const y = parseFloat(saved);

        if (y > 0) {
          // Отменяем предыдущий таймер восстановления
          if (restoreTimeoutId) {
            clearTimeout(restoreTimeoutId);
            restoreTimeoutId = null;
          }

          // Устанавливаем таймер с задержкой
          restoreTimeoutId = window.setTimeout(() => {
            // Проверяем, не начал ли пользователь взаимодействовать за время задержки
            if (isUserInteracting) {
              restoreTimeoutId = null;
              return;
            }

            let scrolled = false;

            // 1. Через scrollOffsetElement
            if (instance && !instance.state().destroyed) {
              try {
                const scrollOffsetElement =
                  instance.elements().scrollOffsetElement;
                if (scrollOffsetElement) {
                  scrollOffsetElement.scrollTo({
                    top: y,
                    behavior: "smooth",
                  });
                  scrolled = true;
                }
              } catch (e) {
                // Игнорируем
              }
            }

            // 2. Через viewport
            if (!scrolled) {
              viewportElement = node.querySelector(
                ".os-viewport",
              ) as HTMLElement;
              if (viewportElement) {
                viewportElement.scrollTo({
                  top: y,
                  behavior: "smooth",
                });
                scrolled = true;
              }
            }

            // 3. Через сам node
            if (!scrolled) {
              node.scrollTo({
                top: y,
                behavior: "smooth",
              });
              scrolled = true;
            }

            if (scrolled) {
              currentScrollY = y;
              hasRestored = true;
            }

            restoreTimeoutId = null;
          }, 300);
        }
      }
    } catch (e) {
      // Игнорируем
    }
  }

  // Обработчик скролла пользователя
  function handleUserScroll() {
    // Отменяем восстановление при первом скролле пользователя
    cancelRestore();

    // Сбрасываем флаг восстановления, чтобы при следующем релоаде снова восстановилось
    // Но только если пользователь прокрутил больше чем на 50px (случайное касание)
    const currentPos = getScrollPosition();
    if (Math.abs(currentPos - currentScrollY) > 50) {
      hasRestored = false;
    }
  }

  // Подписываемся на событие скролла
  instance?.on("scroll", () => {
    // Помечаем, что пользователь скроллит
    handleUserScroll();

    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
    saveTimeout = window.setTimeout(saveScrollPosition, 150);
  });

  // Сохраняем при обновлении
  instance?.on("updated", () => {
    viewportElement = node.querySelector(".os-viewport") as HTMLElement;

    if (viewportElement) {
      // Добавляем обработчик напрямую на viewport
      viewportElement.addEventListener("scroll", () => {
        handleUserScroll();
        currentScrollY = viewportElement!.scrollTop;
        if (saveTimeout) {
          clearTimeout(saveTimeout);
        }
        saveTimeout = window.setTimeout(saveScrollPosition, 150);
      });

      // Отслеживаем начало взаимодействия с мышью/тачем
      viewportElement.addEventListener("pointerdown", () => {
        cancelRestore();
      });

      // Отслеживаем колесико мыши
      viewportElement.addEventListener("wheel", () => {
        cancelRestore();
      });

      // Отслеживаем касания на мобильных
      viewportElement.addEventListener("touchstart", () => {
        cancelRestore();
      });
    }

    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
    saveTimeout = window.setTimeout(saveScrollPosition, 300);
  });

  // Восстанавливаем позицию
  restoreScrollPosition();

  // Дополнительная попытка через 1 секунду
  setTimeout(() => {
    restoreScrollPosition();
  }, 1000);

  // Для главной страницы дополнительные попытки (ждем загрузку карусели)
  if (currentPath === "/") {
    setTimeout(() => {
      restoreScrollPosition();
    }, 2000);
  }

  return {
    destroy() {
      saveScrollPosition();
      if (saveTimeout) {
        clearTimeout(saveTimeout);
      }
      if (restoreTimeoutId) {
        clearTimeout(restoreTimeoutId);
        restoreTimeoutId = null;
      }
      instance?.destroy();
    },
  };
}
