import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router";
import { ResourceClientProvider } from "../client/context";
import { ResourceActionsPanel } from "./ResourceActionsPanel";
import type {
  ArafClient,
  Operation,
  Resource,
  SessionContext,
  ActionRequest,
} from "@araf/api-client";
import type { ResourceDescriptor } from "../descriptor";

const sessionContext: SessionContext = {
  surface: "tenant",
  userId: "user-1",
  userName: "Test User",
  organizationId: "org-1",
  projectId: "project-1",
  regionId: "eu-west",
  capabilities: [
    { resourceType: "compute.server", action: "start" },
    { resourceType: "compute.server", action: "stop" },
    { resourceType: "compute.server", action: "delete" },
    { resourceType: "storage.volume", action: "attach" },
  ],
};

const server: Resource = {
  id: "resource-0000000001",
  name: "server-one",
  resourceType: "compute.server",
  projectId: "project-1",
  regionId: "eu-west",
  status: "ready",
  createdAt: "2024-01-01T00:00:00Z",
  updatedAt: "2024-01-01T00:01:00Z",
};

const serverDescriptor: ResourceDescriptor = {
  id: "compute.server",
  name: "Server",
  pluralName: "Servers",
  iconToken: "server",
  createCapability: { resourceType: "compute.server", action: "create" },
  supportedActions: [
    {
      id: "start",
      name: "Start",
      requiresConfirmation: false,
      riskClass: "normal",
      requiredCapability: { resourceType: "compute.server", action: "start" },
    },
    {
      id: "stop",
      name: "Stop",
      requiresConfirmation: true,
      riskClass: "disruptive",
      requiredCapability: { resourceType: "compute.server", action: "stop" },
    },
    {
      id: "delete",
      name: "Delete",
      requiresConfirmation: true,
      riskClass: "destructive",
      requiredCapability: { resourceType: "compute.server", action: "delete" },
    },
  ],
  columns: [],
  filters: [],
  sortableFields: [],
  detailsSections: [],
  relationships: [],
};

const volumeDescriptor: ResourceDescriptor = {
  id: "storage.volume",
  name: "Volume",
  pluralName: "Volumes",
  iconToken: "storage",
  createCapability: { resourceType: "storage.volume", action: "create" },
  supportedActions: [
    {
      id: "attach",
      name: "Attach",
      requiresConfirmation: false,
      riskClass: "normal",
      requiredCapability: { resourceType: "storage.volume", action: "attach" },
      inputSchema: {
        type: "object",
        required: ["serverId"],
        properties: {
          serverId: { type: "string", minLength: 1, title: "Server ID" },
        },
      },
    },
    {
      id: "detach",
      name: "Detach",
      requiresConfirmation: true,
      riskClass: "disruptive",
      requiredCapability: { resourceType: "storage.volume", action: "detach" },
    },
  ],
  columns: [],
  filters: [],
  sortableFields: [],
  detailsSections: [],
  relationships: [],
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
};

const startOperation: Operation = {
  id: "op-start-1",
  action: "start",
  state: "pending",
  resourceId: server.id,
  resourceType: server.resourceType,
  projectId: "project-1",
  regionId: "eu-west",
  initiatedBy: "user-1",
  startedAt: "2024-01-01T00:00:00Z",
  updatedAt: "2024-01-01T00:00:00Z",
  correlationId: "corr-start-1",
  error: null,
  events: [
    {
      id: "ev-start-1",
      state: "pending",
      occurredAt: "2024-01-01T00:00:00Z",
      message: "Operation created and pending",
      correlationId: "corr-start-1",
    },
  ],
};

const attachOperation: Operation = {
  id: "op-attach-1",
  action: "attach",
  state: "pending",
  resourceId: volume.id,
  resourceType: volume.resourceType,
  projectId: "project-1",
  regionId: "eu-west",
  initiatedBy: "user-1",
  startedAt: "2024-01-01T00:00:00Z",
  updatedAt: "2024-01-01T00:00:00Z",
  correlationId: "corr-attach-1",
  error: null,
  events: [
    {
      id: "ev-attach-1",
      state: "pending",
      occurredAt: "2024-01-01T00:00:00Z",
      message: "Operation created and pending",
      correlationId: "corr-attach-1",
    },
  ],
};

function TestWrapper({
  resource,
  descriptor,
  onOperation,
}: {
  resource: Resource;
  descriptor: ResourceDescriptor;
  onOperation?: (operation: Operation) => void;
}) {
  const client: ArafClient = {
    healthz: vi.fn(),
    getContext: vi.fn().mockResolvedValue(sessionContext),
    listServices: vi.fn(),
    listResources: vi.fn(),
    getResource: vi.fn(),
    createResource: vi.fn(),
    submitAction: vi
      .fn()
      .mockImplementation((_resourceType: string, _id: string, request: ActionRequest) => {
        const actionId = request.actionId;
        if (actionId === "start") return Promise.resolve(startOperation);
        if (actionId === "attach") return Promise.resolve(attachOperation);
        return Promise.reject(new Error("unexpected action"));
      }),
    listOperations: vi.fn(),
    getOperation: vi.fn(),
  };

  return (
    <ResourceClientProvider client={client}>
      <MemoryRouter>
        <ResourceActionsPanel
          resource={resource}
          descriptor={descriptor}
          onOperation={onOperation}
        />
      </MemoryRouter>
    </ResourceClientProvider>
  );
}

describe("ResourceActionsPanel", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("renders actions the session is capable of and hides others", async () => {
    render(<TestWrapper resource={server} descriptor={serverDescriptor} />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Start" })).toBeInTheDocument();
    });

    expect(screen.getByRole("button", { name: "Stop" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Delete" })).toBeInTheDocument();
  });

  it("hides actions the tenant lacks capability for", async () => {
    const limitedDescriptor: ResourceDescriptor = {
      ...serverDescriptor,
      supportedActions: [
        {
          id: "reboot",
          name: "Reboot",
          requiresConfirmation: true,
          riskClass: "disruptive",
          requiredCapability: { resourceType: "compute.server", action: "reboot" },
        },
      ],
    };

    render(<TestWrapper resource={server} descriptor={limitedDescriptor} />);

    await waitFor(() => {
      expect(screen.queryByRole("button", { name: "Reboot" })).not.toBeInTheDocument();
    });
  });

  it("submits a normal action and surfaces the returned operation", async () => {
    const onOperation = vi.fn();
    render(
      <TestWrapper resource={server} descriptor={serverDescriptor} onOperation={onOperation} />,
    );

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Start" })).toBeInTheDocument();
    });

    await userEvent.click(screen.getByRole("button", { name: "Start" }));

    await waitFor(() => {
      expect(onOperation).toHaveBeenCalledWith(startOperation);
    });
  });

  it("opens a confirmation modal for destructive actions", async () => {
    render(<TestWrapper resource={server} descriptor={serverDescriptor} />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Delete" })).toBeInTheDocument();
    });

    await userEvent.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(screen.getByText(/Are you sure you want to delete server-one/i)).toBeInTheDocument();
    });
  });

  it("opens an input form modal for actions with inputSchema and validates it", async () => {
    render(<TestWrapper resource={volume} descriptor={volumeDescriptor} />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Attach" })).toBeInTheDocument();
    });

    await userEvent.click(screen.getByRole("button", { name: "Attach" }));

    await waitFor(() => {
      expect(screen.getByLabelText("Server ID")).toBeInTheDocument();
    });

    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));

    await waitFor(() => {
      expect(screen.getByText(/required property|serverId/i)).toBeInTheDocument();
    });

    await userEvent.type(screen.getByLabelText("Server ID"), "server-1");
    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));

    await waitFor(() => {
      expect(screen.getByText(/op-attach-1/i)).toBeInTheDocument();
    });
  });
});
