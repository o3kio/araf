import {
  Header,
  LoadingState,
  EmptyState,
  ErrorState,
  Table,
  type TableColumnDefinition,
} from "@araf/ui";
import type { PlatformOverview, PlatformAlert, StatusCount } from "@araf/api-client";
import { usePlatformOverview } from "../hooks/usePlatformOverview";
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

export function PlatformOverviewPage() {
  const { overview, loading, error, refresh } = usePlatformOverview();

  return (
    <section aria-label="Platform overview">
      <Header variant="h1" headingLevel="h1">
        Platform overview
      </Header>

      {error ? (
        <ErrorState
          title="Could not load platform overview"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {loading && !overview ? <LoadingState message="Loading platform overview..." /> : null}

      {overview ? <OverviewContent overview={overview} /> : null}
    </section>
  );
}

function OverviewContent({ overview }: { overview: PlatformOverview }) {
  const statusColumns: TableColumnDefinition<StatusCount>[] = [
    {
      id: "status",
      header: "Status",
      cell: (s) => statusLabel(s.status),
      isRowHeader: true,
    },
    { id: "count", header: "Count", cell: (s) => String(s.count) },
  ];

  const alertColumns: TableColumnDefinition<PlatformAlert>[] = [
    {
      id: "severity",
      header: "Severity",
      cell: (a) => a.severity,
      isRowHeader: true,
    },
    { id: "message", header: "Message", cell: (a) => a.message },
    {
      id: "occurred",
      header: "Occurred",
      cell: (a) => formatTimestamp(a.occurredAt),
    },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}>
      <p>
        <strong>Active operations:</strong> {overview.activeOperationsCount}
      </p>

      <section aria-label="Region status summary">
        <Header variant="h2" headingLevel="h2">
          Region status
        </Header>
        <Table<StatusCount>
          items={overview.regionStatusSummary}
          columnDefinitions={statusColumns}
          trackingId="status"
          empty={<EmptyState title="No region status data" />}
          ariaLabels={{ tableLabel: "Region status summary" }}
        />
      </section>

      <section aria-label="Provider status summary">
        <Header variant="h2" headingLevel="h2">
          Provider status
        </Header>
        <Table<StatusCount>
          items={overview.providerStatusSummary}
          columnDefinitions={statusColumns}
          trackingId="status"
          empty={<EmptyState title="No provider status data" />}
          ariaLabels={{ tableLabel: "Provider status summary" }}
        />
      </section>

      <section aria-label="Recent alerts">
        <Header variant="h2" headingLevel="h2">
          Recent alerts
        </Header>
        <Table<PlatformAlert>
          items={overview.recentAlerts}
          columnDefinitions={alertColumns}
          trackingId="id"
          empty={<EmptyState title="No recent alerts" />}
          ariaLabels={{ tableLabel: "Recent alerts" }}
        />
      </section>

      <p>
        <small>Data refreshed at {formatTimestamp(overview.dataFreshnessAt)}</small>
      </p>
    </div>
  );
}
