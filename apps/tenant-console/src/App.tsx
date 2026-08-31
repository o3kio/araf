import { useState } from "react";
import {
  ArafThemeProvider,
  BreadcrumbGroup,
  ConfirmModal,
  DensityMode,
  EmptyState,
  ErrorState,
  FormField,
  FormSection,
  Header,
  LoadingState,
  StatusIndicator,
  Table,
  Tabs,
  Toast,
  type ArafDensity,
  type TableColumnDefinition,
  type ToastItem,
} from "@araf/ui";

interface Resource {
  id: string;
  name: string;
  type: string;
  region: string;
  status: "ready" | "busy" | "error";
}

const resources: Resource[] = [
  { id: "r1", name: "web-fe", type: "Compute", region: "eu-west", status: "ready" },
  { id: "r2", name: "api-db", type: "Database", region: "us-east", status: "busy" },
];

const columns: TableColumnDefinition<Resource>[] = [
  { id: "name", header: "Name", cell: (item) => item.name },
  { id: "type", header: "Type", cell: (item) => item.type },
  { id: "region", header: "Region", cell: (item) => item.region },
  {
    id: "status",
    header: "Status",
    cell: (item) => (
      <StatusIndicator
        type={item.status === "ready" ? "success" : item.status === "busy" ? "pending" : "error"}
      >
        {item.status}
      </StatusIndicator>
    ),
  },
];

const tabs = [
  { id: "resources", label: "Resources", content: <p>Resource list for the selected project.</p> },
  { id: "usage", label: "Usage", content: <p>Usage summary for the current billing period.</p> },
];

export function App() {
  const [density, setDensity] = useState<ArafDensity>("comfortable");
  const [activeTab, setActiveTab] = useState("resources");
  const [showDelete, setShowDelete] = useState(false);
  const [toasts, setToasts] = useState<ToastItem[]>([
    { id: "t1", type: "info", message: "Tenant console loaded" },
  ]);

  return (
    <ArafThemeProvider density={density}>
      <main style={{ padding: "var(--space-scaled-xl, 24px)" }}>
        <BreadcrumbGroup
          items={[
            { text: "Home", href: "/" },
            { text: "Tenant Console", href: "/tenant" },
          ]}
        />

        <Header variant="h1" description="Self-service console for O3K tenants.">
          Araf Tenant Console
        </Header>

        <DensityMode density={density} onChange={setDensity} />

        <Tabs
          tabs={tabs}
          activeTabId={activeTab}
          onChange={setActiveTab}
          ariaLabel="Tenant sections"
        />

        <FormSection title="Create resource" description="Enter the required resource details.">
          <FormField label="Resource name" id="name">
            <input id="name" type="text" placeholder="e.g. web-fe" />
          </FormField>
          <FormField label="Region" id="region">
            <select id="region">
              <option>eu-west</option>
              <option>us-east</option>
            </select>
          </FormField>
        </FormSection>

        <Header variant="h2">Resources</Header>
        <Table<Resource>
          items={resources}
          columnDefinitions={columns}
          trackingId="id"
          header={<Header variant="h3">Recent resources</Header>}
          empty={<EmptyState title="No resources" description="Create your first resource." />}
        />

        <Header variant="h2">States</Header>
        <LoadingState message="Loading resource counts..." />
        <ErrorState
          title="Unable to load quotas"
          message="The quota service returned a transient error."
          correlationId="txn-tenant-42"
          onRetry={() => {
            setToasts((prev) => [
              ...prev,
              { id: String(Date.now()), type: "success", message: "Retry requested" },
            ]);
          }}
        />

        <Toast
          items={toasts}
          onDismiss={(id) => {
            setToasts((prev) => prev.filter((t) => t.id !== id));
          }}
        />

        <button
          type="button"
          onClick={() => {
            setShowDelete(true);
          }}
        >
          Open confirm modal
        </button>
        <ConfirmModal
          open={showDelete}
          title="Delete resource?"
          onConfirm={() => {
            setShowDelete(false);
            setToasts((prev) => [
              ...prev,
              { id: String(Date.now()), type: "success", message: "Resource deleted" },
            ]);
          }}
          onCancel={() => {
            setShowDelete(false);
          }}
        >
          <p>This action cannot be undone.</p>
        </ConfirmModal>
      </main>
    </ArafThemeProvider>
  );
}
