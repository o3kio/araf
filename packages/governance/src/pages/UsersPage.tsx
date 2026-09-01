import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  type TableColumnDefinition,
} from "@araf/ui";
import { Link, useSearchParams } from "react-router";
import { useCapabilities } from "@araf/resources";
import type { User } from "@araf/api-client";
import { useUsers } from "../hooks/useUsers";
import { hasCapability } from "../capabilities";
import { PaginationControls } from "../components/PaginationControls";
import { errorMessage, errorCorrelationId } from "../errors";
import { clampPageSize, DEFAULT_PAGE, DEFAULT_PAGE_SIZE, parseIntOr } from "../pagination";

const PAGE_PARAM = "page";
const PAGE_SIZE_PARAM = "pageSize";

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function UsersPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();

  const page = Math.max(parseIntOr(searchParams.get(PAGE_PARAM), DEFAULT_PAGE), 1);
  const pageSize = clampPageSize(parseIntOr(searchParams.get(PAGE_SIZE_PARAM), DEFAULT_PAGE_SIZE));

  const { collection, loading, error } = useUsers({ page: page - 1, pageSize });
  const canList = hasCapability(capabilities, "tenant.user", "list");

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

  const columns: TableColumnDefinition<User>[] = [
    {
      id: "name",
      header: "Name",
      cell: (user) => (
        <Link to={`/organization/users/${encodeURIComponent(user.id)}`}>{user.name}</Link>
      ),
      isRowHeader: true,
    },
    { id: "id", header: "ID", cell: (user) => user.id },
    { id: "email", header: "Email", cell: (user) => user.email ?? "—" },
    { id: "status", header: "Status", cell: (user) => user.status },
    {
      id: "created",
      header: "Created",
      cell: (user) => formatTimestamp(user.createdAt),
    },
  ];

  const totalPages = collection ? Math.ceil(collection.total / collection.pageSize) : 0;

  return (
    <section aria-label="Users">
      <Header variant="h1" headingLevel="h1">
        Users & Access
      </Header>

      {capabilitiesLoading || loading ? <LoadingState message="Loading users..." /> : null}

      {!canList && !capabilitiesLoading ? (
        <ErrorState title="Access denied" message="You do not have permission to list users." />
      ) : null}

      {error ? (
        <ErrorState
          title="Could not load users"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {canList && !error && (
        <>
          <Table<User>
            items={collection?.items ?? []}
            columnDefinitions={columns}
            trackingId="id"
            loading={loading}
            loadingText="Loading users..."
            empty={
              <EmptyState
                title="No users"
                description="There are no users matching the current scope."
              />
            }
            ariaLabels={{ tableLabel: "Users table" }}
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
