import { Menu } from "@tauri-apps/api/menu";
import { PredefinedMenuItem } from "@tauri-apps/api/menu";

import { goto } from "$app/navigation";

export async function create_popup() {
  const menu = await Menu.new({
    items: [
      {
        id: "home",
        text: "🏠 На главную",
        accelerator: "CmdOrCtrl+H",
        action: () => {
          setTimeout(() => {
            goto("/");
          }, 100);
        },
      },
      {
        id: "settings",
        text: "⚙️ Настройки",
        accelerator: "CmdOrCtrl+,",
        action: () => {
          setTimeout(() => {
            goto("/settings");
          }, 100);
        },
      },
      {
        id: "about",
        text: "ℹ️ О программе",
        action: () => {
          setTimeout(async () => {
            goto("/about");
          }, 100);
        },
      },
      {
        id: "dependencies",
        text: "📦 Зависимости",
        action: () => {
          setTimeout(() => {
            goto("/dependencies");
          }, 100);
        },
      },
      // Разделитель - используем await
      await PredefinedMenuItem.new({ item: "Separator" }),
      {
        id: "reload",
        text: "🔄 Обновить страницу",
        accelerator: "CmdOrCtrl+R",
        action: () => {
          // goto(window.location.pathname, {
          //   replaceState: true,
          //   invalidateAll: true,
          // });
          setTimeout(() => {
            window.location.replace(window.location.href);
          }, 100);
        },
      },
    ],
  });

  await menu.popup();
}
