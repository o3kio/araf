import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router";
import { ScopeProvider } from "@araf/shell";
import type { Scope } from "@araf/shell";
import { ResourceClientProvider } from "../client/context";
import { ResourceCreatePage } from "./ResourceCreatePage";
import type { ArafClient, Operation, ServiceDescriptor, SessionContext } from "@araf/api-client";

function noop(): void {
  // intentionally empty for test harnesses
}

const scope: Scope = { projectId: "project-1", regionId: "eu-west" };

const sessionContext: SessionContext = {
  surface: "tenant",
  userId: "user-1",
  userName: "Test User",
  organizationId: "org-1",
  projectId: "project-1",
  regionId: "eu-west",
  capabilities: [{ resourceType: "compute.server", action: "create" }],
};

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
      createSchema: {
        type: "object",
        required: ["name", "regionId", "projectId"],
        properties: {
          name: {
            type: "string",
            minLength: 1,
            title: "Server name",
            description: "A unique name for the server.",
          },
          regionId: {
            enum: ["eu-west", "us-east", "ap-south"],
            title: "Region",
          },
          projectId: {
            enum: ["project-1", "project-2"],
            title: "Project",
          },
          bootVolumeSizeGb: {
            type: "number",
            minimum: 10,
            title: "Boot volume size (GB)",
            "x-araf": { advanced: true },
          },
        },
      },
      supportedActions: [],
      columns: [{ id: "name", header: "Name", field: "name" }],
      filters: [],
      sortableFields: [],
      detailsSections: [{ id: "overview", label: "Overview", fields: ["id", "name"] }],
      relationships: [],
    },
  ],
};

const createdOperation: Operation = {
  id: "op-create-1",
  action: "create",
  state: "pending",
  resourceId: "resource-new-1",
  resourceType: "compute.server",
  projectId: "project-1",
  regionId: "eu-west",
  initiatedBy: "user-1",
  startedAt: "2024-01-01T00:00:00Z",
  updatedAt: "2024-01-01T00:00:00Z",
  correlationId: "corr-create-1",
  error: null,
  events: [
    {
      id: "ev-create-1",
      state: "pending",
      occurredAt: "2024-01-01T00:00:00Z",
      message: "Operation created and pending",
      correlationId: "corr-create-1",
    },
  ],
};

function TestWrapper({
  children,
  capabilities = sessionContext.capabilities,
}: {
  children: React.ReactNode;
  capabilities?: SessionContext["capabilities"];
}) {
  const client: ArafClient = {
    healthz: vi.fn(),
    getContext: vi.fn().mockResolvedValue({ ...sessionContext, capabilities }),
    listServices: vi.fn().mockResolvedValue([serverDescriptor]),
    listResources: vi.fn(),
    getResource: vi.fn(),
    createResource: vi.fn().mockResolvedValue(createdOperation),
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

describe("ResourceCreatePage", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("renders a schema-driven form and submits valid data", async () => {
    render(
      <TestWrapper>
        <ResourceCreatePage resourceType="compute.server" />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByLabelText("Server name")).toBeInTheDocument();
    });

    await userEvent.type(screen.getByLabelText("Server name"), "web-01");
    await userEvent.selectOptions(screen.getByLabelText("Region"), "eu-west");
    await userEvent.selectOptions(screen.getByLabelText("Project"), "project-1");

    await userEvent.click(screen.getByRole("button", { name: /Create Server/i }));

    await waitFor(() => {
      expect(screen.getByText(/creation submitted/i)).toBeInTheDocument();
    });

    expect(screen.getByText(/Request accepted/i)).toBeInTheDocument();
    expect(screen.getByText(/op-create-1/i)).toBeInTheDocument();
    expect(screen.getByText(/pending/i)).toBeInTheDocument();
    expect(screen.getByText(/corr-create-1/i)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /View operation/i })).toHaveAttribute(
      "href",
      "/operations/op-create-1",
    );
  });

  it("shows inline validation errors for missing required fields", async () => {
    render(
      <TestWrapper>
        <ResourceCreatePage resourceType="compute.server" />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByLabelText("Server name")).toBeInTheDocument();
    });

    await userEvent.click(screen.getByRole("button", { name: /Create Server/i }));

    await waitFor(() => {
      expect(screen.getByText(/Please correct the errors below/i)).toBeInTheDocument();
    });

    expect(screen.getByRole("alert")).toBeInTheDocument();
  });

  it("shows an error state when the session lacks create capability", async () => {
    render(
      <TestWrapper capabilities={[]}>
        <ResourceCreatePage resourceType="compute.server" />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Create not available")).toBeInTheDocument();
    });

    expect(screen.getByText(/compute.server\/create capability required/i)).toBeInTheDocument();
  });

  it("keeps advanced fields collapsed until expanded", async () => {
    render(
      <TestWrapper>
        <ResourceCreatePage resourceType="compute.server" />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByLabelText("Server name")).toBeInTheDocument();
    });

    expect(screen.queryByLabelText("Boot volume size (GB)")).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: /Show advanced/i }));

    await waitFor(() => {
      expect(screen.getByLabelText("Boot volume size (GB)")).toBeInTheDocument();
    });
  });
});
