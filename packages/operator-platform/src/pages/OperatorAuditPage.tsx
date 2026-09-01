import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  Button,
  type TableColumnDefinition,
} from "@araf/ui";
import { useSearchParams } from "react-router";
import type { OperatorAuditEvent } from "@araf/api-client";
import { useCapabilities } from "@araf/resources";
import { useOperatorAuditEvents } from "../hooks/useOperatorAuditEvents";
import { PaginationControls } from "../components/PaginationControls";
import { hasCapability } from "../capabilities";
import { errorMessage, errorCorrelationId } from "../errors";
import { clampPageSize, DEFAULT_PAGE, DEFAULT_PAGE_SIZE, parseIntOr } from "../pagination";

const PAGE_PARAM = "page";
const PAGE_SIZE_PARAM = "pageSize";
const ACTION_PARAM = "action";
const ACTOR_PARAM = "actor";
const ACCOUNT_PARAM = "accountId";

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function OperatorAuditPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();

  const page = Math.max(parseIntOr(searchParams.get(PAGE_PARAM), DEFAULT_PAGE), 1);
  const pageSize = clampPageSize(parseIntOr(searchParams.get(PAGE_SIZE_PARAM), DEFAULT_PAGE_SIZE));
  const action = searchParams.get(ACTION_PARAM) ?? undefined;
  const actor = searchParams.get(ACTOR_PARAM) ?? undefined;
  const accountId = searchParams.get(ACCOUNT_PARAM) ?? undefined;

  const { collection, loading, error } = useOperatorAuditEvents({
    page: page - 1,
    pageSize,
    action,
    actor,
    accountId,
  });
  const canRead = hasCapability(capabilities, "operator.audit", "read");

  const setParam = (key: string, value: string | undefined) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.delete(PAGE_PARAM);
      if (value) {
        next.set(key, value);
      } else {
        next.delete(key);
      }
      return next;
    });
  };

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

  const clearFilters = () => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.delete(ACTION_PARAM);
      next.delete(ACTOR_PARAM);
      next.delete(ACCOUNT_PARAM);
      next.delete(PAGE_PARAM);
      return next;
    });
  };

  const columns: TableColumnDefinition<OperatorAuditEvent>[] = [
    {
      id: "recorded",
      header: "Recorded",
      cell: (event) => formatTimestamp(event.recordedAt),
      isRowHeader: true,
    },
    { id: "actor", header: "Actor", cell: (event) => event.actor },
    { id: "action", header: "Action", cell: (event) => event.action },
    {
      id: "resource",
      header: "Resource",
      cell: (event) =>
        event.resourceType && event.resourceId ? `${event.resourceType}/${event.resourceId}` : "—",
    },
    { id: "account", header: "Account", cell: (event) => event.accountId ?? "—" },
    { id: "project", header: "Project", cell: (event) => event.projectId ?? "—" },
    { id: "outcome", header: "Outcome", cell: (event) => event.outcome },
    { id: "correlation", header: "Correlation ID", cell: (event) => event.correlationId },
  ];

  const totalPages = collection ? Math.ceil(collection.total / collection.pageSize) : 0;

  return (
    <section aria-label="Operator audit events">
      <Header variant="h1" headingLevel="h1">
        Audit
      </Header>

      {capabilitiesLoading || loading ? <LoadingState message="Loading audit events..." /> : null}

      {!canRead && !capabilitiesLoading ? (
        <ErrorState
          title="Access denied"
          message="You do not have permission to view audit events."
        />
      ) : null}

      {error ? (
        <ErrorState
          title="Could not load audit events"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {canRead && !error && (
        <>
          <div style={{ display: "flex", gap: "1rem", margin: "1rem 0", flexWrap: "wrap" }}>
            <FilterInput
              label="Action"
              value={action ?? ""}
              onChange={(value) => {
                setParam(ACTION_PARAM, value || undefined);
              }}
              placeholder="Filter by action..."
            />
            <FilterInput
              label="Actor"
              value={actor ?? ""}
              onChange={(value) => {
                setParam(ACTOR_PARAM, value || undefined);
              }}
              placeholder="Filter by actor..."
            />
            <FilterInput
              label="Account ID"
              value={accountId ?? ""}
              onChange={(value) => {
                setParam(ACCOUNT_PARAM, value || undefined);
              }}
              placeholder="Filter by account..."
            />
            <Button variant="normal" onClick={clearFilters}>
              Clear filters
            </Button>
          </div>

          <Table<OperatorAuditEvent>
            items={collection?.items ?? []}
            columnDefinitions={columns}
            trackingId="id"
            loading={loading}
            loadingText="Loading audit events..."
            empty={
              <EmptyState
                title="No audit events"
                description="There are no audit events matching the current filters."
              />
            }
            ariaLabels={{ tableLabel: "Operator audit events table" }}
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

function FilterInput({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}) {
  return (
    <label style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
      {label}
      <input
        type="search"
        aria-label={label}
        value={value}
        placeholder={placeholder}
        onChange={(e) => {
          onChange(e.target.value);
        }}
      />
    </label>
  );
}
