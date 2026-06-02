import { OverlayScrollbars } from "overlayscrollbars";
import "overlayscrollbars/overlayscrollbars.css";

export function customScroll(node: HTMLElement) {
  const instance = OverlayScrollbars(node, {
    scrollbars: {
      theme: "os-theme-dark",
      autoHide: "leave",
      clickScroll: "instant",
      dragScroll: true,
    },
  });

  return {
    destroy() {
      instance?.destroy();
    },
  };
}
