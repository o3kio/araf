import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router";
import { ScopeProvider } from "@araf/shell";
import type { Scope } from "@araf/shell";
import { ResourceClientProvider } from "../client/context";
import { OperationsClientProvider } from "@araf/operations";
import { ResourceDetailPage } from "./ResourceDetailPage";
import type {
  ArafClient,
  Resource,
  ServiceDescriptor,
  SessionContext,
  PaginatedCollection,
  Operation,
} from "@araf/api-client";

function noop(): void {
  // intentionally empty for test harnesses
}

const scope: Scope = { projectId: "project-1", regionId: "eu-west" };

const volumeDescriptor: ServiceDescriptor = {
  id: "storage",
  name: "Storage",
  category: "Services",
  resourceTypes: [
    {
      id: "storage.volume",
      name: "Volume",
      pluralName: "Volumes",
      iconToken: "storage",
      createCapability: { resourceType: "storage.volume", action: "create" },
      supportedActions: [],
      columns: [{ id: "name", header: "Name", field: "name" }],
      filters: [],
      sortableFields: [],
      detailsSections: [
        {
          id: "overview",
          label: "Overview",
          fields: ["id", "name", "properties.sizeGb", "properties.attachedServerId"],
        },
      ],
      relationships: [
        {
          id: "attachedServer",
          targetResourceType: "compute.server",
          label: "Attached Server",
          sourcePropertyKey: "attachedServerId",
          direction: "to-one",
        },
      ],
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
  capabilities: [],
};

const volume: Resource = {
  id: "volume-00000001",
  name: "volume-one",
  resourceType: "storage.volume",
  projectId: "project-1",
  regionId: "eu-west",
  status: "ready",
  createdAt: "2024-01-01T00:00:00Z",
  updatedAt: "2024-01-01T00:01:00Z",
  properties: {
    sizeGb: 100,
    attachedServerId: "resource-0000000001",
  },
};

const relatedServer: Resource = {
  id: "resource-0000000001",
  name: "server-one",
  resourceType: "compute.server",
  projectId: "project-1",
  regionId: "eu-west",
  status: "ready",
  createdAt: "2024-01-01T00:00:00Z",
  updatedAt: "2024-01-01T00:01:00Z",
};

const operationsCollection: PaginatedCollection<Operation> = {
  items: [
    {
      id: "op-volume-1",
      action: "create",
      state: "succeeded",
      resourceId: volume.id,
      resourceType: volume.resourceType,
      projectId: "project-1",
      regionId: "eu-west",
      initiatedBy: "user-1",
      startedAt: "2024-01-01T00:00:00Z",
      updatedAt: "2024-01-01T00:01:00Z",
      correlationId: "corr-volume-1",
      error: null,
      events: [
        {
          id: "ev-volume-1",
          state: "succeeded",
          occurredAt: "2024-01-01T00:01:00Z",
          message: "Operation completed",
          correlationId: "corr-volume-1",
        },
      ],
    },
  ],
  total: 1,
  page: 0,
  pageSize: 25,
  hasMore: false,
};

function TestWrapper({ children }: { children: React.ReactNode }) {
  const client: ArafClient = {
    healthz: vi.fn(),
    getContext: vi.fn().mockResolvedValue(sessionContext),
    listServices: vi.fn().mockResolvedValue([volumeDescriptor]),
    listResources: vi.fn(),
    getResource: vi.fn().mockImplementation((resourceType: string, id: string) => {
      if (resourceType === "storage.volume" && id === "volume-00000001")
        return Promise.resolve(volume);
      if (resourceType === "compute.server" && id === "resource-0000000001") {
        return Promise.resolve(relatedServer);
      }
      return Promise.reject(new Error("Not found"));
    }),
    createResource: vi.fn(),
    submitAction: vi.fn(),
    listOperations: vi.fn().mockResolvedValue(operationsCollection),
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
  };

  return (
    <ResourceClientProvider client={client}>
      <OperationsClientProvider client={client}>
        <ScopeProvider scope={scope} onChange={noop}>
          <MemoryRouter initialEntries={["/resources/storage.volume/volume-00000001"]}>
            <Routes>
              <Route path="/resources/:resourceType/:id" element={children} />
            </Routes>
          </MemoryRouter>
        </ScopeProvider>
      </OperationsClientProvider>
    </ResourceClientProvider>
  );
}

describe("ResourceDetailPage", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("renders tabs and relationship panel links to related resource", async () => {
    render(
      <TestWrapper>
        <ResourceDetailPage resourceType="storage.volume" />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("tab", { name: "Overview" })).toBeInTheDocument();
    });

    expect(screen.getByRole("tab", { name: "Relationships" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Operations" })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("tab", { name: "Relationships" }));

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "server-one" })).toHaveAttribute(
        "href",
        "/resources/compute.server/resource-0000000001",
      );
    });
  });
});
