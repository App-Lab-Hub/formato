// src/lib/utils/toast.ts
import Toastify from "toastify-js";
import "toastify-js/src/toastify.css";
import "$lib/styles/toast.css";

type ToastType = "success" | "error" | "warning" | "info";

interface ToastOptions {
  text: string;
  type?: ToastType;
  duration?: number;
  gravity?: "top" | "bottom";
  position?: "left" | "center" | "right";
  onClick?: () => void;
}

const colors = {
  success: "#22c55e",
  error: "#ef4444",
  warning: "#f59e0b",
  info: "#3b82f6",
};

// SVG иконки в виде data URI
const iconSVGs = {
  success: encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${colors.success}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>`,
  ),
  error: encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${colors.error}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>`,
  ),
  warning: encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${colors.warning}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 20h20L12 2z"/><line x1="12" y1="9" x2="12" y2="13"/><circle cx="12" cy="17" r="0.5" fill="${colors.warning}" stroke="none"/></svg>`,
  ),
  info: encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${colors.info}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>`,
  ),
};

export function showToast(options: ToastOptions) {
  const {
    text,
    type = "info",
    duration = 3000,
    gravity = "bottom",
    position = "right",
    onClick,
  } = options;

  Toastify({
    text,
    duration,
    gravity,
    position,
    stopOnFocus: true,
    close: true, // Включаем кнопку закрытия
    className: "toastify-custom",
    avatar: `data:image/svg+xml,${iconSVGs[type]}`,
    onClick,
    style: {
      background: `linear-gradient(135deg, ${colors[type]}dd, ${colors[type]}99)`,
      borderRadius: "12px",
      boxShadow: "0 8px 32px rgba(0,0,0,0.15)",
      padding: "12px 16px 12px 12px",
      backdropFilter: "blur(10px)",
      border: "1px solid rgba(255,255,255,0.1)",
      fontSize: "14px",
      fontFamily: "inherit",
      display: "flex",
      alignItems: "center",
      gap: "8px",
    },
  }).showToast();
}

export const toast = {
  success: (text: string, duration?: number, onClick?: () => void) =>
    showToast({ text, type: "success", duration, onClick }),
  error: (text: string, duration?: number, onClick?: () => void) =>
    showToast({ text, type: "error", duration, onClick }),
  warning: (text: string, duration?: number, onClick?: () => void) =>
    showToast({ text, type: "warning", duration, onClick }),
  info: (text: string, duration?: number, onClick?: () => void) =>
    showToast({ text, type: "info", duration, onClick }),
};
