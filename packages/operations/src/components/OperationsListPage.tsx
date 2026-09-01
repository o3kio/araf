import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  Button,
  type TableColumnDefinition,
} from "@araf/ui";
import { Link } from "react-router";
import { useOperations } from "../hooks/useOperations";
import { useOperationTransport } from "../hooks/useOperationTransport";
import { OperationStatus } from "./OperationStatus";
import type { Operation, OperationState } from "@araf/api-client";
import type { OperationsFilters } from "../hooks/useOperations";
import { errorMessage, errorCorrelationId } from "../errors";

const STATE_OPTIONS: { value: OperationState | ""; label: string }[] = [
  { value: "", label: "All states" },
  { value: "pending", label: "Pending" },
  { value: "running", label: "Running" },
  { value: "succeeded", label: "Succeeded" },
  { value: "failed", label: "Failed" },
];

function formatTimestamp(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function OperationsListPage() {
  const {
    collection,
    filters,
    loading,
    error,
    setPage,
    setPageSize,
    setFilter,
    clearFilters,
    refresh,
  } = useOperations();

  const terminal = collection?.items.every(
    (op) => op.state === "succeeded" || op.state === "failed",
  );
  useOperationTransport(refresh, !terminal && !loading && !error);

  const columns: TableColumnDefinition<Operation>[] = [
    {
      id: "id",
      header: "ID",
      cell: (op) => <Link to={`/operations/${encodeURIComponent(op.id)}`}>{op.id}</Link>,
      isRowHeader: true,
    },
    { id: "action", header: "Action", cell: (op) => op.action },
    {
      id: "state",
      header: "State",
      cell: (op) => <OperationStatus state={op.state} />,
    },
    {
      id: "resource",
      header: "Resource",
      cell: (op) =>
        op.resourceType && op.resourceId ? `${op.resourceType}/${op.resourceId}` : "—",
    },
    {
      id: "scope",
      header: "Scope",
      cell: (op) => [op.projectId, op.regionId].filter(Boolean).join(" / ") || "—",
    },
    {
      id: "started",
      header: "Started",
      cell: (op) => formatTimestamp(op.startedAt),
    },
    {
      id: "updated",
      header: "Updated",
      cell: (op) => formatTimestamp(op.updatedAt),
    },
  ];

  const totalPages = collection ? Math.ceil(collection.total / collection.pageSize) : 0;

  return (
    <section aria-label="Operations list">
      <Header variant="h1" headingLevel="h1">
        Operations
      </Header>

      {error ? (
        <ErrorState
          title="Could not load operations"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {!error && loading && !collection ? <LoadingState message="Loading operations..." /> : null}

      {!error && (
        <>
          <FilterControls filters={filters} onSetFilter={setFilter} onClearFilters={clearFilters} />

          <Table<Operation>
            items={collection?.items ?? []}
            columnDefinitions={columns}
            trackingId="id"
            loading={loading}
            loadingText="Loading operations..."
            empty={
              <EmptyState
                title="No operations"
                description="There are no operations matching the current filters."
              />
            }
            ariaLabels={{ tableLabel: "Operations table" }}
          />

          {collection && totalPages > 0 ? (
            <PaginationControls
              page={collection.page + 1}
              pageSize={collection.pageSize}
              totalPages={totalPages}
              hasMore={collection.hasMore}
              total={collection.total}
              onPageChange={setPage}
              onPageSizeChange={setPageSize}
            />
          ) : null}
        </>
      )}
    </section>
  );
}

interface FilterControlsProps {
  filters: OperationsFilters;
  onSetFilter: <K extends keyof OperationsFilters>(key: K, value: OperationsFilters[K]) => void;
  onClearFilters: () => void;
}

function FilterControls({ filters, onSetFilter, onClearFilters }: FilterControlsProps) {
  const textFilter = (label: string, key: keyof OperationsFilters, placeholder: string) => (
    <label key={key} style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
      {label}
      <input
        type="search"
        aria-label={label}
        value={filters[key]}
        placeholder={placeholder}
        onChange={(e) => {
          onSetFilter(key, e.target.value as OperationsFilters[typeof key]);
        }}
      />
    </label>
  );

  return (
    <div
      className="araf-operations-filters"
      style={{
        display: "flex",
        gap: "1rem",
        margin: "1rem 0",
        flexWrap: "wrap",
        alignItems: "end",
      }}
    >
      <label style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
        State
        <select
          aria-label="State"
          value={filters.state ?? ""}
          onChange={(e) => {
            const value = e.target.value;
            onSetFilter("state", value === "" ? undefined : (value as OperationState));
          }}
        >
          {STATE_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>

      {textFilter("Action", "action", "Filter by action...")}
      {textFilter("Resource type", "resourceType", "Filter by resource type...")}
      {textFilter("Resource ID", "resourceId", "Filter by resource ID...")}
      {textFilter("Project", "projectId", "Filter by project...")}
      {textFilter("Region", "regionId", "Filter by region...")}

      <Button variant="normal" onClick={onClearFilters}>
        Clear filters
      </Button>
    </div>
  );
}

interface PaginationControlsProps {
  page: number;
  pageSize: number;
  totalPages: number;
  hasMore: boolean;
  total: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
}

function PaginationControls({
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
