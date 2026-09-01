import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  type TableColumnDefinition,
} from "@araf/ui";
import type { ProviderHealth } from "@araf/api-client";
import { useProviderHealth } from "../hooks/useProviderHealth";
import { errorMessage, errorCorrelationId } from "../errors";

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

function statusLabel(status: string): string {
  return status.charAt(0).toUpperCase() + status.slice(1);
}

export function ProviderHealthPage() {
  const { providers, loading, error, refresh } = useProviderHealth();

  const columns: TableColumnDefinition<ProviderHealth>[] = [
    {
      id: "name",
      header: "Name",
      cell: (provider) => provider.name,
      isRowHeader: true,
    },
    { id: "kind", header: "Kind", cell: (provider) => provider.kind },
    {
      id: "status",
      header: "Status",
      cell: (provider) => statusLabel(provider.status),
    },
    { id: "region", header: "Region", cell: (provider) => provider.regionId },
    {
      id: "lastSeen",
      header: "Last seen",
      cell: (provider) => formatTimestamp(provider.lastSeenAt),
    },
    { id: "message", header: "Message", cell: (provider) => provider.message },
  ];

  return (
    <section aria-label="Provider health">
      <Header variant="h1" headingLevel="h1">
        Provider health
      </Header>

      {error ? (
        <ErrorState
          title="Could not load provider health"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {loading && !providers ? <LoadingState message="Loading provider health..." /> : null}

      {!error && (
        <Table<ProviderHealth>
          items={providers ?? []}
          columnDefinitions={columns}
          trackingId="id"
          loading={loading}
          loadingText="Loading provider health..."
          empty={
            <EmptyState
              title="No provider health data"
              description="No provider health data available."
            />
          }
          ariaLabels={{ tableLabel: "Provider health table" }}
        />
      )}
    </section>
  );
}
