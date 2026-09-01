import type { ArafClient, SessionContext } from "@araf/api-client";
import { vi } from "vitest";
import { ResourceClientProvider } from "@araf/resources";
import { GovernanceClientProvider } from "../client/context";
import type { ReactNode } from "react";
import { MemoryRouter } from "react-router";

export const allGovernanceCapabilities = [
  { resourceType: "tenant.project", action: "list" },
  { resourceType: "tenant.project", action: "read" },
  { resourceType: "tenant.user", action: "list" },
  { resourceType: "tenant.user", action: "read" },
  { resourceType: "tenant.role", action: "list" },
  { resourceType: "tenant.quota", action: "read" },
  { resourceType: "tenant.audit", action: "read" },
  { resourceType: "tenant.api-credential", action: "list" },
  { resourceType: "tenant.api-credential", action: "create" },
  { resourceType: "tenant.api-credential", action: "delete" },
];

export const sessionContext: SessionContext = {
  surface: "tenant-bff",
  userId: "user-1",
  userName: "Test User",
  organizationId: "org-1",
  projectId: "project-1",
  regionId: "eu-west",
  capabilities: allGovernanceCapabilities,
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
    listServiceCatalog: vi.fn(),
    listUsage: vi.fn(),
    listInstalledServices: vi.fn(),
    listDiscoveredResourceTypes: vi.fn(),
    getCapacitySummary: vi.fn(),
    listCustomerAccounts: vi.fn(),
    listAccountProjects: vi.fn(),
    listOperatorOperations: vi.fn(),
    listOperatorAuditEvents: vi.fn(),
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
    <ResourceClientProvider client={client}>
      <GovernanceClientProvider client={client}>
        <MemoryRouter initialEntries={initialEntries}>{children}</MemoryRouter>
      </GovernanceClientProvider>
    </ResourceClientProvider>
  );
}
