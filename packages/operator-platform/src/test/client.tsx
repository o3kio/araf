import type { ArafClient, SessionContext } from "@araf/api-client";
import { vi } from "vitest";
import type { ReactNode } from "react";
import { MemoryRouter } from "react-router";
import { OperatorPlatformClientProvider } from "../client/context";

export const allOperatorPlatformCapabilities = [
  { resourceType: "platform.overview", action: "read" },
  { resourceType: "platform.region", action: "list" },
  { resourceType: "platform.region", action: "read" },
  { resourceType: "platform.health", action: "read" },
  { resourceType: "platform.capacity", action: "read" },
  { resourceType: "operator.account", action: "list" },
  { resourceType: "operator.project", action: "list" },
  { resourceType: "operator.operation", action: "list" },
  { resourceType: "operator.audit", action: "read" },
];

export const sessionContext: SessionContext = {
  surface: "operator-bff",
  userId: "operator-1",
  userName: "Test Operator",
  organizationId: null,
  projectId: null,
  regionId: "global",
  capabilities: allOperatorPlatformCapabilities,
};

const emptyCollection = { items: [], total: 0, page: 0, pageSize: 25, hasMore: false };

export function createMockClient(overrides: Partial<ArafClient> = {}): ArafClient {
  return {
    healthz: vi.fn(),
    getContext: vi.fn().mockResolvedValue(sessionContext),
    listServices: vi.fn(),
    listResources: vi.fn(),
    getResource: vi.fn(),
    createResource: vi.fn(),
    submitAction: vi.fn(),
    listOperations: vi.fn(),
    getOperation: vi.fn(),
    listProjects: vi.fn().mockResolvedValue(emptyCollection),
    getProject: vi.fn().mockRejectedValue(new Error("Not found")),
    listProjectMembers: vi.fn().mockResolvedValue([]),
    listUsers: vi.fn().mockResolvedValue(emptyCollection),
    getUser: vi.fn().mockRejectedValue(new Error("Not found")),
    listRoles: vi.fn().mockResolvedValue(emptyCollection),
    listQuotas: vi.fn().mockResolvedValue(emptyCollection),
    listAuditEvents: vi.fn().mockResolvedValue(emptyCollection),
    listApiCredentials: vi.fn().mockResolvedValue(emptyCollection),
    createApiCredential: vi.fn(),
    deleteApiCredential: vi.fn().mockResolvedValue(undefined),
    getPlatformOverview: vi.fn(),
    listRegions: vi.fn(),
    listAvailabilityZones: vi.fn(),
    listProviderHealth: vi.fn(),
    listServiceHealth: vi.fn(),
    getCapacitySummary: vi.fn(),
    listCustomerAccounts: vi.fn().mockResolvedValue(emptyCollection),
    listAccountProjects: vi.fn().mockResolvedValue(emptyCollection),
    listOperatorOperations: vi.fn().mockResolvedValue(emptyCollection),
    listOperatorAuditEvents: vi.fn().mockResolvedValue(emptyCollection),
    ...overrides,
  };
}

export function TestWrapper({
  children,
  client,
  initialEntries,
}: {
  children: ReactNode;
  client: ArafClient;
  initialEntries?: string[];
}) {
  return (
    <OperatorPlatformClientProvider client={client}>
      <MemoryRouter initialEntries={initialEntries}>{children}</MemoryRouter>
    </OperatorPlatformClientProvider>
  );
}
