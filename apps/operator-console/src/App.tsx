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

interface Provider {
  id: string;
  name: string;
  kind: string;
  region: string;
  health: "healthy" | "degraded" | "critical";
}

const providers: Provider[] = [
  { id: "p1", name: "core-eu", kind: "Compute", region: "eu-west", health: "healthy" },
  { id: "p2", name: "net-us", kind: "Network", region: "us-east", health: "degraded" },
];

const columns: TableColumnDefinition<Provider>[] = [
  { id: "name", header: "Name", cell: (item) => item.name },
  { id: "kind", header: "Kind", cell: (item) => item.kind },
  { id: "region", header: "Region", cell: (item) => item.region },
  {
    id: "health",
    header: "Health",
    cell: (item) => (
      <StatusIndicator
        type={
          item.health === "healthy" ? "success" : item.health === "degraded" ? "warning" : "error"
        }
      >
        {item.health}
      </StatusIndicator>
    ),
  },
];

const tabs = [
  { id: "platform", label: "Platform", content: <p>Platform-wide health and capacity.</p> },
  { id: "customers", label: "Customers", content: <p>Accounts and organizations.</p> },
];

export function App() {
  const [density, setDensity] = useState<ArafDensity>("compact");
  const [activeTab, setActiveTab] = useState("platform");
  const [showPause, setShowPause] = useState(false);
  const [toasts, setToasts] = useState<ToastItem[]>([
    { id: "o1", type: "warning", message: "Operator console loaded" },
  ]);

  return (
    <ArafThemeProvider density={density}>
      <div style={{ padding: "24px" }}>
        <BreadcrumbGroup
          items={[
            { text: "Platform", href: "/" },
            { text: "Operator Console", href: "/operator" },
          ]}
        />

        <Header variant="h1" description="Management console for O3K platform operators.">
          Araf Operator Console
        </Header>

        <DensityMode density={density} onChange={setDensity} />

        <Tabs
          tabs={tabs}
          activeTabId={activeTab}
          onChange={setActiveTab}
          ariaLabel="Operator sections"
        />

        <FormSection title="Region action" description="Apply a controlled change to a region.">
          <FormField label="Region" id="region">
            <select id="region">
              <option>eu-west</option>
              <option>us-east</option>
            </select>
          </FormField>
          <FormField label="Reason" id="reason">
            <input id="reason" type="text" placeholder="Maintenance reference" />
          </FormField>
        </FormSection>

        <Header variant="h2">Providers</Header>
        <Table<Provider>
          items={providers}
          columnDefinitions={columns}
          trackingId="id"
          header={<Header variant="h3">Infrastructure providers</Header>}
          empty={<EmptyState title="No providers" description="No provider data is available." />}
        />

        <Header variant="h2">States</Header>
        <LoadingState message="Loading platform diagnostics..." />
        <ErrorState
          title="Capacity feed unavailable"
          message="The capacity service did not respond."
          correlationId="txn-operator-7"
          onRetry={() => {
            setToasts((prev) => [
              ...prev,
              { id: String(Date.now()), type: "info", message: "Retry requested" },
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
            setShowPause(true);
          }}
        >
          Open operator confirm modal
        </button>
        <ConfirmModal
          open={showPause}
          title="Pause provider?"
          onConfirm={() => {
            setShowPause(false);
            setToasts((prev) => [
              ...prev,
              { id: String(Date.now()), type: "success", message: "Provider paused" },
            ]);
          }}
          onCancel={() => {
            setShowPause(false);
          }}
          danger
        >
          <p>Traffic will be drained from this provider.</p>
        </ConfirmModal>
      </div>
    </ArafThemeProvider>
  );
}
