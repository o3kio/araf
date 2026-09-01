import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { ScopeProvider } from "@araf/shell";
import type { Scope } from "@araf/shell";
import { ResourceClientProvider } from "../client/context";
import { ResourceCollectionPage } from "./ResourceCollectionPage";
import type {
  ArafClient,
  Resource,
  ServiceDescriptor,
  PaginatedCollection,
  SessionContext,
} from "@araf/api-client";

function noop(): void {
  // intentionally empty for test harnesses
}

const scope: Scope = { projectId: "project-1", regionId: "eu-west" };

const serverDescriptor: ServiceDescriptor = {
  id: "compute",
  name: "Compute",
  category: "Services",
  resourceTypes: [
    {
      id: "compute.server",
      name: "Server",
      pluralName: "Servers",
      iconToken: "server",
      createCapability: { resourceType: "compute.server", action: "create" },
      supportedActions: [],
      columns: [
        { id: "name", header: "Name", field: "name" },
        { id: "status", header: "Status", field: "status" },
      ],
      filters: [{ id: "project", label: "Project", field: "projectId", kind: "select" }],
      sortableFields: ["name"],
      detailsSections: [{ id: "overview", label: "Overview", fields: ["id", "name"] }],
      relationships: [],
    },
  ],
};

const sessionContext: SessionContext = {
  surface: "tenant",
  userId: "user-1",
  userName: "Test User",
  organizationId: "org-1",
  projectId: "project-1",
  regionId: "eu-west",
  capabilities: [{ resourceType: "compute.server", action: "create" }],
};

const resources: Resource[] = [
  {
    id: "resource-0000000001",
    name: "server-one",
    resourceType: "compute.server",
    projectId: "project-1",
    regionId: "eu-west",
    status: "ready",
    createdAt: "2024-01-01T00:00:00Z",
    updatedAt: "2024-01-01T00:01:00Z",
  },
  {
    id: "resource-0000000002",
    name: "server-two",
    resourceType: "compute.server",
    projectId: "project-1",
    regionId: "eu-west",
    status: "error",
    createdAt: "2024-01-01T00:00:00Z",
    updatedAt: "2024-01-01T00:01:00Z",
  },
];

const collection: PaginatedCollection<Resource> = {
  items: resources,
  total: 2,
  page: 0,
  pageSize: 25,
  hasMore: false,
};

function TestWrapper({ children }: { children: React.ReactNode }) {
  const client: ArafClient = {
    healthz: vi.fn(),
    getContext: vi.fn().mockResolvedValue(sessionContext),
    listServices: vi.fn().mockResolvedValue([serverDescriptor]),
    listResources: vi.fn().mockResolvedValue(collection),
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
    listServiceCatalog: vi.fn(),
    listUsage: vi.fn(),
    listInstalledServices: vi.fn(),
    listDiscoveredResourceTypes: vi.fn(),
    getCapacitySummary: vi.fn(),
    listCustomerAccounts: vi.fn(),
    listAccountProjects: vi.fn(),
    listOperatorOperations: vi.fn(),
    listOperatorAuditEvents: vi.fn(),
  };

  return (
    <ResourceClientProvider client={client}>
      <ScopeProvider scope={scope} onChange={noop}>
        <MemoryRouter>{children}</MemoryRouter>
      </ScopeProvider>
    </ResourceClientProvider>
  );
}

describe("ResourceCollectionPage", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("renders rows, status indicators, and row links", async () => {
    render(
      <TestWrapper>
        <ResourceCollectionPage resourceType="compute.server" />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "server-one" })).toHaveAttribute(
        "href",
        "/resources/compute.server/resource-0000000001",
      );
    });

    expect(screen.getByRole("link", { name: "server-two" })).toHaveAttribute(
      "href",
      "/resources/compute.server/resource-0000000002",
    );
    expect(screen.getByText("Ready")).toBeInTheDocument();
    expect(screen.getByText("Error")).toBeInTheDocument();
  });

  it("shows pagination controls", async () => {
    render(
      <TestWrapper>
        <ResourceCollectionPage resourceType="compute.server" />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("navigation", { name: "Pagination" })).toBeInTheDocument();
    });

    expect(screen.getByText(/Page 1 of 1 \(2 total\)/)).toBeInTheDocument();
  });

  it("displays error state with correlation id on failure", async () => {
    const TestWrapperWithError = ({ children }: { children: React.ReactNode }) => {
      const errorClient: ArafClient = {
        healthz: vi.fn(),
        getContext: vi.fn().mockResolvedValue(sessionContext),
        listServices: vi.fn().mockRejectedValue(new Error("Network error")),
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
        listServiceCatalog: vi.fn(),
        listUsage: vi.fn(),
        listInstalledServices: vi.fn(),
        listDiscoveredResourceTypes: vi.fn(),
        getCapacitySummary: vi.fn(),
        listCustomerAccounts: vi.fn(),
        listAccountProjects: vi.fn(),
        listOperatorOperations: vi.fn(),
        listOperatorAuditEvents: vi.fn(),
      };

      return (
        <ResourceClientProvider client={errorClient}>
          <ScopeProvider scope={scope} onChange={noop}>
            <MemoryRouter>{children}</MemoryRouter>
          </ScopeProvider>
        </ResourceClientProvider>
      );
    };

    render(
      <TestWrapperWithError>
        <ResourceCollectionPage resourceType="compute.server" />
      </TestWrapperWithError>,
    );

    await waitFor(() => {
      expect(screen.getByText("Could not load resources")).toBeInTheDocument();
    });

    expect(screen.getByText("Network error")).toBeInTheDocument();
  });
});
