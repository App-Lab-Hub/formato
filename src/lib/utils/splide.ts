// src/lib/utils/splide.ts
export const SPLIDE_INDEX_KEY = "splide_active_index";

export function normalizeIndex(index: number, length: number): number {
  if (length === 0) return 0;
  if (index < 0) {
    return ((index % length) + length) % length;
  } else {
    return index % length;
  }
}

export function goToConvert(
  formatId: string,
  index: number,
  browser: boolean,
  sessionStorage: Storage,
  goto: (url: string) => void,
  key: string = SPLIDE_INDEX_KEY,
) {
  if (browser && index >= 0) {
    sessionStorage.setItem(key, String(index));
  }
  goto(`/convert/${formatId}`);
}

export function restoreSplidePosition(
  splideInstance: any,
  isRestoring: boolean,
  setRestoring: (value: boolean) => void,
  formatsLength: number,
  sessionStorage: Storage,
  key: string = SPLIDE_INDEX_KEY,
): boolean {
  if (!splideInstance || isRestoring) return false;

  try {
    const savedIndex = sessionStorage.getItem(key);
    if (savedIndex) {
      const index = parseInt(savedIndex);
      if (index >= 0 && index < formatsLength) {
        setRestoring(true);
        splideInstance.go(index, 0);
        setRestoring(false);
        return true;
      }
    }
  } catch (e) {
    setRestoring(false);
  }
  return false;
}

export function shouldClearOnRefresh(performance: Performance): boolean {
  return performance?.navigation?.type === 1;
}
