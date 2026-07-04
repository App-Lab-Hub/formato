export function formatSize(mb: number): string {
  if (mb < 1.0) return `${Math.round(mb * 1024)} KB`;
  if (mb >= 1024.0) return `${(mb / 1024.0).toFixed(1)} GB`;
  return `${mb} MB`;
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1000) return `${bytes} B`;
  if (bytes < 1000 * 1000) return `${(bytes / 1000).toFixed(1)} KB`;
  if (bytes < 1000 * 1000 * 1000)
    return `${(bytes / (1000 * 1000)).toFixed(1)} MB`;
  return `${(bytes / (1000 * 1000 * 1000)).toFixed(2)} GB`;
}
