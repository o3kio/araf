export const DEFAULT_PAGE = 1;
export const DEFAULT_PAGE_SIZE = 25;

export function parseIntOr(value: string | null, fallback: number): number {
  if (value === null) return fallback;
  const parsed = Number.parseInt(value, 10);
  return Number.isNaN(parsed) ? fallback : parsed;
}

export function clampPageSize(pageSize: number): number {
  return Math.min(Math.max(pageSize, 1), 100);
}
