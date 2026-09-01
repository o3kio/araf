import { useMemo, useState } from "react";
import { EmptyState, ErrorState, LoadingState, Table, type TableColumnDefinition } from "@araf/ui";
import { useUsage, type UseUsageResult } from "../hooks/useUsage";
import { useQuotas } from "../hooks/useQuotas";

function dateInputValue(date: Date): string {
  return date.toISOString().slice(0, 16);
}

const NOW = new Date();
const SEVEN_DAYS_AGO = new Date(NOW.getTime() - 7 * 24 * 60 * 60 * 1000);

interface UsageContentProps {
  usage: UseUsageResult;
  quotas: ReturnType<typeof useQuotas>;
}

function UsageContent({ usage, quotas }: UsageContentProps) {
  if (usage.loading) {
    return <LoadingState message="Loading usage data" />;
  }

  if (usage.error) {
    return (
      <ErrorState
        title="Failed to load usage data"
        message={usage.error.message}
        onRetry={usage.refresh}
      />
    );
  }

  const summary = usage.summary;
  if (!summary || summary.records.length === 0) {
    return (
      <EmptyState
        title="No usage data"
        description="No usage records are available for the selected period."
      />
    );
  }

  // Build a map of resource type -> merged entry with quota info.
  const quotaMap = useMemo(() => {
    const map = new Map<string, { limit: number; unit: string }>();
    for (const project of quotas.collection?.items ?? []) {
      for (const entry of project.entries) {
        map.set(entry.resourceType, { limit: entry.limit, unit: entry.unit });
      }
    }
    return map;
  }, [quotas.collection]);

  // Latest value per resource type from the records.
  const latestByType = useMemo(() => {
    const map = new Map<string, { value: number; unit: string; timestamp: string }>();
    for (const record of summary.records) {
      map.set(record.resourceType, {
        value: record.value,
        unit: record.unit,
        timestamp: record.timestamp,
      });
    }
    return map;
  }, [summary.records]);

  interface UsageRow {
    resourceType: string;
    value: number;
    unit: string;
    timestamp: string;
    limit: number | undefined;
  }

  const columnDefinitions: TableColumnDefinition<UsageRow>[] = [
    { id: "resourceType", header: "Resource type", cell: (r) => r.resourceType },
    { id: "value", header: "Current usage", cell: (r) => `${r.value.toLocaleString()} ${r.unit}` },
    {
      id: "limit",
      header: "Limit",
      cell: (r) => (r.limit !== undefined ? `${r.limit.toLocaleString()} ${r.unit}` : "\u2014"),
    },
    {
      id: "usage",
      header: "Usage",
      cell: (r) => {
        if (r.limit === undefined || r.limit === 0) return "\u2014";
        const pct = Math.round((r.value / r.limit) * 100);
        return `${String(pct)}%`;
      },
    },
    { id: "updated", header: "Last updated", cell: (r) => new Date(r.timestamp).toLocaleString() },
  ];

  const items: UsageRow[] = Array.from(latestByType.entries()).map(([resourceType, rec]) => {
    const quota = quotaMap.get(resourceType);
    return {
      resourceType,
      value: rec.value,
      unit: rec.unit,
      timestamp: rec.timestamp,
      limit: quota?.limit,
    };
  });

  return (
    <section aria-label="Usage summary">
      <Table<UsageRow>
        columnDefinitions={columnDefinitions}
        items={items}
        ariaLabels={{ tableLabel: "Usage by resource type" }}
      />
    </section>
  );
}

export function UsagePage() {
  const [since, setSince] = useState(dateInputValue(SEVEN_DAYS_AGO));
  const [until, setUntil] = useState(dateInputValue(NOW));

  const usage = useUsage({ since, until });
  const quotas = useQuotas();

  return (
    <section>
      <h1>Usage &amp; Cost</h1>
      <p>
        This page shows resource consumption across your project. Cost estimates are shown only when
        authoritative pricing data is available from the upstream O3K service.
      </p>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          // Trigger refresh by toggling a query param change.
          usage.refresh();
        }}
        style={{ display: "flex", gap: "1rem", alignItems: "end", marginBlock: "1rem" }}
      >
        <label>
          From
          <input
            type="datetime-local"
            value={since}
            onChange={(e) => {
              setSince(e.target.value);
            }}
            aria-label="Start date and time"
          />
        </label>
        <label>
          To
          <input
            type="datetime-local"
            value={until}
            onChange={(e) => {
              setUntil(e.target.value);
            }}
            aria-label="End date and time"
          />
        </label>
        <button type="submit" aria-label="Refresh usage data">
          Refresh
        </button>
      </form>

      <UsageContent usage={usage} quotas={quotas} />

      <section style={{ marginTop: "2rem" }}>
        <h2>Quota overview</h2>
        {quotas.loading ? (
          <LoadingState message="Loading quota data" />
        ) : quotas.error ? (
          <ErrorState
            title="Failed to load quotas"
            message={quotas.error.message}
            onRetry={quotas.refresh}
          />
        ) : quotas.collection && quotas.collection.items.length > 0 ? (
          <QuotaOverview projects={quotas.collection.items} />
        ) : (
          <EmptyState title="No quota data" description="No quota information is available." />
        )}
      </section>
    </section>
  );
}

function QuotaOverview({
  projects,
}: {
  projects: {
    projectId: string;
    entries: { resourceType: string; limit: number; used: number; unit: string }[];
  }[];
}) {
  interface QuotaRow {
    projectId: string;
    resourceType: string;
    limit: number;
    used: number;
    unit: string;
  }

  const columnDefinitions: TableColumnDefinition<QuotaRow>[] = [
    { id: "project", header: "Project", cell: (r) => r.projectId },
    { id: "resourceType", header: "Resource type", cell: (r) => r.resourceType },
    { id: "used", header: "Used", cell: (r) => `${r.used.toLocaleString()} ${r.unit}` },
    { id: "limit", header: "Limit", cell: (r) => `${r.limit.toLocaleString()} ${r.unit}` },
    {
      id: "usagePct",
      header: "Usage",
      cell: (r) => {
        if (r.limit === 0) return "\u2014";
        const pct = Math.round((r.used / r.limit) * 100);
        return `${String(pct)}%`;
      },
    },
  ];

  const items: QuotaRow[] = projects.flatMap((p) =>
    p.entries.map((e) => ({
      projectId: p.projectId,
      resourceType: e.resourceType,
      limit: e.limit,
      used: e.used,
      unit: e.unit,
    })),
  );

  return (
    <Table<QuotaRow>
      columnDefinitions={columnDefinitions}
      items={items}
      ariaLabels={{ tableLabel: "Quota overview" }}
    />
  );
}
