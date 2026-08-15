// src/lib/utils/about.ts
import type { Format } from "$lib/types/format";

export function getFormatCount(formats: Format[]): number {
  return formats.length;
}

export function getTechStack(): {
  name: string;
  color: string;
  icon?: string;
}[] {
  return [
    { name: "Tauri", color: "primary" },
    { name: "Rust", color: "cyan" },
    { name: "SvelteKit", color: "yellow" },
    { name: "TypeScript", color: "blue" },
    { name: "Tailwind CSS", color: "purple" },
    { name: "OverlayScrollbars", color: "pink" },
    { name: "Lucide Icons", color: "green" },
    { name: "Splide", color: "red" },
  ];
}

export function getTechColorClasses(tech: string): string {
  const colors: Record<string, string> = {
    Tauri:
      "dark:bg-primary/10 light:bg-purple-300/50 dark:text-primary light:text-purple-700 border dark:border-primary/20 light:border-purple-300/50",
    Rust: "dark:bg-cyan-500/10 light:bg-cyan-200/50 dark:text-cyan-400 light:text-cyan-700 border dark:border-cyan-400/20 light:border-cyan-300/50",
    SvelteKit:
      "dark:bg-yellow-500/10 light:bg-yellow-200/50 dark:text-yellow-400 light:text-yellow-700 border dark:border-yellow-400/20 light:border-yellow-300/50",
    TypeScript:
      "dark:bg-blue-500/10 light:bg-blue-200/50 dark:text-blue-400 light:text-blue-700 border dark:border-blue-400/20 light:border-blue-300/50",
    "Tailwind CSS":
      "dark:bg-purple-500/10 light:bg-purple-300/50 dark:text-purple-400 light:text-purple-700 border dark:border-purple-400/20 light:border-purple-300/50",
    OverlayScrollbars:
      "dark:bg-pink-500/10 light:bg-pink-200/50 dark:text-pink-400 light:text-pink-700 border dark:border-pink-400/20 light:border-pink-300/50",
    "Lucide Icons":
      "dark:bg-green-500/10 light:bg-green-200/50 dark:text-green-400 light:text-green-700 border dark:border-green-400/20 light:border-green-300/50",
    Splide:
      "dark:bg-red-500/10 light:bg-red-200/50 dark:text-red-400 light:text-red-700 border dark:border-red-400/20 light:border-red-300/50",
  };
  return colors[tech] || colors["Tauri"];
}

export function getVersion(): string {
  return "v0.1.0";
}

export function getGithubUrl(): string {
  return "https://github.com/yourusername/formato";
}
