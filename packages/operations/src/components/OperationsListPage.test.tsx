import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router";
import { OperationsClientProvider } from "../client/context";
import { OperationsListPage } from "./OperationsListPage";
import type { ArafClient, Operation, PaginatedCollection } from "@araf/api-client";

const operations: Operation[] = [
  {
    id: "op-0000000001",
    action: "create",
    state: "pending",
    resourceId: "resource-0000000001",
    resourceType: "compute.server",
    projectId: "project-1",
    regionId: "eu-west",
    initiatedBy: "user-1",
    startedAt: "2024-01-01T00:00:00Z",
    updatedAt: "2024-01-01T00:00:00Z",
    correlationId: "corr-1",
    error: null,
    events: [
      {
        id: "ev-1",
        state: "pending",
        occurredAt: "2024-01-01T00:00:00Z",
        message: "Pending",
        correlationId: "corr-1",
      },
    ],
  },
  {
    id: "op-0000000002",
    action: "delete",
    state: "succeeded",
    resourceId: "resource-0000000002",
    resourceType: "compute.server",
    projectId: "project-2",
    regionId: "us-east",
    initiatedBy: "user-1",
    startedAt: "2024-01-02T00:00:00Z",
    updatedAt: "2024-01-02T00:01:00Z",
    correlationId: "corr-2",
    error: null,
    events: [
      {
        id: "ev-2",
        state: "succeeded",
        occurredAt: "2024-01-02T00:01:00Z",
        message: "Done",
        correlationId: "corr-2",
      },
    ],
  },
];

const collection: PaginatedCollection<Operation> = {
  items: operations,
  total: 2,
  page: 0,
  pageSize: 25,
  hasMore: false,
};

function TestWrapper({ children, client }: { children: React.ReactNode; client?: ArafClient }) {
  const defaultClient: ArafClient = {
    healthz: vi.fn(),
    getContext: vi.fn(),
    listServices: vi.fn(),
    listResources: vi.fn(),
    getResource: vi.fn(),
    createResource: vi.fn(),
    submitAction: vi.fn(),
    listOperations: vi.fn().mockResolvedValue(collection),
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
    <OperationsClientProvider client={client ?? defaultClient}>
      <MemoryRouter>{children}</MemoryRouter>
    </OperationsClientProvider>
  );
}

describe("OperationsListPage", () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders operation rows and links to detail pages", async () => {
    render(
      <TestWrapper>
        <OperationsListPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "op-0000000001" })).toHaveAttribute(
        "href",
        "/operations/op-0000000001",
      );
    });

    expect(screen.getByRole("link", { name: "op-0000000002" })).toHaveAttribute(
      "href",
      "/operations/op-0000000002",
    );
    expect(screen.getAllByText("Pending").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Succeeded").length).toBeGreaterThanOrEqual(1);
  });

  it("shows pagination controls", async () => {
    render(
      <TestWrapper>
        <OperationsListPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("navigation", { name: "Pagination" })).toBeInTheDocument();
    });

    expect(screen.getByText(/Page 1 of 1 \(2 total\)/)).toBeInTheDocument();
  });

  it("applies state and resource filters", async () => {
    const listOperations = vi.fn().mockResolvedValue(collection);
    const client: ArafClient = {
      healthz: vi.fn(),
      getContext: vi.fn(),
      listServices: vi.fn(),
      listResources: vi.fn(),
      getResource: vi.fn(),
      createResource: vi.fn(),
      submitAction: vi.fn(),
      listOperations,
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

    render(
      <TestWrapper client={client}>
        <OperationsListPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "op-0000000001" })).toBeInTheDocument();
    });

    await userEvent.selectOptions(screen.getByLabelText("State"), "pending");
    await userEvent.type(screen.getByLabelText("Resource type"), "compute.server");

    await waitFor(() => {
      expect(listOperations).toHaveBeenLastCalledWith(
        expect.objectContaining({ state: "pending", resourceType: "compute.server" }),
      );
    });
  });

  it("displays an error state when loading fails", async () => {
    const client: ArafClient = {
      healthz: vi.fn(),
      getContext: vi.fn(),
      listServices: vi.fn(),
      listResources: vi.fn(),
      getResource: vi.fn(),
      createResource: vi.fn(),
      submitAction: vi.fn(),
      listOperations: vi.fn().mockRejectedValue(new Error("Network error")),
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

    render(
      <TestWrapper client={client}>
        <OperationsListPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Could not load operations")).toBeInTheDocument();
    });

    expect(screen.getByText("Network error")).toBeInTheDocument();
  });
});
