import { useEffect, useState } from "react";
import { Header, LoadingState, ErrorState } from "@araf/ui";
import { useParams } from "react-router";
import type { AuditEvent } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export function AuditDetailPage() {
  const { id } = useParams<{ id: string }>();
  const client = useGovernanceClient();
  const [event, setEvent] = useState<AuditEvent>();
  const [error, setError] = useState<Error>();
  useEffect(() => {
    if (!id || !client.getAuditEvent) return;
    client
      .getAuditEvent(id)
      .then((result) => {
        setEvent(result);
      })
      .catch((reason: unknown) => {
        setError(reason instanceof Error ? reason : new Error(String(reason)));
      });
  }, [client, id]);
  return (
    <section aria-label="Audit event detail">
      <Header variant="h1" headingLevel="h1">
        Audit event
      </Header>
      {!event && !error ? <LoadingState message="Loading audit event..." /> : null}
      {error ? <ErrorState title="Could not load audit event" message={error.message} /> : null}
      {event ? (
        <dl>
          <dt>Event ID</dt>
          <dd>{event.id}</dd>
          <dt>Actor</dt>
          <dd>{event.actor}</dd>
          <dt>Action</dt>
          <dd>{event.action}</dd>
          <dt>Outcome</dt>
          <dd>{event.outcome}</dd>
          <dt>Resource</dt>
          <dd>
            {event.resourceType && event.resourceId
              ? `${event.resourceType}/${event.resourceId}`
              : "—"}
          </dd>
          <dt>Correlation ID</dt>
          <dd>{event.correlationId}</dd>
          <dt>Recorded</dt>
          <dd>{event.recordedAt}</dd>
        </dl>
      ) : null}
    </section>
  );
}
