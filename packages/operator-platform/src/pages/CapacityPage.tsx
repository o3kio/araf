import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  type TableColumnDefinition,
} from "@araf/ui";
import type { CapacitySummary } from "@araf/api-client";
import { useCapacity } from "../hooks/useCapacity";
import { errorMessage, errorCorrelationId } from "../errors";

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function CapacityPage() {
  const { summary, loading, error, refresh } = useCapacity();

  const columns: TableColumnDefinition<CapacitySummary>[] = [
    {
      id: "resourceClass",
      header: "Resource class",
      cell: (entry) => entry.resourceClass,
      isRowHeader: true,
    },
    { id: "total", header: "Total", cell: (entry) => String(entry.total) },
    { id: "used", header: "Used", cell: (entry) => String(entry.used) },
    {
      id: "available",
      header: "Available",
      cell: (entry) => String(entry.available),
    },
    { id: "unit", header: "Unit", cell: (entry) => entry.unit },
    {
      id: "updated",
      header: "Updated",
      cell: (entry) => formatTimestamp(entry.updatedAt),
    },
  ];

  return (
    <section aria-label="Capacity">
      <Header variant="h1" headingLevel="h1">
        Capacity
      </Header>

      {error ? (
        <ErrorState
          title="Could not load capacity summary"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {loading && !summary ? <LoadingState message="Loading capacity summary..." /> : null}

      {!error && (
        <Table<CapacitySummary>
          items={summary ?? []}
          columnDefinitions={columns}
          trackingId="resourceClass"
          loading={loading}
          loadingText="Loading capacity summary..."
          empty={
            <EmptyState
              title="No capacity data"
              description="No capacity summary data available."
            />
          }
          ariaLabels={{ tableLabel: "Capacity table" }}
        />
      )}
    </section>
  );
}
