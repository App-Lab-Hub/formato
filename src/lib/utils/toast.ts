// src/lib/utils/toast.ts
import Toastify from "toastify-js";
import "toastify-js/src/toastify.css";

type ToastType = "success" | "error" | "warning" | "info";

interface ToastOptions {
  text: string;
  type?: ToastType;
  duration?: number;
  gravity?: "top" | "bottom";
  position?: "left" | "center" | "right";
}

const colors = {
  success: "linear-gradient(to right, #00b09b, #96c93d)",
  error: "linear-gradient(to right, #ff5f6d, #ffc371)",
  warning: "linear-gradient(to right, #f7971e, #ffd200)",
  info: "linear-gradient(to right, #2193b0, #6dd5ed)",
};

export function showToast(options: ToastOptions) {
  const {
    text,
    type = "info",
    duration = 3000,
    gravity = "bottom",
    position = "right",
  } = options;

  Toastify({
    text,
    duration,
    gravity,
    position,
    style: {
      background: colors[type],
      borderRadius: "8px",
      boxShadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
      padding: "12px 20px",
      fontSize: "14px",
      fontFamily: "inherit",
    },
    className: "toastify-custom",
    stopOnFocus: true,
  }).showToast();
}

// Удобные сокращения
export const toast = {
  success: (text: string, duration?: number) =>
    showToast({ text, type: "success", duration }),
  error: (text: string, duration?: number) =>
    showToast({ text, type: "error", duration }),
  warning: (text: string, duration?: number) =>
    showToast({ text, type: "warning", duration }),
  info: (text: string, duration?: number) =>
    showToast({ text, type: "info", duration }),
};
