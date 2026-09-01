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
import type { Project } from "@araf/api-client";
import { useProjects } from "../hooks/useProjects";
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

export function ProjectsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();

  const page = Math.max(parseIntOr(searchParams.get(PAGE_PARAM), DEFAULT_PAGE), 1);
  const pageSize = clampPageSize(parseIntOr(searchParams.get(PAGE_SIZE_PARAM), DEFAULT_PAGE_SIZE));

  const { collection, loading, error } = useProjects({ page: page - 1, pageSize });

  const canList = hasCapability(capabilities, "tenant.project", "list");

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

  const columns: TableColumnDefinition<Project>[] = [
    {
      id: "name",
      header: "Name",
      cell: (project) => (
        <Link to={`/organization/projects/${encodeURIComponent(project.id)}`}>{project.name}</Link>
      ),
      isRowHeader: true,
    },
    { id: "id", header: "ID", cell: (project) => project.id },
    { id: "organization", header: "Organization", cell: (project) => project.organizationId },
    { id: "status", header: "Status", cell: (project) => project.status },
    {
      id: "created",
      header: "Created",
      cell: (project) => formatTimestamp(project.createdAt),
    },
  ];

  const totalPages = collection ? Math.ceil(collection.total / collection.pageSize) : 0;

  return (
    <section aria-label="Projects">
      <Header variant="h1" headingLevel="h1">
        Projects
      </Header>

      {capabilitiesLoading || loading ? <LoadingState message="Loading projects..." /> : null}

      {!canList && !capabilitiesLoading ? (
        <ErrorState title="Access denied" message="You do not have permission to list projects." />
      ) : null}

      {error ? (
        <ErrorState
          title="Could not load projects"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {canList && !error && (
        <>
          <Table<Project>
            items={collection?.items ?? []}
            columnDefinitions={columns}
            trackingId="id"
            loading={loading}
            loadingText="Loading projects..."
            empty={
              <EmptyState
                title="No projects"
                description="There are no projects matching the current scope."
              />
            }
            ariaLabels={{ tableLabel: "Projects table" }}
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
