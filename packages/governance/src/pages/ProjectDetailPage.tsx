import { Header, LoadingState, ErrorState, SpaceBetween } from "@araf/ui";
import { useParams } from "react-router";
import { useCapabilities } from "@araf/resources";
import type { ProjectMember } from "@araf/api-client";
import { useProject } from "../hooks/useProject";
import { hasCapability } from "../capabilities";
import { errorMessage, errorCorrelationId } from "../errors";

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { project, members, loading, error } = useProject(id);
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();
  const canRead = hasCapability(capabilities, "tenant.project", "read");

  return (
    <section aria-label={`Project ${id ?? ""} detail`}>
      <Header variant="h1" headingLevel="h1">
        Project {id}
      </Header>

      {!canRead && !capabilitiesLoading ? (
        <ErrorState
          title="Access denied"
          message="You do not have permission to view this project."
        />
      ) : null}

      {error ? (
        <ErrorState
          title="Could not load project"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {!error && loading && !project ? <LoadingState message="Loading project..." /> : null}

      {canRead && !error && project && (
        <SpaceBetween size="l" direction="vertical">
          <section aria-labelledby="project-details-heading">
            <h2 id="project-details-heading">Details</h2>
            <dl>
              <DetailItem label="Name" value={project.name} />
              <DetailItem label="ID" value={project.id} />
              <DetailItem label="Organization" value={project.organizationId} />
              <DetailItem label="Status" value={project.status} />
              <DetailItem label="Created" value={formatTimestamp(project.createdAt)} />
              <DetailItem label="Updated" value={formatTimestamp(project.updatedAt)} />
            </dl>
          </section>

          <section aria-labelledby="project-members-heading">
            <h2 id="project-members-heading">Members & roles</h2>
            {members && members.length > 0 ? (
              <ul>
                {members.map((member) => (
                  <MemberItem key={member.user.id} member={member} />
                ))}
              </ul>
            ) : (
              <p>No members found for this project.</p>
            )}
          </section>
        </SpaceBetween>
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

function MemberItem({ member }: { member: ProjectMember }) {
  return (
    <li style={{ marginBottom: "0.5rem" }}>
      <strong>{member.user.name}</strong> ({member.user.email ?? member.user.id})
      <br />
      Roles: {member.roles.join(", ") || "—"}
    </li>
  );
}
