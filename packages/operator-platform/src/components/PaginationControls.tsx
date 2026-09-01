export interface PaginationControlsProps {
  page: number;
  pageSize: number;
  totalPages: number;
  hasMore: boolean;
  total: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
}

export function PaginationControls({
  page,
  pageSize,
  totalPages,
  hasMore,
  total,
  onPageChange,
  onPageSizeChange,
}: PaginationControlsProps) {
  return (
    <nav
      aria-label="Pagination"
      className="araf-pagination-controls"
      style={{ display: "flex", gap: "1rem", alignItems: "center", marginTop: "1rem" }}
    >
      <button
        type="button"
        aria-label="Previous page"
        disabled={page <= 1}
        onClick={() => {
          onPageChange(page - 1);
        }}
      >
        Previous
      </button>
      <span>
        Page {page} of {totalPages} ({total} total)
      </span>
      <button
        type="button"
        aria-label="Next page"
        disabled={!hasMore}
        onClick={() => {
          onPageChange(page + 1);
        }}
      >
        Next
      </button>
      <label>
        Page size:{" "}
        <select
          aria-label="Page size"
          value={pageSize}
          onChange={(e) => {
            onPageSizeChange(Number.parseInt(e.target.value, 10));
          }}
        >
          <option value={10}>10</option>
          <option value={25}>25</option>
          <option value={50}>50</option>
          <option value={100}>100</option>
        </select>
      </label>
    </nav>
  );
}
