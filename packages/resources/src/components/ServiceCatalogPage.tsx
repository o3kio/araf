import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  type TableColumnDefinition,
} from "@araf/ui";
import type { ServiceCatalogEntry } from "@araf/api-client";
import { useServiceCatalog } from "../hooks/useServiceCatalog";
import { errorMessage, errorCorrelationId } from "../errors";

function formatRegions(regions: string[]): string {
  if (regions.length === 0) return "Global";
  return regions.join(", ");
}

function formatCapabilities(capabilities: ServiceCatalogEntry["capabilities"]): string {
  return capabilities.map((c) => `${c.resourceType}:${c.action}`).join(", ");
}

export function ServiceCatalogPage() {
  const { entries, loading, error, refresh } = useServiceCatalog();

  const columns: TableColumnDefinition<ServiceCatalogEntry>[] = [
    {
      id: "name",
      header: "Name",
      cell: (entry) => entry.name,
      isRowHeader: true,
    },
    { id: "id", header: "ID", cell: (entry) => entry.id },
    { id: "version", header: "Version", cell: (entry) => entry.version },
    { id: "lifecycle", header: "Lifecycle", cell: (entry) => entry.lifecycleState },
    {
      id: "regions",
      header: "Regions",
      cell: (entry) => formatRegions(entry.regions),
    },
    {
      id: "capabilities",
      header: "Capabilities",
      cell: (entry) => formatCapabilities(entry.capabilities),
    },
  ];

  return (
    <section aria-label="Service catalog">
      <Header variant="h1" headingLevel="h1">
        Service catalog
      </Header>

      {error ? (
        <ErrorState
          title="Could not load service catalog"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {loading && !entries ? <LoadingState message="Loading service catalog..." /> : null}

      {!error && (
        <Table<ServiceCatalogEntry>
          items={entries ?? []}
          columnDefinitions={columns}
          trackingId="id"
          loading={loading}
          loadingText="Loading service catalog..."
          empty={
            <EmptyState
              title="No services"
              description="There are no services available in the catalog."
            />
          }
          ariaLabels={{ tableLabel: "Service catalog" }}
        />
      )}
    </section>
  );
}
