/**
 * Typed frontend API client for the Araf BFF.
 *
 * Mirrors the JSON shapes exposed by `console-bff-core` and provides a thin
 * wrapper around `fetch` with request/correlation ID headers and structured
 * Problem Details error handling.
 */

export type ResourceStatus = "ready" | "busy" | "error" | "unknown";

export type OperationState =
  "pending" | "running" | "succeeded" | "failed" | "retryable" | "unknownOutcome";

export type SortDirection = "asc" | "desc";

export interface HealthzResponse {
  status: "ok";
  service: string;
}

export interface Capability {
  resourceType: string;
  action: string;
}

export interface SessionContext {
  surface: string;
  userId: string;
  userName: string;
  organizationId: string | null;
  projectId: string | null;
  regionId: string | null;
  capabilities: Capability[];
}

export type ActionRiskClass = "normal" | "disruptive" | "destructive" | "privileged";

export interface ActionDescriptor {
  id: string;
  name: string;
  requiresConfirmation: boolean;
  riskClass: ActionRiskClass;
  requiredCapability: Capability;
  inputSchema?: unknown;
}

export type FilterKind = "text" | "select";

export interface ColumnDescriptor {
  id: string;
  header: string;
  field: string;
  width?: string;
}

export interface FilterDescriptor {
  id: string;
  label: string;
  field: string;
  kind: FilterKind;
}

export interface DetailsSectionDescriptor {
  id: string;
  label: string;
  fields: string[];
}

export type RelationshipDirection = "to-one" | "to-many";

export interface RelationshipDescriptor {
  id: string;
  targetResourceType: string;
  label: string;
  sourcePropertyKey: string;
  direction: RelationshipDirection;
}

export interface ResourceTypeDescriptor {
  id: string;
  name: string;
  pluralName: string;
  iconToken: string;
  supportedActions: ActionDescriptor[];
  columns: ColumnDescriptor[];
  filters: FilterDescriptor[];
  sortableFields: string[];
  detailsSections: DetailsSectionDescriptor[];
  relationships: RelationshipDescriptor[];
  createSchema?: unknown;
  createCapability: Capability;
}

export interface CreateResourceRequest {
  resourceType: string;
  payload: unknown;
}

export interface ServiceDescriptor {
  id: string;
  name: string;
  category: string;
  resourceTypes: ResourceTypeDescriptor[];
}

export interface Resource {
  id: string;
  name: string;
  resourceType: string;
  projectId: string;
  regionId: string;
  status: ResourceStatus;
  createdAt: string;
  updatedAt: string;
  properties?: Record<string, unknown>;
}

export interface PaginatedCollection<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export interface OperationError {
  code: string;
  title: string;
  detail: string;
}

export interface OperationEvent {
  id: string;
  state: OperationState;
  occurredAt: string;
  message: string;
  correlationId: string;
}

export interface Operation {
  id: string;
  action: string;
  state: OperationState;
  resourceId: string | null;
  resourceType: string | null;
  projectId: string | null;
  regionId: string | null;
  initiatedBy: string | null;
  startedAt: string | null;
  updatedAt: string | null;
  correlationId: string;
  error: OperationError | null;
  events: OperationEvent[];
}

export interface ActionRequest {
  actionId: string;
  payload?: unknown;
}

export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail: string;
  correlationId: string;
  operationId: string | null;
  resourceId: string | null;
}

export interface ListResourcesQuery {
  page?: number;
  pageSize?: number;
  projectId?: string;
  regionId?: string;
  sortField?: string;
  sortDirection?: SortDirection;
  [filterKey: string]: string | number | undefined;
}

export interface ListOperationsQuery {
  page?: number;
  pageSize?: number;
  state?: OperationState;
  action?: string;
  resourceType?: string;
  resourceId?: string;
  projectId?: string;
  regionId?: string;
  since?: string;
  until?: string;
}

// Governance types

export interface Project {
  id: string;
  name: string;
  organizationId: string;
  status: string;
  createdAt: string;
  updatedAt: string;
}

export interface User {
  id: string;
  name: string;
  email?: string;
  status: string;
  createdAt: string;
}

export interface Role {
  id: string;
  name: string;
  description?: string;
}

export interface ProjectMember {
  user: User;
  roles: string[];
}

export interface QuotaEntry {
  resourceType: string;
  limit: number;
  used: number;
  unit: string;
}

export interface ProjectQuota {
  projectId: string;
  entries: QuotaEntry[];
}

export interface AuditEvent {
  id: string;
  actor: string;
  action: string;
  resourceType: string | null;
  resourceId: string | null;
  projectId: string | null;
  outcome: string;
  recordedAt: string;
  correlationId: string;
}

export interface ApiCredential {
  id: string;
  name: string;
  kind: string;
  projectId: string;
  createdAt: string;
  expiresAt: string | null;
  secret?: string;
}

export interface CreateApiCredentialRequest {
  name: string;
  kind: string;
  projectId: string;
  expiresAt?: string | null;
}

export interface ListProjectsQuery {
  page?: number;
  pageSize?: number;
}

export interface ListUsersQuery {
  page?: number;
  pageSize?: number;
}

export interface ListRolesQuery {
  page?: number;
  pageSize?: number;
}

export interface ListQuotasQuery {
  page?: number;
  pageSize?: number;
  projectId?: string;
}

export interface ListAuditEventsQuery {
  page?: number;
  pageSize?: number;
  projectId?: string;
  action?: string;
  actor?: string;
  since?: string;
  until?: string;
}

export interface ListApiCredentialsQuery {
  page?: number;
  pageSize?: number;
}

export class ArafApiError extends Error {
  readonly status: number;
  readonly problem: ProblemDetails;
  readonly correlationId: string;

  constructor(status: number, problem: ProblemDetails) {
    super(`${problem.title} (${String(status)}): ${problem.detail}`);
    this.status = status;
    this.problem = problem;
    this.correlationId = problem.correlationId;
  }
}

function generateId(): string {
  if (
    typeof globalThis.crypto !== "undefined" &&
    typeof globalThis.crypto.randomUUID === "function"
  ) {
    return globalThis.crypto.randomUUID();
  }

  // Fallback for environments without `crypto.randomUUID`.
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/gu, (char) => {
    const random = (Math.random() * 16) | 0;
    const value = char === "x" ? random : (random & 0x3) | 0x8;
    return value.toString(16);
  });
}

function appendSearchParams(url: URL, params: Record<string, string | number | undefined>): void {
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined) {
      url.searchParams.set(key, String(value));
    }
  }
}

async function parseJson<T>(response: Response): Promise<T> {
  return (await response.json()) as T;
}

function buildProblemDetails(status: number, requestCorrelationId: string): ProblemDetails {
  return {
    type: `https://araf.o3k.io/errors/${String(status)}`,
    title: "Unexpected error",
    status,
    detail: "The server returned an error response that could not be parsed.",
    correlationId: requestCorrelationId,
    operationId: null,
    resourceId: null,
  };
}

async function handleError(response: Response, requestCorrelationId: string): Promise<never> {
  const text = await response.text();
  let problem: ProblemDetails;
  try {
    problem = JSON.parse(text) as ProblemDetails;
  } catch {
    problem = buildProblemDetails(response.status, requestCorrelationId);
  }
  throw new ArafApiError(response.status, problem);
}

export interface ArafClient {
  healthz(): Promise<HealthzResponse>;
  getContext(): Promise<SessionContext>;
  listServices(): Promise<ServiceDescriptor[]>;
  listResources(
    resourceType: string,
    query?: ListResourcesQuery,
  ): Promise<PaginatedCollection<Resource>>;
  getResource(resourceType: string, id: string): Promise<Resource>;
  createResource(resourceType: string, payload: unknown): Promise<Operation>;
  submitAction(resourceType: string, id: string, actionRequest: ActionRequest): Promise<Operation>;
  listOperations(query?: ListOperationsQuery): Promise<PaginatedCollection<Operation>>;
  getOperation(id: string): Promise<Operation>;

  // Governance
  listProjects(query?: ListProjectsQuery): Promise<PaginatedCollection<Project>>;
  getProject(id: string): Promise<Project>;
  listProjectMembers(id: string): Promise<ProjectMember[]>;
  listUsers(query?: ListUsersQuery): Promise<PaginatedCollection<User>>;
  getUser(id: string): Promise<User>;
  listRoles(query?: ListRolesQuery): Promise<PaginatedCollection<Role>>;
  listQuotas(query?: ListQuotasQuery): Promise<PaginatedCollection<ProjectQuota>>;
  listAuditEvents(query?: ListAuditEventsQuery): Promise<PaginatedCollection<AuditEvent>>;
  listApiCredentials(query?: ListApiCredentialsQuery): Promise<PaginatedCollection<ApiCredential>>;
  createApiCredential(payload: CreateApiCredentialRequest): Promise<ApiCredential>;
  deleteApiCredential(id: string): Promise<void>;
}

export function createArafClient(baseUrl: string | URL): ArafClient {
  const resolvedBaseUrl = typeof baseUrl === "string" ? baseUrl : baseUrl.toString();

  async function request<T>(
    path: string,
    options: {
      method?: string;
      body?: string;
      query?: Record<string, string | number | undefined>;
    } = {},
  ): Promise<T> {
    const url = new URL(path, resolvedBaseUrl);
    if (options.query) {
      appendSearchParams(url, options.query);
    }

    const requestId = generateId();
    const correlationId = generateId();

    const response = await fetch(url, {
      method: options.method ?? "GET",
      credentials: "include",
      headers: {
        Accept: "application/json",
        "Content-Type": "application/json",
        "x-request-id": requestId,
        "x-correlation-id": correlationId,
      },
      body: options.body,
    });

    if (!response.ok) {
      await handleError(response, correlationId);
    }

    return parseJson<T>(response);
  }

  return {
    healthz: () => request<HealthzResponse>("/healthz"),

    getContext: () => request<SessionContext>("/api/v1/context"),

    listServices: () => request<ServiceDescriptor[]>("/api/v1/services"),

    listResources: (resourceType, query) => {
      const { sortField, sortDirection, page, pageSize, projectId, regionId, ...filters } =
        query ?? {};
      return request<PaginatedCollection<Resource>>(
        `/api/v1/resources/${encodeURIComponent(resourceType)}`,
        {
          query: {
            page,
            pageSize,
            projectId,
            regionId,
            sortField,
            sortDirection,
            ...filters,
          },
        },
      );
    },

    getResource: (resourceType, id) =>
      request<Resource>(
        `/api/v1/resources/${encodeURIComponent(resourceType)}/${encodeURIComponent(id)}`,
      ),

    createResource: (resourceType, payload) =>
      request<Operation>(`/api/v1/resources/${encodeURIComponent(resourceType)}`, {
        method: "POST",
        body: JSON.stringify(payload),
      }),

    submitAction: (resourceType, id, actionRequest) =>
      request<Operation>(
        `/api/v1/resources/${encodeURIComponent(resourceType)}/${encodeURIComponent(id)}/actions`,
        {
          method: "POST",
          body: JSON.stringify(actionRequest),
        },
      ),

    listOperations: (query) =>
      request<PaginatedCollection<Operation>>("/api/v1/operations", {
        query: {
          page: query?.page,
          pageSize: query?.pageSize,
          state: query?.state,
          action: query?.action,
          resourceType: query?.resourceType,
          resourceId: query?.resourceId,
          projectId: query?.projectId,
          regionId: query?.regionId,
          since: query?.since,
          until: query?.until,
        },
      }),

    getOperation: (id) => request<Operation>(`/api/v1/operations/${encodeURIComponent(id)}`),

    // Governance
    listProjects: (query) =>
      request<PaginatedCollection<Project>>("/api/v1/governance/projects", {
        query: { page: query?.page, pageSize: query?.pageSize },
      }),

    getProject: (id) => request<Project>(`/api/v1/governance/projects/${encodeURIComponent(id)}`),

    listProjectMembers: (id) =>
      request<ProjectMember[]>(`/api/v1/governance/projects/${encodeURIComponent(id)}/members`),

    listUsers: (query) =>
      request<PaginatedCollection<User>>("/api/v1/governance/users", {
        query: { page: query?.page, pageSize: query?.pageSize },
      }),

    getUser: (id) => request<User>(`/api/v1/governance/users/${encodeURIComponent(id)}`),

    listRoles: (query) =>
      request<PaginatedCollection<Role>>("/api/v1/governance/roles", {
        query: { page: query?.page, pageSize: query?.pageSize },
      }),

    listQuotas: (query) =>
      request<PaginatedCollection<ProjectQuota>>("/api/v1/governance/quotas", {
        query: { page: query?.page, pageSize: query?.pageSize, projectId: query?.projectId },
      }),

    listAuditEvents: (query) =>
      request<PaginatedCollection<AuditEvent>>("/api/v1/governance/audit", {
        query: {
          page: query?.page,
          pageSize: query?.pageSize,
          projectId: query?.projectId,
          action: query?.action,
          actor: query?.actor,
          since: query?.since,
          until: query?.until,
        },
      }),

    listApiCredentials: (query) =>
      request<PaginatedCollection<ApiCredential>>("/api/v1/governance/api-credentials", {
        query: { page: query?.page, pageSize: query?.pageSize },
      }),

    createApiCredential: (payload) =>
      request<ApiCredential>("/api/v1/governance/api-credentials", {
        method: "POST",
        body: JSON.stringify(payload),
      }),

    deleteApiCredential: async (id) => {
      await request<unknown>(`/api/v1/governance/api-credentials/${encodeURIComponent(id)}`, {
        method: "DELETE",
      });
    },
  };
}
