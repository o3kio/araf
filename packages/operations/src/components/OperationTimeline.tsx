import type { Operation, OperationEvent } from "@araf/api-client";
import { OperationStatus } from "./OperationStatus";

export interface OperationTimelineProps {
  operation: Operation;
}

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

function deriveEvents(operation: Operation): OperationEvent[] {
  const events: OperationEvent[] = [];
  const correlationId = operation.correlationId;

  if (operation.startedAt) {
    events.push({
      id: `${operation.id}-pending`,
      state: "pending",
      occurredAt: operation.startedAt,
      message: "Operation created and pending",
      correlationId,
    });
  }

  if (operation.updatedAt && operation.updatedAt !== operation.startedAt) {
    if (operation.state === "succeeded") {
      events.push({
        id: `${operation.id}-succeeded`,
        state: "succeeded",
        occurredAt: operation.updatedAt,
        message: "Operation completed successfully",
        correlationId,
      });
    } else if (operation.state === "failed") {
      events.push({
        id: `${operation.id}-failed`,
        state: "failed",
        occurredAt: operation.updatedAt,
        message: operation.error
          ? `Operation failed: ${operation.error.detail}`
          : "Operation failed",
        correlationId,
      });
    } else if (operation.state === "retryable") {
      events.push({
        id: `${operation.id}-retryable`,
        state: "retryable",
        occurredAt: operation.updatedAt,
        message: operation.error
          ? `Operation retryable: ${operation.error.detail}`
          : "Operation retryable",
        correlationId,
      });
    } else if (operation.state === "unknownOutcome") {
      events.push({
        id: `${operation.id}-unknown-outcome`,
        state: "unknownOutcome",
        occurredAt: operation.updatedAt,
        message: operation.error
          ? `Operation outcome unknown: ${operation.error.detail}`
          : "Operation outcome unknown",
        correlationId,
      });
    } else if (operation.state === "running") {
      events.push({
        id: `${operation.id}-running`,
        state: "running",
        occurredAt: operation.updatedAt,
        message: "Operation started running",
        correlationId,
      });
    }
  }

  return events;
}

/**
 * Render an Operation's event timeline.
 *
 * If the upstream Operation has no authoritative events (O3K does not expose
 * an event array), a minimal timeline is derived from `startedAt`, `updatedAt`,
 * `state`, and `error`. No intermediate orchestration steps are invented.
 */
export function OperationTimeline({ operation }: OperationTimelineProps) {
  const events = operation.events.length > 0 ? operation.events : deriveEvents(operation);

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
