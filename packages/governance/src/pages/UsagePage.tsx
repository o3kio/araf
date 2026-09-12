import { useMemo, useState } from "react";
import type { MeterUsage } from "@araf/api-client";
import { EmptyState, ErrorState, LoadingState, Table, type TableColumnDefinition } from "@araf/ui";
import { useUsage, type UseUsageResult } from "../hooks/useUsage";
import { useQuotas } from "../hooks/useQuotas";

function dateInputValue(date: Date): string {
  // O3K metering v1 requires UTC hour-aligned boundaries. Keep the browser
  // control explicit about that contract instead of silently rounding a
  // tenant-selected range in the BFF.
  const aligned = new Date(Math.floor(date.getTime() / 3_600_000) * 3_600_000);
  return aligned.toISOString().slice(0, 16);
}

const NOW = new Date();
const SEVEN_DAYS_AGO = new Date(NOW.getTime() - 7 * 24 * 60 * 60 * 1000);

interface UsageContentProps {
  usage: UseUsageResult;
  quotas: ReturnType<typeof useQuotas>;
}

function UsageContent({ usage, quotas }: UsageContentProps) {
  const quotaMap = useMemo(() => {
    const map = new Map<string, { limit: number | null; unit: string }>();
    for (const project of quotas.collection?.items ?? []) {
      for (const entry of project.entries) {
        map.set(entry.resourceType, { limit: entry.limit, unit: entry.unit });
      }
    }
    return map;
  }, [quotas.collection]);

  const latestByType = useMemo(() => {
    const map = new Map<string, { value: number; unit: string; timestamp: string }>();
    for (const record of usage.summary?.records ?? []) {
      map.set(record.resourceType, {
        value: record.value,
        unit: record.unit,
        timestamp: record.timestamp,
      });
    }
    return map;
  }, [usage.summary?.records]);

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
  const meters = summary?.meters ?? [];
  if (!summary || (summary.records.length === 0 && meters.length === 0)) {
    return (
      <EmptyState
        title="No usage data"
        description="No usage records are available for the selected period."
      />
    );
  }

  if (meters.length > 0) {
    return (
      <section aria-label="Authoritative usage summary">
        <p>
          Usage is reported by O3K in exact meter units. Completeness is shown for each series; cost
          is unavailable because this O3K profile does not advertise authoritative pricing.
        </p>
        <Table<MeterUsage>
          columnDefinitions={[
            { id: "meter", header: "Meter", cell: (row) => row.meterKey },
            { id: "total", header: "Total", cell: (row) => `${row.total} ${row.unit}` },
            { id: "status", header: "Completeness", cell: (row) => row.status },
            {
              id: "period",
              header: "Period (UTC)",
              cell: (row) =>
                `${new Date(row.start).toLocaleString()} – ${new Date(row.end).toLocaleString()}`,
            },
            {
              id: "observed",
              header: "Observed through",
              cell: (row) => new Date(row.observedThrough).toLocaleString(),
            },
          ]}
          items={meters}
          ariaLabels={{ tableLabel: "Authoritative metering usage" }}
        />
        {summary.definitions && summary.definitions.length > 0 ? (
          <p>
            Advertised meters: {summary.definitions.map((definition) => definition.key).join(", ")}.
          </p>
        ) : null}
      </section>
    );
  }

  interface UsageRow {
    resourceType: string;
    value: number;
    unit: string;
    timestamp: string;
    limit: number | null | undefined;
  }

  const columnDefinitions: TableColumnDefinition<UsageRow>[] = [
    { id: "resourceType", header: "Resource type", cell: (r) => r.resourceType },
    { id: "value", header: "Current usage", cell: (r) => `${r.value.toLocaleString()} ${r.unit}` },
    {
      id: "limit",
      header: "Limit",
      cell: (r) => (r.limit != null ? `${r.limit.toLocaleString()} ${r.unit}` : "\u2014"),
    },
    {
      id: "usage",
      header: "Usage",
      cell: (r) => {
        if (r.limit == null || r.limit === 0) return "\u2014";
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
            step={3600}
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
            step={3600}
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
    entries: { resourceType: string; limit: number | null; used: number; unit: string }[];
  }[];
}) {
  interface QuotaRow {
    projectId: string;
    resourceType: string;
    limit: number | null;
    used: number;
    unit: string;
  }

  const columnDefinitions: TableColumnDefinition<QuotaRow>[] = [
    { id: "project", header: "Project", cell: (r) => r.projectId },
    { id: "resourceType", header: "Resource type", cell: (r) => r.resourceType },
    { id: "used", header: "Used", cell: (r) => `${r.used.toLocaleString()} ${r.unit}` },
    {
      id: "limit",
      header: "Limit",
      cell: (r) =>
        r.limit == null ? `Unlimited ${r.unit}` : `${r.limit.toLocaleString()} ${r.unit}`,
    },
    {
      id: "usagePct",
      header: "Usage",
      cell: (r) => {
        if (r.limit == null || r.limit === 0) return "\u2014";
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
