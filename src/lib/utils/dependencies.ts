// src/lib/utils/dependencies.ts
import type { DependenciesData } from "$lib/services/dependencies";

export function hasDependencies(group: any[]): boolean {
  return Array.isArray(group) && group.length > 0;
}

export function getNpmGroups(deps: DependenciesData) {
  const groups: { key: string; label: string; data: any[] }[] = [];
  const npm = deps.npm;

  if (hasDependencies(npm.dependencies)) {
    groups.push({ key: "dependencies", label: "Main", data: npm.dependencies });
  }
  if (hasDependencies(npm.devDependencies)) {
    groups.push({
      key: "devDependencies",
      label: "Dev",
      data: npm.devDependencies,
    });
  }
  if (hasDependencies(npm.optionalDependencies)) {
    groups.push({
      key: "optionalDependencies",
      label: "Optional",
      data: npm.optionalDependencies,
    });
  }
  if (hasDependencies(npm.peerDependencies)) {
    groups.push({
      key: "peerDependencies",
      label: "Peer",
      data: npm.peerDependencies,
    });
  }
  if (hasDependencies(npm.bundleDependencies)) {
    groups.push({
      key: "bundleDependencies",
      label: "Bundled",
      data: npm.bundleDependencies,
    });
  }

  return groups;
}

export function getCargoGroups(deps: DependenciesData) {
  const groups: { key: string; label: string; data: any[] }[] = [];
  const cargo = deps.cargo;

  if (hasDependencies(cargo.dependencies)) {
    groups.push({
      key: "dependencies",
      label: "Main",
      data: cargo.dependencies,
    });
  }
  if (hasDependencies(cargo.devDependencies)) {
    groups.push({
      key: "dev-dependencies",
      label: "Dev",
      data: cargo.devDependencies,
    });
  }
  if (hasDependencies(cargo.buildDependencies)) {
    groups.push({
      key: "build-dependencies",
      label: "Build",
      data: cargo.buildDependencies,
    });
  }
  if (hasDependencies(cargo.targetDependencies)) {
    groups.push({
      key: "target-dependencies",
      label: "Platform",
      data: cargo.targetDependencies,
    });
  }

  return groups;
}

export function getTotalCount(deps: DependenciesData): number {
  let count = 0;
  count += deps.npm.dependencies?.length || 0;
  count += deps.npm.devDependencies?.length || 0;
  count += deps.npm.optionalDependencies?.length || 0;
  count += deps.npm.peerDependencies?.length || 0;
  count += deps.npm.bundleDependencies?.length || 0;
  count += deps.cargo.dependencies?.length || 0;
  count += deps.cargo.devDependencies?.length || 0;
  count += deps.cargo.buildDependencies?.length || 0;
  count += deps.cargo.targetDependencies?.length || 0;
  return count;
}

export function getGroupCount(
  deps: DependenciesData,
  type: "npm" | "cargo",
): number {
  if (type === "npm") {
    return getNpmGroups(deps).reduce((acc, g) => acc + g.data.length, 0);
  } else {
    return getCargoGroups(deps).reduce((acc, g) => acc + g.data.length, 0);
  }
}

export function getLabelMap(): Record<string, string> {
  return {
    dependencies: "Main",
    devDependencies: "Dev",
    optionalDependencies: "Optional",
    peerDependencies: "Peer",
    bundleDependencies: "Bundled",
    "dev-dependencies": "Dev",
    "build-dependencies": "Build",
    "target-dependencies": "Platform",
  };
}

export function getGroupLabel(key: string): string {
  const map = getLabelMap();
  return map[key] || key;
}
