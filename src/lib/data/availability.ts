// src/lib/data/availability.ts

import { invoke } from "@tauri-apps/api/core";

export interface AvailabilityResponse {
  text: string;
  image: string;
  audio: string;
  video: string;
  document: string;
  enable_text_mode: boolean;
  exceptions: Record<string, string[]>; // 🚫 Исключения конвертации
}

export async function getAvailability(
  formatType: string,
): Promise<AvailabilityResponse> {
  try {
    return await invoke<AvailabilityResponse>("get_availability", {
      fromType: formatType,
    });
  } catch (error) {
    // console.error("Failed to get availability:", error);
    return {
      text: "not_available",
      image: "not_available",
      audio: "not_available",
      video: "not_available",
      document: "not_available",
      enable_text_mode: false,
      exceptions: {},
    };
  }
}

export function getAvailabilityStatus(status: string): {
  label: string;
  color: string;
  icon: string;
} {
  switch (status) {
    case "available":
      return { label: "Доступно", color: "text-green-400", icon: "[OK]" };
    case "available_with_ai":
      return { label: "Доступно с AI", color: "text-yellow-400", icon: "🤖" };
    case "not_available":
      return { label: "Недоступно", color: "text-red-400", icon: "❌" };
    default:
      return { label: "Неизвестно", color: "text-gray-400", icon: "❓" };
  }
}

// Получить исключения для конкретного формата
export function getExceptionsForFormat(
  availability: AvailabilityResponse | null,
  formatId: string,
): string[] {
  if (!availability || !availability.exceptions) return [];
  return availability.exceptions[formatId] || [];
}

// Проверить, исключен ли формат для конвертации
export function isFormatExcluded(
  availability: AvailabilityResponse | null,
  sourceFormatId: string,
  targetFormatType: string,
): boolean {
  const exceptions = getExceptionsForFormat(availability, sourceFormatId);
  return exceptions.includes(targetFormatType);
}
