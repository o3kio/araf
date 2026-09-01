import { Header, LoadingState, ErrorState } from "@araf/ui";
import { useParams } from "react-router";
import { useCapabilities } from "@araf/resources";
import { useUser } from "../hooks/useUser";
import { hasCapability } from "../capabilities";
import { errorMessage, errorCorrelationId } from "../errors";

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function UserDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { user, loading, error } = useUser(id);
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();
  const canRead = hasCapability(capabilities, "tenant.user", "read");

  return (
    <section aria-label={`User ${id ?? ""} detail`}>
      <Header variant="h1" headingLevel="h1">
        User {id}
      </Header>

      {!canRead && !capabilitiesLoading ? (
        <ErrorState title="Access denied" message="You do not have permission to view this user." />
      ) : null}

      {error ? (
        <ErrorState
          title="Could not load user"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {!error && loading && !user ? <LoadingState message="Loading user..." /> : null}

      {canRead && !error && user && (
        <section aria-labelledby="user-details-heading">
          <h2 id="user-details-heading">Details</h2>
          <dl>
            <DetailItem label="Name" value={user.name} />
            <DetailItem label="ID" value={user.id} />
            <DetailItem label="Email" value={user.email ?? "—"} />
            <DetailItem label="Status" value={user.status} />
            <DetailItem label="Created" value={formatTimestamp(user.createdAt)} />
          </dl>
        </section>
      )}
    </section>
  );
}

function DetailItem({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ marginBottom: "0.5rem" }}>
      <dt style={{ fontWeight: "bold" }}>{label}</dt>
      <dd style={{ marginInlineStart: 0 }}>{value}</dd>
    </div>
  );
}
