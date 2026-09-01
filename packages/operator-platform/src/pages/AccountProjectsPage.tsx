import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  BreadcrumbGroup,
  type TableColumnDefinition,
} from "@araf/ui";
import { useParams } from "react-router";
import type { OperatorProject } from "@araf/api-client";
import { useAccountProjects } from "../hooks/useAccountProjects";
import { errorMessage, errorCorrelationId } from "../errors";

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function AccountProjectsPage() {
  const { id } = useParams<{ id: string }>();
  const accountId = decodeURIComponent(id ?? "");
  const { collection, loading, error, refresh } = useAccountProjects(accountId);

  const columns: TableColumnDefinition<OperatorProject>[] = [
    {
      id: "name",
      header: "Name",
      cell: (project) => project.name,
      isRowHeader: true,
    },
    { id: "id", header: "ID", cell: (project) => project.id },
    { id: "region", header: "Region", cell: (project) => project.regionId },
    { id: "status", header: "Status", cell: (project) => project.status },
    {
      id: "created",
      header: "Created",
      cell: (project) => formatTimestamp(project.createdAt),
    },
  ];

  return (
    <section aria-label="Account projects">
      <BreadcrumbGroup
        items={[
          { text: "Accounts", href: "/customers/accounts" },
          { text: accountId, href: `#` },
        ]}
      />

      <Header variant="h1" headingLevel="h1">
        Projects for {accountId}
      </Header>

      {error ? (
        <ErrorState
          title="Could not load account projects"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {loading && !collection ? <LoadingState message="Loading account projects..." /> : null}

      {!error && (
        <Table<OperatorProject>
          items={collection?.items ?? []}
          columnDefinitions={columns}
          trackingId="id"
          loading={loading}
          loadingText="Loading account projects..."
          empty={
            <EmptyState title="No projects" description="There are no projects for this account." />
          }
          ariaLabels={{ tableLabel: "Account projects table" }}
        />
      )}
    </section>
  );
}
