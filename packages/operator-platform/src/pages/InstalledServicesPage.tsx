import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  SpaceBetween,
  type TableColumnDefinition,
} from "@araf/ui";
import type { InstalledService, DiscoveredResourceType } from "@araf/api-client";
import { useInstalledServices } from "../hooks/useInstalledServices";
import { useDiscoveredResourceTypes } from "../hooks/useDiscoveredResourceTypes";
import { errorMessage, errorCorrelationId } from "../errors";

function formatTimestamp(iso: string | undefined): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

function InstalledServicesTable({
  services,
  loading,
}: {
  services: InstalledService[] | undefined;
  loading: boolean;
}) {
  const columns: TableColumnDefinition<InstalledService>[] = [
    {
      id: "name",
      header: "Name",
      cell: (service) => service.name,
      isRowHeader: true,
    },
    { id: "id", header: "ID", cell: (service) => service.id },
    { id: "version", header: "Version", cell: (service) => service.version },
    { id: "health", header: "Health", cell: (service) => service.health },
    { id: "lifecycle", header: "Lifecycle", cell: (service) => service.lifecycleState },
    {
      id: "resourceTypes",
      header: "Resource types",
      cell: (service) => service.resourceTypes.join(", ") || "—",
    },
    {
      id: "installed",
      header: "Installed",
      cell: (service) => formatTimestamp(service.installedAt),
    },
  ];

  return (
    <Table<InstalledService>
      items={services ?? []}
      columnDefinitions={columns}
      trackingId="id"
      loading={loading}
      loadingText="Loading installed services..."
      empty={
        <EmptyState
          title="No installed services"
          description="There are no installed services visible to you."
        />
      }
      ariaLabels={{ tableLabel: "Installed services" }}
    />
  );
}

function DiscoveredResourceTypesTable({
  resourceTypes,
  loading,
}: {
  resourceTypes: DiscoveredResourceType[] | undefined;
  loading: boolean;
}) {
  const columns: TableColumnDefinition<DiscoveredResourceType>[] = [
    {
      id: "name",
      header: "Name",
      cell: (rt) => rt.name,
      isRowHeader: true,
    },
    { id: "namespace", header: "Namespace", cell: (rt) => rt.namespace },
    { id: "serviceId", header: "Service ID", cell: (rt) => rt.serviceId },
    { id: "collection", header: "Collection", cell: (rt) => rt.collection },
    { id: "scope", header: "Scope", cell: (rt) => rt.scope },
    { id: "ready", header: "Ready", cell: (rt) => (rt.ready ? "Yes" : "No") },
  ];

  return (
    <Table<DiscoveredResourceType>
      items={resourceTypes ?? []}
      columnDefinitions={columns}
      trackingId="name"
      loading={loading}
      loadingText="Loading discovered resource types..."
      empty={
        <EmptyState
          title="No resource types"
          description="There are no discovered resource types visible to you."
        />
      }
      ariaLabels={{ tableLabel: "Discovered resource types" }}
    />
  );
}

export function InstalledServicesPage() {
  const {
    services,
    loading: servicesLoading,
    error: servicesError,
    refresh: refreshServices,
  } = useInstalledServices();
  const {
    resourceTypes,
    loading: typesLoading,
    error: typesError,
    refresh: refreshTypes,
  } = useDiscoveredResourceTypes();

  const error = servicesError ?? typesError;
  const loading = servicesLoading || typesLoading;

  return (
    <section aria-label="Installed services">
      <Header variant="h1" headingLevel="h1">
        Installed services
      </Header>

      {error ? (
        <ErrorState
          title="Could not load services"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={() => {
            refreshServices();
            refreshTypes();
          }}
        />
      ) : null}

      {loading && !services && !resourceTypes ? (
        <LoadingState message="Loading installed services..." />
      ) : null}

      {!error && (
        <SpaceBetween direction="vertical" size="l">
          <section aria-label="Installed services list">
            <Header variant="h2" headingLevel="h2">
              Services
            </Header>
            <InstalledServicesTable services={services} loading={servicesLoading} />
          </section>

          <section aria-label="Discovered resource types list">
            <Header variant="h2" headingLevel="h2">
              Discovered resource types
            </Header>
            <DiscoveredResourceTypesTable resourceTypes={resourceTypes} loading={typesLoading} />
          </section>
        </SpaceBetween>
      )}
    </section>
  );
}
