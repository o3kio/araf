import type { ReactNode } from "react";
import { Header, LoadingState, ErrorState, SpaceBetween } from "@araf/ui";
import { useParams } from "react-router";
import { useOperation } from "../hooks/useOperation";
import { useOperationTransport } from "../hooks/useOperationTransport";
import { OperationStatus } from "./OperationStatus";
import { OperationTimeline } from "./OperationTimeline";
import { errorMessage, errorCorrelationId } from "../errors";

function formatTimestamp(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function OperationDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { operation, loading, error, refresh } = useOperation(id);

  const terminal = operation?.state === "succeeded" || operation?.state === "failed";
  useOperationTransport(refresh, !terminal && !loading && !error);

  return (
    <section aria-label={`Operation ${id ?? ""} detail`}>
      <Header
        variant="h1"
        headingLevel="h1"
        description={operation ? `Action: ${operation.action}` : "Operation detail"}
        actions={operation ? <OperationStatus state={operation.state} /> : undefined}
      >
        Operation {id}
      </Header>

      {error ? (
        <ErrorState
          title="Could not load operation"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {!error && loading && !operation ? <LoadingState message="Loading operation..." /> : null}

      {!error && operation && (
        <SpaceBetween size="l" direction="vertical">
          <section aria-labelledby="operation-details-heading">
            <h2 id="operation-details-heading">Details</h2>
            <dl>
              <DetailItem label="Action" value={operation.action} />
              <DetailItem label="State" value={<OperationStatus state={operation.state} />} />
              <DetailItem
                label="Resource"
                value={
                  operation.resourceType && operation.resourceId
                    ? `${operation.resourceType}/${operation.resourceId}`
                    : "—"
                }
              />
              <DetailItem label="Project" value={operation.projectId ?? "—"} />
              <DetailItem label="Region" value={operation.regionId ?? "—"} />
              <DetailItem label="Initiator" value={operation.initiatedBy ?? "—"} />
              <DetailItem label="Started" value={formatTimestamp(operation.startedAt)} />
              <DetailItem label="Updated" value={formatTimestamp(operation.updatedAt)} />
              <DetailItem label="Correlation ID" value={operation.correlationId} />
            </dl>
          </section>

          {operation.error ? (
            <section aria-labelledby="operation-error-heading">
              <h2 id="operation-error-heading">Error</h2>
              <dl>
                <DetailItem label="Code" value={operation.error.code} />
                <DetailItem label="Title" value={operation.error.title} />
                <DetailItem label="Detail" value={operation.error.detail} />
              </dl>
            </section>
          ) : null}

          <section aria-labelledby="operation-timeline-heading">
            <h2 id="operation-timeline-heading">Timeline</h2>
            <OperationTimeline events={operation.events} />
          </section>
        </SpaceBetween>
      )}
    </section>
  );
}

function DetailItem({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div style={{ marginBottom: "0.5rem" }}>
      <dt style={{ fontWeight: "bold" }}>{label}</dt>
      <dd style={{ marginInlineStart: 0 }}>{value}</dd>
    </div>
  );
}
