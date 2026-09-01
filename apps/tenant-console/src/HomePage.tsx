import { Button, ColumnLayout, Container, Header, SpaceBetween } from "@araf/ui";
import { useScope } from "@araf/shell";
import "./home.css";

function ContextValue({ label, value }: { label: string; value: string }) {
  return (
    <div className="araf-home__context-value">
      <span className="araf-home__eyebrow">{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

export function HomePage() {
  const { scope } = useScope();
  const project = scope.projectName ?? scope.projectId ?? "No project selected";
  const region = scope.regionName ?? scope.regionId ?? "Global";
  const organization = scope.organizationName ?? scope.organizationId ?? "Current organization";

  return (
    <SpaceBetween size="l">
      <Header
        variant="h1"
        description={`Operate ${project} in ${region}. Cloud state and operations remain authoritative in O3K.`}
      >
        Tenant home
      </Header>

      <Container header={<Header variant="h2">Current context</Header>}>
        <ColumnLayout columns={3} variant="text-grid">
          <ContextValue label="Organization" value={organization} />
          <ContextValue label="Project" value={project} />
          <ContextValue label="Region" value={region} />
        </ColumnLayout>
      </Container>

      <ColumnLayout columns={3}>
        <Container header={<Header variant="h2">Cloud services</Header>}>
          <SpaceBetween size="m">
            <p className="araf-home__description">
              Discover the services enabled for this project and manage their resources through the
              generic O3K resource model.
            </p>
            <div className="araf-home__actions">
              <Button variant="primary" href="/services/catalog">
                Service catalog
              </Button>
              <Button href="/resources">All resources</Button>
            </div>
          </SpaceBetween>
        </Container>

        <Container header={<Header variant="h2">Operations</Header>}>
          <SpaceBetween size="m">
            <p className="araf-home__description">
              Follow asynchronous cloud work through canonical O3K Operations instead of transient
              frontend status.
            </p>
            <Button href="/operations">Open Operations</Button>
          </SpaceBetween>
        </Container>

        <Container header={<Header variant="h2">Governance</Header>}>
          <SpaceBetween size="m">
            <p className="araf-home__description">
              Review project access, quotas, usage and audit information for the active tenant scope.
            </p>
            <div className="araf-home__actions">
              <Button href="/organization/projects">Projects</Button>
              <Button href="/organization/quotas">Quotas</Button>
            </div>
          </SpaceBetween>
        </Container>
      </ColumnLayout>

      <Container header={<Header variant="h2">Automation first</Header>}>
        <div className="araf-home__automation">
          <div>
            <span className="araf-home__eyebrow">Same control plane</span>
            <p className="araf-home__description">
              Araf is a client of the O3K native API. Console workflows must remain reproducible by
              API, CLI and infrastructure automation rather than becoming console-only behavior.
            </p>
          </div>
          <Button href="/developer/api">API &amp; CLI</Button>
        </div>
      </Container>
    </SpaceBetween>
  );
}
