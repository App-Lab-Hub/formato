// src/lib/types/format.ts

import type { Component } from "svelte";

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

export interface Format {
  id: string;
  name: string;
  extensions: string[];
  description: string;
  icon: Component;
  color: string;
  glow: string;
  textColor: string;
  borderHover: string;
}
