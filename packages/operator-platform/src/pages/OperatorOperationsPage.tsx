import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  Button,
  type TableColumnDefinition,
} from "@araf/ui";
import { Link, useSearchParams } from "react-router";
import type { Operation, OperationState } from "@araf/api-client";
import { useCapabilities } from "@araf/resources";
import { useOperatorOperations } from "../hooks/useOperatorOperations";
import { PaginationControls } from "../components/PaginationControls";
import { hasCapability } from "../capabilities";
import { errorMessage, errorCorrelationId } from "../errors";
import { clampPageSize, DEFAULT_PAGE, DEFAULT_PAGE_SIZE, parseIntOr } from "../pagination";

const PAGE_PARAM = "page";
const PAGE_SIZE_PARAM = "pageSize";
const STATE_PARAM = "state";

function formatTimestamp(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

const STATE_OPTIONS: { value: OperationState | ""; label: string }[] = [
  { value: "", label: "All states" },
  { value: "pending", label: "Pending" },
  { value: "running", label: "Running" },
  { value: "succeeded", label: "Succeeded" },
  { value: "failed", label: "Failed" },
];

export function OperatorOperationsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();

  const page = Math.max(parseIntOr(searchParams.get(PAGE_PARAM), DEFAULT_PAGE), 1);
  const pageSize = clampPageSize(parseIntOr(searchParams.get(PAGE_SIZE_PARAM), DEFAULT_PAGE_SIZE));
  const state = (searchParams.get(STATE_PARAM) as OperationState | null) ?? undefined;

  const { collection, loading, error, refresh } = useOperatorOperations({
    page: page - 1,
    pageSize,
    state,
  });
  const canList = hasCapability(capabilities, "operator.operation", "list");

  const setPage = (nextPage: number) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      if (nextPage === DEFAULT_PAGE) {
        next.delete(PAGE_PARAM);
      } else {
        next.set(PAGE_PARAM, String(nextPage));
      }
      return next;
    });
  };

  const setPageSize = (nextPageSize: number) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.delete(PAGE_PARAM);
      next.set(PAGE_SIZE_PARAM, String(clampPageSize(nextPageSize)));
      return next;
    });
  };

  const setState = (value: OperationState | undefined) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.delete(PAGE_PARAM);
      if (value) {
        next.set(STATE_PARAM, value);
      } else {
        next.delete(STATE_PARAM);
      }
      return next;
    });
  };

  const columns: TableColumnDefinition<Operation>[] = [
    {
      id: "id",
      header: "ID",
      cell: (op) => <Link to={`/operations/${encodeURIComponent(op.id)}`}>{op.id}</Link>,
      isRowHeader: true,
    },
    { id: "action", header: "Action", cell: (op) => op.action },
    { id: "state", header: "State", cell: (op) => op.state },
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
    <section aria-label="Operator operations">
      <Header variant="h1" headingLevel="h1">
        Operations
      </Header>

      {capabilitiesLoading || loading ? <LoadingState message="Loading operations..." /> : null}

      {!canList && !capabilitiesLoading ? (
        <ErrorState
          title="Access denied"
          message="You do not have permission to list operations."
        />
      ) : null}

      {error ? (
        <ErrorState
          title="Could not load operations"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {canList && !error && (
        <>
          <div style={{ display: "flex", gap: "1rem", margin: "1rem 0", flexWrap: "wrap" }}>
            <label style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
              State
              <select
                aria-label="State"
                value={state ?? ""}
                onChange={(e) => {
                  const value = e.target.value;
                  setState(value === "" ? undefined : (value as OperationState));
                }}
              >
                {STATE_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>

            <Button
              variant="normal"
              onClick={() => {
                setState(undefined);
              }}
            >
              Clear filters
            </Button>
          </div>

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
            ariaLabels={{ tableLabel: "Operator operations table" }}
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
