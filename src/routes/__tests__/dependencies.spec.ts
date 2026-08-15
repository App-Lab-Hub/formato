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

    it("should include optionalDependencies when present", () => {
      const deps = {
        npm: {
          dependencies: [],
          devDependencies: [],
          optionalDependencies: [{ name: "optional", version: "1.0.0" }],
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
      const groups = getNpmGroups(deps);
      expect(groups).toHaveLength(1);
      expect(groups[0].key).toBe("optionalDependencies");
      expect(groups[0].label).toBe("Optional");
    });

    it("should include peerDependencies when present", () => {
      const deps = {
        npm: {
          dependencies: [],
          devDependencies: [],
          optionalDependencies: [],
          peerDependencies: [{ name: "peer", version: "1.0.0" }],
          bundleDependencies: [],
        },
        cargo: {
          dependencies: [],
          devDependencies: [],
          buildDependencies: [],
          targetDependencies: [],
        },
      };
      const groups = getNpmGroups(deps);
      expect(groups).toHaveLength(1);
      expect(groups[0].key).toBe("peerDependencies");
      expect(groups[0].label).toBe("Peer");
    });

    it("should include bundleDependencies when present", () => {
      const deps = {
        npm: {
          dependencies: [],
          devDependencies: [],
          optionalDependencies: [],
          peerDependencies: [],
          bundleDependencies: [{ name: "bundled", version: "1.0.0" }],
        },
        cargo: {
          dependencies: [],
          devDependencies: [],
          buildDependencies: [],
          targetDependencies: [],
        },
      };
      const groups = getNpmGroups(deps);
      expect(groups).toHaveLength(1);
      expect(groups[0].key).toBe("bundleDependencies");
      expect(groups[0].label).toBe("Bundled");
    });

    it("should handle empty npm object", () => {
      const deps = {
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
      const groups = getNpmGroups(deps);
      expect(groups).toHaveLength(0);
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

    it("should include devDependencies when present", () => {
      const deps = {
        npm: {
          dependencies: [],
          devDependencies: [],
          optionalDependencies: [],
          peerDependencies: [],
          bundleDependencies: [],
        },
        cargo: {
          dependencies: [],
          devDependencies: [{ name: "dev", version: "1.0.0" }],
          buildDependencies: [],
          targetDependencies: [],
        },
      };
      const groups = getCargoGroups(deps);
      expect(groups).toHaveLength(1);
      expect(groups[0].key).toBe("dev-dependencies");
      expect(groups[0].label).toBe("Dev");
    });

    it("should include targetDependencies when present", () => {
      const deps = {
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
          targetDependencies: [{ name: "target", version: "1.0.0" }],
        },
      };
      const groups = getCargoGroups(deps);
      expect(groups).toHaveLength(1);
      expect(groups[0].key).toBe("target-dependencies");
      expect(groups[0].label).toBe("Platform");
    });

    it("should handle empty cargo object", () => {
      const deps = {
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
      const groups = getCargoGroups(deps);
      expect(groups).toHaveLength(0);
    });
  });

  // ============================================================
  // getTotalCount
  // ============================================================

  describe("getTotalCount", () => {
    it("should count all dependencies", () => {
      const count = getTotalCount(mockDeps);
      expect(count).toBe(2 + 2 + 3 + 1);
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

    it("should count only non-empty groups", () => {
      const deps = {
        npm: {
          dependencies: [{ name: "a", version: "1.0.0" }],
          devDependencies: [],
          optionalDependencies: [],
          peerDependencies: [],
          bundleDependencies: [],
        },
        cargo: {
          dependencies: [{ name: "b", version: "1.0.0" }],
          devDependencies: [],
          buildDependencies: [],
          targetDependencies: [],
        },
      };
      expect(getTotalCount(deps)).toBe(2);
    });
  });

  // ============================================================
  // getGroupCount
  // ============================================================

  describe("getGroupCount", () => {
    it("should count npm dependencies", () => {
      const count = getGroupCount(mockDeps, "npm");
      expect(count).toBe(4);
    });

    it("should count cargo dependencies", () => {
      const count = getGroupCount(mockDeps, "cargo");
      expect(count).toBe(4);
    });

    it("should handle empty dependencies for npm", () => {
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
    });

    it("should handle empty dependencies for cargo", () => {
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
