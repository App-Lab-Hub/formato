// src/lib/utils/context-menu.ts
import { Menu, MenuItem, PredefinedMenuItem } from "@tauri-apps/api/menu";
import { toast } from "./toast";

export interface ContextMenuAction {
  label: string;
  action: () => void;
  icon?: string; // Для будущих иконок
  disabled?: boolean;
}

export interface ContextMenuConfig {
  items: ContextMenuAction[];
  onClose?: () => void;
}

export async function showContextMenu(
  e: MouseEvent,
  config: ContextMenuConfig,
): Promise<void> {
  e.preventDefault();
  e.stopPropagation();

  const { items, onClose } = config;

  if (items.length === 0) {
    return;
  }

  try {
    const menuItems = [];

    // Создаем пункты меню
    for (let i = 0; i < items.length; i++) {
      const item = items[i];

      // Разделитель
      if (item.label === "---" || item.label === "separator") {
        const separator = await PredefinedMenuItem.new({ item: "Separator" });
        menuItems.push(separator);
        continue;
      }

      // Обычный пункт
      const menuItem = await MenuItem.new({
        text: item.label,
        enabled: !item.disabled,
        action: () => {
          item.action();
        },
      });
      menuItems.push(menuItem);
    }

    // Создаем меню
    const menu = await Menu.new({
      items: menuItems,
    });

    // Показываем меню
    await menu.popup();

    // Вызываем onClose если есть
    if (onClose) {
      // Немного задерживаем, чтобы меню успело открыться
      setTimeout(onClose, 100);
    }
  } catch (error) {
    console.error("Failed to show context menu:", error);
    toast.error("Не удалось открыть контекстное меню");
  }
}

// Утилита для создания стандартных действий
export function getDefaultActions(
  fileId: string,
  options: {
    onConvert: (id: string) => void;
    onDownload: (id: string) => void;
    onPreview: (id: string) => void;
    onRemove: (id: string) => void;
    isConverted: boolean;
    isConverting: boolean;
  },
): ContextMenuAction[] {
  const {
    onConvert,
    onDownload,
    onPreview,
    onRemove,
    isConverted,
    isConverting,
  } = options;

  const actions: ContextMenuAction[] = [];

  if (!isConverting) {
    actions.push({
      label: "Конвертировать",
      action: () => onConvert(fileId),
      disabled: false,
    });
  }

  if (isConverted) {
    actions.push({
      label: "Предпросмотр",
      action: () => onPreview(fileId),
      disabled: false,
    });

    actions.push({
      label: "Скачать",
      action: () => onDownload(fileId),
      disabled: false,
    });
  }

  if (actions.length > 0) {
    actions.push({ label: "---", action: () => {} });
  }

  actions.push({
    label: "Удалить",
    action: () => onRemove(fileId),
    disabled: isConverting,
  });

  return actions;
}
