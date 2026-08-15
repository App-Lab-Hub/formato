import { describe, it, expect } from "vitest";
import {
  getFormatCount,
  getTechStack,
  getTechColorClasses,
  getVersion,
} from "$lib/utils/about";

describe("about utils", () => {
  // ✅ Создаем полный объект формата
  const mockFormats = [
    {
      id: "json",
      name: "JSON",
      format_id: "json",
      extensions: ["json", "hjson"],
      icon: {} as any, // Мок для компонента Svelte
      color: "from-yellow-500/30",
      glow: "shadow-yellow-500/20",
      textColor: "text-yellow-400",
      borderHover: "hover:border-yellow-500/60",
      formatType: "text",
    },
    {
      id: "yaml",
      name: "YAML",
      format_id: "yaml",
      extensions: ["yaml", "yml"],
      icon: {} as any,
      color: "from-blue-500/30",
      glow: "shadow-blue-500/20",
      textColor: "text-blue-400",
      borderHover: "hover:border-blue-500/60",
      formatType: "text",
    },
    {
      id: "pdf",
      name: "PDF",
      format_id: "pdf",
      extensions: ["pdf"],
      icon: {} as any,
      color: "from-red-500/30",
      glow: "shadow-red-500/20",
      textColor: "text-red-400",
      borderHover: "hover:border-red-500/60",
      formatType: "document",
    },
  ];

  // ============================================================
  // getFormatCount
  // ============================================================

  describe("getFormatCount", () => {
    it("should return correct format count", () => {
      expect(getFormatCount(mockFormats)).toBe(3);
    });

    it("should return 0 for empty array", () => {
      expect(getFormatCount([])).toBe(0);
    });
  });

  // ============================================================
  // getTechStack
  // ============================================================

  describe("getTechStack", () => {
    it("should return array of tech stack items", () => {
      const stack = getTechStack();
      expect(stack).toHaveLength(8);
      expect(stack[0].name).toBe("Tauri");
      expect(stack[1].name).toBe("Rust");
    });

    it("should include all expected techs", () => {
      const stack = getTechStack();
      const names = stack.map(t => t.name);
      expect(names).toContain("Tauri");
      expect(names).toContain("Rust");
      expect(names).toContain("SvelteKit");
      expect(names).toContain("TypeScript");
      expect(names).toContain("Tailwind CSS");
    });
  });

  // ============================================================
  // getTechColorClasses
  // ============================================================

  describe("getTechColorClasses", () => {
    it("should return classes for Tauri", () => {
      const classes = getTechColorClasses("Tauri");
      expect(classes).toContain("primary");
      expect(classes).toContain("border");
    });

    it("should return classes for Rust", () => {
      const classes = getTechColorClasses("Rust");
      expect(classes).toContain("cyan");
      expect(classes).toContain("border");
    });

    it("should return classes for SvelteKit", () => {
      const classes = getTechColorClasses("SvelteKit");
      expect(classes).toContain("yellow");
      expect(classes).toContain("border");
    });

    it("should return default classes for unknown tech", () => {
      const classes = getTechColorClasses("Unknown");
      expect(classes).toBeTruthy();
    });
  });

  // ============================================================
  // getVersion
  // ============================================================

  describe("getVersion", () => {
    it("should return version string", () => {
      expect(getVersion()).toBe("v0.1.0");
    });
  });
});
