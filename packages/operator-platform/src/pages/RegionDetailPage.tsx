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
import type { AvailabilityZone } from "@araf/api-client";
import { useAvailabilityZones } from "../hooks/useAvailabilityZones";
import { errorMessage, errorCorrelationId } from "../errors";

function statusLabel(status: string): string {
  return status.charAt(0).toUpperCase() + status.slice(1);
}

export function RegionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const regionId = decodeURIComponent(id ?? "");
  const { zones, loading, error, refresh } = useAvailabilityZones(regionId);

  const columns: TableColumnDefinition<AvailabilityZone>[] = [
    {
      id: "name",
      header: "Name",
      cell: (zone) => zone.name,
      isRowHeader: true,
    },
    { id: "id", header: "ID", cell: (zone) => zone.id },
    {
      id: "status",
      header: "Status",
      cell: (zone) => statusLabel(zone.status),
    },
  ];

  return (
    <section aria-label="Region details">
      <BreadcrumbGroup
        items={[
          { text: "Regions", href: "/platform/regions" },
          { text: regionId, href: `#` },
        ]}
      />

      <Header variant="h1" headingLevel="h1">
        {regionId} availability zones
      </Header>

      {error ? (
        <ErrorState
          title="Could not load availability zones"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {loading && !zones ? <LoadingState message="Loading availability zones..." /> : null}

      {!error && (
        <Table<AvailabilityZone>
          items={zones ?? []}
          columnDefinitions={columns}
          trackingId="id"
          loading={loading}
          loadingText="Loading availability zones..."
          empty={
            <EmptyState
              title="No availability zones"
              description="No availability zone data for this region."
            />
          }
          ariaLabels={{ tableLabel: "Availability zones table" }}
        />
      )}
    </section>
  );
}
