// src/lib/types/format.ts

import type { ComponentType } from "svelte";

// Тип с бекенда (уже готовый JSON)
export interface FormatDB {
  format_id: string;
  name: string;
  extensions: string[];
  description: string;
  icon: string;
  color: string;
  glow: string;
  text_color: string;
  border_hover: string;
}

// Тип для фронтенда (с Svelte компонентами)
export interface Format {
  id: string;
  name: string;
  extensions: string[];
  description: string;
  icon: ComponentType;
  color: string;
  glow: string;
  textColor: string;
  borderHover: string;
}
