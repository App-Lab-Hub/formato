import { describe, it, expect } from "vitest";
import {
  hasDependencies,
  getNpmGroups,
  getCargoGroups,
  getTotalCount,
  getGroupCount,
  getGroupLabel,
} from "$lib/utils/dependencies";

describe("dependencies utils", () => {
  const mockDeps = {
    npm: {
      dependencies: [
        { name: "svelte", version: "^5.0.0" },
        { name: "vite", version: "^6.0.0" },
      ],
      devDependencies: [
        { name: "vitest", version: "^2.0.0" },
        { name: "@testing-library/svelte", version: "^5.0.0" },
      ],
      optionalDependencies: [],
      peerDependencies: [],
      bundleDependencies: [],
    },
    cargo: {
      dependencies: [
        { name: "tauri", version: "^2.0.0" },
        { name: "serde", version: "^1.0.0" },
        { name: "serde_json", version: "^1.0.0" },
      ],
      devDependencies: [],
      buildDependencies: [{ name: "tauri-build", version: "^2.0.0" }],
      targetDependencies: [],
    },
  };

  // ============================================================
  // hasDependencies
  // ============================================================

  describe("hasDependencies", () => {
    it("should return true for non-empty array", () => {
      expect(hasDependencies([{ name: "test", version: "1.0.0" }])).toBe(true);
    });

    it("should return false for empty array", () => {
      expect(hasDependencies([])).toBe(false);
    });

    it("should return false for null", () => {
      expect(hasDependencies(null as any)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(hasDependencies(undefined as any)).toBe(false);
    });
  });

  // ============================================================
  // getNpmGroups
  // ============================================================

  describe("getNpmGroups", () => {
    it("should return only groups with dependencies", () => {
      const groups = getNpmGroups(mockDeps);
      expect(groups).toHaveLength(2);
      expect(groups[0].key).toBe("dependencies");
      expect(groups[1].key).toBe("devDependencies");
    });

    it("should include correct data", () => {
      const groups = getNpmGroups(mockDeps);
      expect(groups[0].data).toHaveLength(2);
      expect(groups[0].data[0].name).toBe("svelte");
      expect(groups[1].data).toHaveLength(2);
      expect(groups[1].data[0].name).toBe("vitest");
    });
  });

  // ============================================================
  // getCargoGroups
  // ============================================================

  describe("getCargoGroups", () => {
    it("should return only groups with dependencies", () => {
      const groups = getCargoGroups(mockDeps);
      expect(groups).toHaveLength(2);
      expect(groups[0].key).toBe("dependencies");
      expect(groups[1].key).toBe("build-dependencies");
    });

    it("should include correct data", () => {
      const groups = getCargoGroups(mockDeps);
      expect(groups[0].data).toHaveLength(3);
      expect(groups[0].data[0].name).toBe("tauri");
      expect(groups[1].data).toHaveLength(1);
      expect(groups[1].data[0].name).toBe("tauri-build");
    });
  });

  // ============================================================
  // getTotalCount
  // ============================================================

  describe("getTotalCount", () => {
    it("should count all dependencies", () => {
      const count = getTotalCount(mockDeps);
      expect(count).toBe(2 + 2 + 3 + 1); // npm deps + npm dev + cargo deps + cargo build
    });

    it("should handle empty dependencies", () => {
      const emptyDeps = {
        npm: {
          dependencies: [],
          devDependencies: [],
          optionalDependencies: [],
          peerDependencies: [],
          bundleDependencies: [],
        },
        cargo: {
          dependencies: [],
          devDependencies: [],
          buildDependencies: [],
          targetDependencies: [],
        },
      };
      const count = getTotalCount(emptyDeps);
      expect(count).toBe(0);
    });
  });

  // ============================================================
  // getGroupCount
  // ============================================================

  describe("getGroupCount", () => {
    it("should count npm dependencies", () => {
      const count = getGroupCount(mockDeps, "npm");
      expect(count).toBe(4); // 2 deps + 2 devDeps
    });

    it("should count cargo dependencies", () => {
      const count = getGroupCount(mockDeps, "cargo");
      expect(count).toBe(4); // 3 deps + 1 build
    });

    it("should handle empty dependencies", () => {
      const emptyDeps = {
        npm: {
          dependencies: [],
          devDependencies: [],
          optionalDependencies: [],
          peerDependencies: [],
          bundleDependencies: [],
        },
        cargo: {
          dependencies: [],
          devDependencies: [],
          buildDependencies: [],
          targetDependencies: [],
        },
      };
      expect(getGroupCount(emptyDeps, "npm")).toBe(0);
      expect(getGroupCount(emptyDeps, "cargo")).toBe(0);
    });
  });

  // ============================================================
  // getGroupLabel
  // ============================================================

  describe("getGroupLabel", () => {
    it("should return correct labels for npm groups", () => {
      expect(getGroupLabel("dependencies")).toBe("Main");
      expect(getGroupLabel("devDependencies")).toBe("Dev");
      expect(getGroupLabel("optionalDependencies")).toBe("Optional");
      expect(getGroupLabel("peerDependencies")).toBe("Peer");
      expect(getGroupLabel("bundleDependencies")).toBe("Bundled");
    });

    it("should return correct labels for cargo groups", () => {
      expect(getGroupLabel("dev-dependencies")).toBe("Dev");
      expect(getGroupLabel("build-dependencies")).toBe("Build");
      expect(getGroupLabel("target-dependencies")).toBe("Platform");
    });

    it("should return original key if not found", () => {
      expect(getGroupLabel("unknown-group")).toBe("unknown-group");
    });
  });
});
