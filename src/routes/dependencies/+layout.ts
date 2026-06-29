// src/routes/dependencies/+layout.ts
import type { LayoutLoad } from "./$types";
import {
  loadDependencies,
  type DependenciesData,
} from "$lib/services/dependencies";

export const load: LayoutLoad = async () => {
  // Загружаем зависимости ДО рендера
  const deps = await loadDependencies();

  return {
    deps,
  };
};
