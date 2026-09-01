import type { ReactNode } from "react";
import { Header, BreadcrumbGroup, Tabs, LoadingState, ErrorState } from "@araf/ui";
import { useParams } from "react-router";
import type { Resource } from "@araf/api-client";
import { useResourceDetail } from "../hooks/useResourceDetail";
import { useResourceDescriptor } from "../hooks/useResourceDescriptor";
import { getResourceField, formatResourceField } from "../fields";
import { mapResourceStatus } from "../status";
import { RelationshipPanel } from "./RelationshipPanel";
import { ResourceActionsPanel } from "./ResourceActionsPanel";
import type { TabItem } from "@araf/ui";
import type { ResourceDescriptor, DetailsSectionDescriptor } from "../descriptor";
import { errorMessage, errorCorrelationId } from "../errors";

export interface ResourceDetailPageProps {
  resourceType: string;
}

/**
 * Generic resource detail page rendered from the resource descriptor.
 *
 * Only renders tabs that the descriptor supports. The Overview tab always
 * shows the configured detail sections.
 */
export function ResourceDetailPage({ resourceType }: ResourceDetailPageProps) {
  const { id } = useParams<{ id: string }>();
  const {
    descriptor,
    loading: descriptorLoading,
    error: descriptorError,
  } = useResourceDescriptor(resourceType);
  const {
    resource,
    loading: resourceLoading,
    error: resourceError,
  } = useResourceDetail(resourceType, id ?? "");

  const loading = descriptorLoading || resourceLoading;
  const error = descriptorError ?? resourceError;

  const breadcrumbItems = [
    {
      text: descriptor?.pluralName ?? resourceType,
      href: `/resources/${encodeURIComponent(resourceType)}`,
    },
    { text: resource?.name ?? id ?? "Detail", href: `#` },
  ];

  const tabs: TabItem[] = [];

  if (descriptor && resource) {
    tabs.push({
      id: "overview",
      label: "Overview",
      content: <OverviewPanel resource={resource} descriptor={descriptor} />,
    });

    const configSections = descriptor.detailsSections.filter((s) => s.id !== "overview");
    if (configSections.length > 0) {
      tabs.push({
        id: "configuration",
        label: "Configuration",
        content: <ConfigurationPanel resource={resource} sections={configSections} />,
      });
    }

    if (descriptor.relationships.length > 0) {
      tabs.push({
        id: "relationships",
        label: "Relationships",
        content: (
          <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
            {descriptor.relationships.map((relationship) => (
              <RelationshipPanel
                key={relationship.id}
                resource={resource}
                relationship={relationship}
              />
            ))}
          </div>
        ),
      });
    }

    tabs.push({
      id: "operations",
      label: "Operations",
      content: <OperationsPanel resource={resource} />,
    });
  }

  return (
    <section aria-label={`${descriptor?.name ?? resourceType} detail`}>
      <BreadcrumbGroup items={breadcrumbItems} ariaLabel="Resource navigation" />
      <Header
        variant="h1"
        headingLevel="h1"
        description={resource ? `ID: ${resource.id}` : "Resource detail"}
      >
        {resource?.name ?? descriptor?.name ?? resourceType}
      </Header>

      {error ? (
        <ErrorState
          title="Could not load resource"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {!error && loading && !resource ? <LoadingState message="Loading resource..." /> : null}

      {!error && resource && descriptor ? (
        <>
          <div style={{ margin: "1rem 0" }}>
            <ResourceActionsPanel resource={resource} descriptor={descriptor} />
          </div>
          <Tabs tabs={tabs} ariaLabel="Resource detail tabs" />
        </>
      ) : null}
    </section>
  );
}

interface OverviewPanelProps {
  resource: Resource;
  descriptor: ResourceDescriptor;
}

function OverviewPanel({ resource, descriptor }: OverviewPanelProps) {
  const overviewSection = descriptor.detailsSections.find((s) => s.id === "overview");
  const sections = overviewSection ? [overviewSection] : descriptor.detailsSections;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}>
      {sections.map((section) => (
        <section key={section.id} aria-labelledby={`section-${section.id}`}>
          <h2 id={`section-${section.id}`}>{section.label}</h2>
          <dl>
            {section.fields.map((field) => {
              const value = getResourceField(resource, field);
              let display: ReactNode = formatResourceField(value);
              if (field === "status") {
                const { label } = mapResourceStatus(resource.status);
                display = <span>{label}</span>;
              }
              return (
                <div key={field} style={{ marginBottom: "0.5rem" }}>
                  <dt style={{ fontWeight: "bold" }}>{field}</dt>
                  <dd style={{ marginInlineStart: 0 }}>{display}</dd>
                </div>
              );
            })}
          </dl>
        </section>
      ))}
    </div>
  );
}

interface ConfigurationPanelProps {
  resource: Resource;
  sections: readonly DetailsSectionDescriptor[];
}

function ConfigurationPanel({ resource, sections }: ConfigurationPanelProps) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}>
      {sections.map((section) => (
        <section key={section.id} aria-labelledby={`section-${section.id}`}>
          <h2 id={`section-${section.id}`}>{section.label}</h2>
          <dl>
            {section.fields.map((field) => (
              <div key={field} style={{ marginBottom: "0.5rem" }}>
                <dt style={{ fontWeight: "bold" }}>{field}</dt>
                <dd style={{ marginInlineStart: 0 }}>
                  {formatResourceField(getResourceField(resource, field))}
                </dd>
              </div>
            ))}
          </dl>
        </section>
      ))}
    </div>
  );
}

interface OperationsPanelProps {
  resource: Resource;
}

function OperationsPanel({ resource }: OperationsPanelProps) {
  return (
    <div>
      <p>
        Operations for <strong>{resource.name}</strong> will be shown here.
      </p>
    </div>
  );
}
