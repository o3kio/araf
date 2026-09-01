import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  type TableColumnDefinition,
} from "@araf/ui";
import { useSearchParams } from "react-router";
import { useCapabilities } from "@araf/resources";
import type { ProjectQuota, QuotaEntry } from "@araf/api-client";
import { useQuotas } from "../hooks/useQuotas";
import { hasCapability } from "../capabilities";
import { PaginationControls } from "../components/PaginationControls";
import { errorMessage, errorCorrelationId } from "../errors";
import { clampPageSize, DEFAULT_PAGE, DEFAULT_PAGE_SIZE, parseIntOr } from "../pagination";

const PAGE_PARAM = "page";
const PAGE_SIZE_PARAM = "pageSize";
const PROJECT_PARAM = "projectId";

export function QuotasPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();

  const page = Math.max(parseIntOr(searchParams.get(PAGE_PARAM), DEFAULT_PAGE), 1);
  const pageSize = clampPageSize(parseIntOr(searchParams.get(PAGE_SIZE_PARAM), DEFAULT_PAGE_SIZE));
  const projectId = searchParams.get(PROJECT_PARAM) ?? undefined;

  const { collection, loading, error } = useQuotas({ projectId, page: page - 1, pageSize });
  const canRead = hasCapability(capabilities, "tenant.quota", "read");

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

  const setProjectId = (value: string) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.delete(PAGE_PARAM);
      if (value) {
        next.set(PROJECT_PARAM, value);
      } else {
        next.delete(PROJECT_PARAM);
      }
      return next;
    });
  };

  const columns: TableColumnDefinition<ProjectQuota>[] = [
    { id: "projectId", header: "Project", cell: (quota) => quota.projectId },
    {
      id: "entries",
      header: "Usage",
      cell: (quota) => <QuotaEntries entries={quota.entries} />,
    },
  ];

  const totalPages = collection ? Math.ceil(collection.total / collection.pageSize) : 0;

  return (
    <section aria-label="Quotas">
      <Header variant="h1" headingLevel="h1">
        Quotas
      </Header>

      {capabilitiesLoading || loading ? <LoadingState message="Loading quotas..." /> : null}

      {!canRead && !capabilitiesLoading ? (
        <ErrorState title="Access denied" message="You do not have permission to view quotas." />
      ) : null}

      {error ? (
        <ErrorState
          title="Could not load quotas"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {canRead && !error && (
        <>
          <div
            className="araf-quotas-filters"
            style={{ display: "flex", gap: "1rem", margin: "1rem 0", flexWrap: "wrap" }}
          >
            <label>
              Project ID:{" "}
              <input
                type="search"
                aria-label="Project ID"
                value={projectId ?? ""}
                placeholder="Filter by project..."
                onChange={(e) => {
                  setProjectId(e.target.value);
                }}
              />
            </label>
          </div>

          <Table<ProjectQuota>
            items={collection?.items ?? []}
            columnDefinitions={columns}
            trackingId="projectId"
            loading={loading}
            loadingText="Loading quotas..."
            empty={
              <EmptyState
                title="No quotas"
                description="There are no quotas matching the current filters."
              />
            }
            ariaLabels={{ tableLabel: "Quotas table" }}
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

function QuotaEntries({ entries }: { entries: QuotaEntry[] }) {
  return (
    <ul style={{ margin: 0, paddingInlineStart: "1rem" }}>
      {entries.map((entry) => {
        const percentage = entry.limit > 0 ? Math.min((entry.used / entry.limit) * 100, 100) : 0;
        return (
          <li key={entry.resourceType} style={{ marginBottom: "0.5rem" }}>
            <div>
              {entry.resourceType}: {entry.used} / {entry.limit} {entry.unit}
            </div>
            <div
              role="img"
              aria-label={`${entry.resourceType} usage ${percentage.toFixed(0)} percent`}
              style={{
                width: "100%",
                maxWidth: "200px",
                height: "0.5rem",
                backgroundColor: "var(--color-bg-layout, #e9ebed)",
                borderRadius: "0.25rem",
                overflow: "hidden",
              }}
            >
              <div
                style={{
                  width: `${String(percentage)}%`,
                  height: "100%",
                  backgroundColor: "var(--color-text-accent, #0972d3)",
                }}
              />
            </div>
          </li>
        );
      })}
    </ul>
  );
}
