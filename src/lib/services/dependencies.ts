// src/lib/services/dependencies.ts
import { browser } from "$app/environment";

export interface Dependency {
  name: string;
  version: string;
}

export type DependencyGroup = Dependency[];

export interface DependenciesData {
  npm: {
    dependencies: DependencyGroup;
    devDependencies: DependencyGroup;
    optionalDependencies: DependencyGroup;
    peerDependencies: DependencyGroup;
    bundleDependencies: DependencyGroup;
  };
  cargo: {
    dependencies: DependencyGroup;
    devDependencies: DependencyGroup;
    buildDependencies: DependencyGroup;
    targetDependencies: DependencyGroup;
  };
}

// Парсинг Cargo.toml
function parseCargoToml(content: string): {
  deps: Dependency[];
  devDeps: Dependency[];
  buildDeps: Dependency[];
  targetDeps: Dependency[];
} {
  const deps: Dependency[] = [];
  const devDeps: Dependency[] = [];
  const buildDeps: Dependency[] = [];
  const targetDeps: Dependency[] = [];

  let currentSection:
    | "dependencies"
    | "dev-dependencies"
    | "build-dependencies"
    | "target"
    | null = null;

  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (trimmed === "[dependencies]") {
      currentSection = "dependencies";
      continue;
    } else if (trimmed === "[dev-dependencies]") {
      currentSection = "dev-dependencies";
      continue;
    } else if (trimmed === "[build-dependencies]") {
      currentSection = "build-dependencies";
      continue;
    } else if (
      trimmed.startsWith("[target.") &&
      trimmed.includes("dependencies")
    ) {
      currentSection = "target";
      continue;
    } else if (trimmed.startsWith("[")) {
      currentSection = null;
      continue;
    }

    if (
      currentSection &&
      trimmed &&
      !trimmed.startsWith("#") &&
      !trimmed.startsWith("#")
    ) {
      const match = trimmed.match(/^([\w-]+)\s*=\s*["']([^"']+)["']/);
      if (match) {
        const [, name, version] = match;
        if (currentSection === "dependencies") {
          deps.push({ name, version });
        } else if (currentSection === "dev-dependencies") {
          devDeps.push({ name, version });
        } else if (currentSection === "build-dependencies") {
          buildDeps.push({ name, version });
        } else if (currentSection === "target") {
          targetDeps.push({ name, version });
        }
      }
    }
  }

  return { deps, devDeps, buildDeps, targetDeps };
}

export async function loadDependencies(): Promise<DependenciesData> {
  try {
    // Загружаем package.json из static
    const packageJsonResponse = await fetch("/package.json");
    const packageJson = await packageJsonResponse.json();

    // Загружаем Cargo.toml из static
    const cargoTomlResponse = await fetch("/Cargo.toml");
    const cargoTomlContent = await cargoTomlResponse.text();
    const cargo = parseCargoToml(cargoTomlContent);

    return {
      npm: {
        dependencies: Object.entries(packageJson.dependencies || {}).map(
          ([name, version]) => ({ name, version: String(version) }),
        ),
        devDependencies: Object.entries(packageJson.devDependencies || {}).map(
          ([name, version]) => ({ name, version: String(version) }),
        ),
        optionalDependencies: Object.entries(
          packageJson.optionalDependencies || {},
        ).map(([name, version]) => ({ name, version: String(version) })),
        peerDependencies: Object.entries(
          packageJson.peerDependencies || {},
        ).map(([name, version]) => ({ name, version: String(version) })),
        bundleDependencies: (packageJson.bundleDependencies || []).map(
          (name: string) => ({ name, version: "" }),
        ),
      },
      cargo: {
        dependencies: cargo.deps,
        devDependencies: cargo.devDeps,
        buildDependencies: cargo.buildDeps,
        targetDependencies: cargo.targetDeps,
      },
    };
  } catch (e) {
    console.error("Failed to load dependencies:", e);
    return getFallbackDependencies();
  }
}

function getFallbackDependencies(): DependenciesData {
  return {
    npm: {
      dependencies: [
        { name: "@splidejs/svelte-splide", version: "^0.2.0" },
        { name: "@tauri-apps/api", version: "^2.0.0" },
      ],
      devDependencies: [
        { name: "typescript", version: "^5.0.0" },
        { name: "svelte", version: "^5.0.0" },
      ],
      optionalDependencies: [],
      peerDependencies: [],
      bundleDependencies: [],
    },
    cargo: {
      dependencies: [
        { name: "tauri", version: "^2.0.0" },
        { name: "serde", version: "^1.0.0" },
      ],
      devDependencies: [{ name: "tauri-build", version: "^2.0.0" }],
      buildDependencies: [],
      targetDependencies: [],
    },
  };
}
