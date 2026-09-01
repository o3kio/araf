/**
 * Typed frontend API client for the Araf BFF.
 *
 * Mirrors the JSON shapes exposed by `console-bff-core` and provides a thin
 * wrapper around `fetch` with request/correlation ID headers and structured
 * Problem Details error handling.
 */

export type ResourceStatus = "ready" | "busy" | "error" | "unknown";

export type OperationState = "pending" | "running" | "succeeded" | "failed";

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

export interface ActionDescriptor {
  id: string;
  name: string;
  requiresConfirmation: boolean;
}

export interface ResourceTypeDescriptor {
  id: string;
  name: string;
  pluralName: string;
  supportedActions: ActionDescriptor[];
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
}

export interface ListOperationsQuery {
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
  submitAction(resourceType: string, id: string, actionRequest: ActionRequest): Promise<Operation>;
  listOperations(query?: ListOperationsQuery): Promise<PaginatedCollection<Operation>>;
  getOperation(id: string): Promise<Operation>;
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

    listResources: (resourceType, query) =>
      request<PaginatedCollection<Resource>>(
        `/api/v1/resources/${encodeURIComponent(resourceType)}`,
        {
          query: {
            page: query?.page,
            pageSize: query?.pageSize,
            projectId: query?.projectId,
            regionId: query?.regionId,
          },
        },
      ),

    getResource: (resourceType, id) =>
      request<Resource>(
        `/api/v1/resources/${encodeURIComponent(resourceType)}/${encodeURIComponent(id)}`,
      ),

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
        },
      }),

    getOperation: (id) => request<Operation>(`/api/v1/operations/${encodeURIComponent(id)}`),
  };
}
