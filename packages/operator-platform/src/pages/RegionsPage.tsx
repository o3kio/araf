import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  type TableColumnDefinition,
} from "@araf/ui";
import { Link } from "react-router";
import type { Region } from "@araf/api-client";
import { useRegions } from "../hooks/useRegions";
import { errorMessage, errorCorrelationId } from "../errors";

function statusLabel(status: string): string {
  return status.charAt(0).toUpperCase() + status.slice(1);
}

export function RegionsPage() {
  const { regions, loading, error, refresh } = useRegions();

  const columns: TableColumnDefinition<Region>[] = [
    {
      id: "name",
      header: "Name",
      cell: (region) => (
        <Link to={`/platform/regions/${encodeURIComponent(region.id)}`}>{region.name}</Link>
      ),
      isRowHeader: true,
    },
    { id: "id", header: "ID", cell: (region) => region.id },
    {
      id: "status",
      header: "Status",
      cell: (region) => statusLabel(region.status),
    },
    {
      id: "zones",
      header: "Availability zones",
      cell: (region) => String(region.azs.length),
    },
  ];

  return (
    <section aria-label="Regions">
      <Header variant="h1" headingLevel="h1">
        Regions
      </Header>

      {error ? (
        <ErrorState
          title="Could not load regions"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {loading && !regions ? <LoadingState message="Loading regions..." /> : null}

      {!error && (
        <Table<Region>
          items={regions ?? []}
          columnDefinitions={columns}
          trackingId="id"
          loading={loading}
          loadingText="Loading regions..."
          empty={<EmptyState title="No regions" description="No region data available." />}
          ariaLabels={{ tableLabel: "Regions table" }}
        />
      )}
    </section>
  );
}
