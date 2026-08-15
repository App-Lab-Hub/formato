// src/lib/utils/files.ts
import type { FileInfo } from "$lib/types/files";

export function getTypeLabel(type: string): string {
  return type === "converted" ? "Converted" : "Temporary";
}

export function getTypeColor(type: string): string {
  return type === "converted"
    ? "text-emerald-400 bg-emerald-400/10"
    : "text-amber-400 bg-amber-400/10";
}

export function formatDate(dateStr: string, locale: string = "en"): string {
  try {
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return "Unknown";

    return date.toLocaleString(locale === "ru" ? "ru-RU" : "en-US", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return "Unknown";
  }
}

export function getEmptyMessage(
  searchQuery: string,
  filterType: "all" | "converted" | "temp",
  messages: {
    noSearchResults: string;
    emptyAll: string;
    emptyConverted: string;
    emptyTemp: string;
    empty: string;
  },
): string {
  if (searchQuery) return messages.noSearchResults;

  switch (filterType) {
    case "all":
      return messages.emptyAll;
    case "converted":
      return messages.emptyConverted;
    case "temp":
      return messages.emptyTemp;
    default:
      return messages.empty;
  }
}

export function filterFiles<T extends { name: string; file_type: string }>(
  files: T[],
  searchQuery: string,
  filterType: "all" | "converted" | "temp",
): T[] {
  return files.filter(f => {
    const matchesSearch = f.name
      .toLowerCase()
      .includes(searchQuery.toLowerCase());
    const matchesType = filterType === "all" || f.file_type === filterType;
    return matchesSearch && matchesType;
  });
}

export function getCurrentPageFiles<T>(
  files: T[],
  currentPage: number,
  itemsPerPage: number,
): T[] {
  return files.slice(
    currentPage * itemsPerPage,
    (currentPage + 1) * itemsPerPage,
  );
}

export function getTotalPages(
  totalItems: number,
  itemsPerPage: number,
): number {
  return Math.ceil(totalItems / itemsPerPage);
}

export function goToPage(page: number, totalPages: number): number {
  if (page < 0) return 0;
  if (page >= totalPages) return Math.max(0, totalPages - 1);
  return page;
}

export function getPaginationInfo(
  currentPage: number,
  itemsPerPage: number,
  totalItems: number,
): { from: number; to: number } {
  const from = currentPage * itemsPerPage + 1;
  const to = Math.min((currentPage + 1) * itemsPerPage, totalItems);
  return { from, to };
}

export function getPageNumbers(
  currentPage: number,
  totalPages: number,
): (number | "...")[] {
  const pages: (number | "...")[] = [];

  // Если страниц <= 7 — показываем все
  if (totalPages <= 7) {
    for (let i = 0; i < totalPages; i++) {
      pages.push(i);
    }
    return pages;
  }

  // Иначе — с ellipsis
  for (let i = 0; i < totalPages; i++) {
    if (
      i === 0 ||
      i === totalPages - 1 ||
      (i >= currentPage - 1 && i <= currentPage + 1)
    ) {
      pages.push(i);
    } else if (
      (i === currentPage - 2 && currentPage > 2) ||
      (i === currentPage + 2 && currentPage < totalPages - 3)
    ) {
      pages.push("...");
    }
  }

  return pages;
}
