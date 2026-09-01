import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { ResourceClientProvider } from "../client/context";
import { ServiceCatalogPage } from "./ServiceCatalogPage";
import type { ArafClient, ServiceCatalogEntry, SessionContext } from "@araf/api-client";

const sessionContext: SessionContext = {
  surface: "tenant",
  userId: "user-1",
  userName: "Test User",
  organizationId: "org-1",
  projectId: "project-1",
  regionId: "eu-west",
  capabilities: [],
};

const catalog: ServiceCatalogEntry[] = [
  {
    id: "compute",
    namespace: "o3k.io",
    name: "Compute",
    version: "1.0.0",
    ownership: null,
    lifecycleState: "stable",
    capabilities: [{ resourceType: "compute.server", action: "list" }],
    regions: ["eu-west", "us-east"],
    description: "Virtual machines and related resources.",
  },
  {
    id: "object.storage",
    namespace: "o3k.io",
    name: "Object Storage",
    version: "2.1.0",
    ownership: null,
    lifecycleState: "stable",
    capabilities: [{ resourceType: "object.storage.bucket", action: "create" }],
    regions: [],
  },
];

function TestWrapper({
  children,
  catalogEntries = catalog,
}: {
  children: React.ReactNode;
  catalogEntries?: ServiceCatalogEntry[];
}) {
  const client: ArafClient = {
    healthz: vi.fn(),
    getContext: vi.fn().mockResolvedValue(sessionContext),
    listServices: vi.fn(),
    listServiceCatalog: vi.fn().mockResolvedValue(catalogEntries),
    listUsage: vi.fn(),
    listInstalledServices: vi.fn(),
    listDiscoveredResourceTypes: vi.fn(),
    listResources: vi.fn(),
    getResource: vi.fn(),
    createResource: vi.fn(),
    submitAction: vi.fn(),
    listOperations: vi.fn(),
    getOperation: vi.fn(),
    listProjects: vi.fn(),
    getProject: vi.fn(),
    listProjectMembers: vi.fn(),
    listUsers: vi.fn(),
    getUser: vi.fn(),
    listRoles: vi.fn(),
    listQuotas: vi.fn(),
    listAuditEvents: vi.fn(),
    listApiCredentials: vi.fn(),
    createApiCredential: vi.fn(),
    deleteApiCredential: vi.fn(),
    getPlatformOverview: vi.fn(),
    listRegions: vi.fn(),
    listAvailabilityZones: vi.fn(),
    listProviderHealth: vi.fn(),
    listServiceHealth: vi.fn(),
    getCapacitySummary: vi.fn(),
    listCustomerAccounts: vi.fn(),
    listAccountProjects: vi.fn(),
    listOperatorOperations: vi.fn(),
    listOperatorAuditEvents: vi.fn(),
  };

  return (
    <ResourceClientProvider client={client}>
      <MemoryRouter>{children}</MemoryRouter>
    </ResourceClientProvider>
  );
}

describe("ServiceCatalogPage", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("renders the service catalog entries", async () => {
    render(
      <TestWrapper>
        <ServiceCatalogPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Service catalog" })).toBeInTheDocument();
    });

    expect(screen.getByText("Compute")).toBeInTheDocument();
    expect(screen.getByText("Object Storage")).toBeInTheDocument();
    expect(screen.getByText("1.0.0")).toBeInTheDocument();
    expect(screen.getByText("eu-west, us-east")).toBeInTheDocument();
    expect(screen.getByText("compute.server:list")).toBeInTheDocument();
  });

  it("shows empty state when the catalog is empty", async () => {
    render(
      <TestWrapper catalogEntries={[]}>
        <ServiceCatalogPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("No services")).toBeInTheDocument();
    });
  });

  it("displays error state when loading fails", async () => {
    const client: ArafClient = {
      healthz: vi.fn(),
      getContext: vi.fn().mockResolvedValue(sessionContext),
      listServices: vi.fn(),
      listServiceCatalog: vi.fn().mockRejectedValue(new Error("Catalog unavailable")),
      listUsage: vi.fn(),
      listInstalledServices: vi.fn(),
      listDiscoveredResourceTypes: vi.fn(),
      listResources: vi.fn(),
      getResource: vi.fn(),
      createResource: vi.fn(),
      submitAction: vi.fn(),
      listOperations: vi.fn(),
      getOperation: vi.fn(),
      listProjects: vi.fn(),
      getProject: vi.fn(),
      listProjectMembers: vi.fn(),
      listUsers: vi.fn(),
      getUser: vi.fn(),
      listRoles: vi.fn(),
      listQuotas: vi.fn(),
      listAuditEvents: vi.fn(),
      listApiCredentials: vi.fn(),
      createApiCredential: vi.fn(),
      deleteApiCredential: vi.fn(),
      getPlatformOverview: vi.fn(),
      listRegions: vi.fn(),
      listAvailabilityZones: vi.fn(),
      listProviderHealth: vi.fn(),
      listServiceHealth: vi.fn(),
      getCapacitySummary: vi.fn(),
      listCustomerAccounts: vi.fn(),
      listAccountProjects: vi.fn(),
      listOperatorOperations: vi.fn(),
      listOperatorAuditEvents: vi.fn(),
    };

    render(
      <ResourceClientProvider client={client}>
        <MemoryRouter>
          <ServiceCatalogPage />
        </MemoryRouter>
      </ResourceClientProvider>,
    );

    await waitFor(() => {
      expect(screen.getByText("Could not load service catalog")).toBeInTheDocument();
    });

    expect(screen.getByText("Catalog unavailable")).toBeInTheDocument();
  });
});
