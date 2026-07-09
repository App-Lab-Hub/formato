// src/lib/utils/context-menu.ts
import {
  Menu,
  MenuItem,
  IconMenuItem,
  PredefinedMenuItem,
  Submenu,
} from "@tauri-apps/api/menu";
import { Image } from "@tauri-apps/api/image";
import { toast } from "./toast";

// Флаг, чтобы не показывать несколько меню одновременно
let isMenuShowing = false;

export interface ContextMenuAction {
  label: string;
  action: () => void;
  icon?: string; // Можно использовать NativeIcon или путь к иконке
  disabled?: boolean;
  accelerator?: string; // Горячие клавиши
}

export interface ContextMenuConfig {
  items: ContextMenuAction[];
  onClose?: () => void;
}

export async function showContextMenu(
  e: MouseEvent,
  config: ContextMenuConfig,
): Promise<void> {
  // Предотвращаем множественные меню
  if (isMenuShowing) {
    return;
  }

  e.preventDefault();
  e.stopPropagation();
  e.stopImmediatePropagation();

  const { items, onClose } = config;

  if (items.length === 0) {
    return;
  }

  isMenuShowing = true;

  try {
    const menuItems = [];

    for (let i = 0; i < items.length; i++) {
      const item = items[i];

      // Разделитель
      if (item.label === "---" || item.label === "separator") {
        const separator = await PredefinedMenuItem.new({ item: "Separator" });
        menuItems.push(separator);
        continue;
      }

      // Обычный пункт меню с иконкой
      const menuItem = await MenuItem.new({
        text: item.label,
        enabled: !item.disabled,
        accelerator: item.accelerator,
        action: () => {
          isMenuShowing = false;
          item.action();
        },
      });
      menuItems.push(menuItem);
    }

    const menu = await Menu.new({
      items: menuItems,
    });

    await menu.popup();

    if (onClose) {
      setTimeout(() => {
        isMenuShowing = false;
        onClose();
      }, 100);
    } else {
      setTimeout(() => {
        isMenuShowing = false;
      }, 100);
    }
  } catch (error) {
    console.error("Failed to show context menu:", error);
    toast.error("Не удалось открыть контекстное меню");
    isMenuShowing = false;
  }
}

// Утилита для создания стандартных действий для файлов
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
      label: "🔄 Конвертировать",
      action: () => onConvert(fileId),
      disabled: false,
      accelerator: "CmdOrCtrl+Enter",
    });
  }

  if (isConverted) {
    actions.push({
      label: "👁️ Предпросмотр",
      action: () => onPreview(fileId),
      disabled: false,
      accelerator: "CmdOrCtrl+P",
    });

    actions.push({
      label: "⬇️ Скачать",
      action: () => onDownload(fileId),
      disabled: false,
      accelerator: "CmdOrCtrl+S",
    });
  }

  if (actions.length > 0) {
    actions.push({ label: "---", action: () => {} });
  }

  actions.push({
    label: "🗑️ Удалить",
    action: () => onRemove(fileId),
    disabled: isConverting,
    accelerator: "Delete",
  });

  return actions;
}

// Создание глобального контекстного меню с подменю
export async function createGlobalContextMenu(currentPath: string) {
  const actions: ContextMenuAction[] = [];

  // Навигация
  if (currentPath !== "/") {
    actions.push({
      label: "🏠 На главную",
      action: () => (window.location.href = "/"),
      accelerator: "CmdOrCtrl+H",
    });
  }

  if (currentPath !== "/settings") {
    actions.push({
      label: "⚙️ Настройки",
      action: () => (window.location.href = "/settings"),
      accelerator: "CmdOrCtrl+,",
    });
  }

  if (currentPath !== "/about") {
    actions.push({
      label: "ℹ️ О программе",
      action: () => (window.location.href = "/about"),
    });
  }

  if (currentPath !== "/dependencies") {
    actions.push({
      label: "📦 Зависимости",
      action: () => (window.location.href = "/dependencies"),
    });
  }

  // Добавляем разделитель, если есть навигационные пункты
  if (actions.length > 0) {
    actions.push({ label: "---", action: () => {} });
  }

  // Дополнительные действия
  actions.push({
    label: "🔄 Обновить страницу",
    action: () => {
      setTimeout(() => {
        window.location.reload();
      }, 50);
    },
    accelerator: "CmdOrCtrl+R",
  });

  return actions;
}
