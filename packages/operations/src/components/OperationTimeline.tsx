import type { OperationEvent } from "@araf/api-client";
import { OperationStatus } from "./OperationStatus";

export interface OperationTimelineProps {
  events: readonly OperationEvent[];
}

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

/**
 * Render an Operation's authoritative event timeline.
 *
 * Events are displayed newest-first so the most recent state is at the top.
 */
export function OperationTimeline({ events }: OperationTimelineProps) {
  if (events.length === 0) {
    return <p>No events recorded.</p>;
  }

  const ordered = [...events].sort(
    (a, b) => new Date(b.occurredAt).getTime() - new Date(a.occurredAt).getTime(),
  );

  return (
    <ol aria-label="Operation timeline" style={{ listStyle: "none", paddingInlineStart: 0 }}>
      {ordered.map((event) => (
        <li
          key={event.id}
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "0.25rem",
            padding: "0.75rem 0",
            borderBottom: "1px solid var(--awsui-color-border-divider-default, #e9ebed)",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <OperationStatus state={event.state} />
            <time dateTime={event.occurredAt}>{formatTimestamp(event.occurredAt)}</time>
          </div>
          <p style={{ margin: 0 }}>{event.message}</p>
          <p style={{ margin: 0, fontSize: "0.875rem", color: "#5f6b7a" }}>
            Correlation ID: {event.correlationId}
          </p>
        </li>
      ))}
    </ol>
  );
}
