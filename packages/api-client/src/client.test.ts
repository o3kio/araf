import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createArafClient,
  ArafApiError,
  type ActionRequest,
  type HealthzResponse,
  type Operation,
  type PaginatedCollection,
  type Resource,
} from "./index.js";

describe("createArafClient", () => {
  const baseUrl = "http://localhost:9999";
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    globalThis.fetch = vi.fn();
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  function mockFetch(response: Response): void {
    vi.mocked(globalThis.fetch).mockResolvedValue(response);
  }

  function lastCall(): { input: URL; init: RequestInit | undefined } {
    const calls = vi.mocked(globalThis.fetch).mock.calls;
    expect(calls.length).toBeGreaterThan(0);
    const lastCall = calls[calls.length - 1];
    if (!lastCall) {
      throw new Error("no fetch call recorded");
    }
    const [input, init] = lastCall;
    return { input: input as URL, init };
  }

  it("calls healthz and returns the parsed response", async () => {
    const body: HealthzResponse = { status: "ok", service: "tenant-bff" };
    mockFetch(new Response(JSON.stringify(body), { status: 200 }));

    const client = createArafClient(baseUrl);
    const result = await client.healthz();

    expect(result).toEqual(body);
    const { input, init } = lastCall();
    expect(input.href).toBe(`${baseUrl}/healthz`);
    const headers = new Headers(init?.headers);
    expect(headers.get("x-request-id")).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu,
    );
    expect(headers.get("x-correlation-id")).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu,
    );
  });

  it("serializes listResources query parameters in camelCase", async () => {
    const body: PaginatedCollection<Resource> = {
      items: [],
      total: 100_000,
      page: 2,
      pageSize: 50,
      hasMore: false,
    };
    mockFetch(new Response(JSON.stringify(body), { status: 200 }));

    const client = createArafClient(baseUrl);
    await client.listResources("compute.server", {
      page: 2,
      pageSize: 50,
      projectId: "project-fixture",
      regionId: "global",
    });

    const { input } = lastCall();
    expect(input.pathname).toBe("/api/v1/resources/compute.server");
    expect(input.searchParams.get("page")).toBe("2");
    expect(input.searchParams.get("pageSize")).toBe("50");
    expect(input.searchParams.get("projectId")).toBe("project-fixture");
    expect(input.searchParams.get("regionId")).toBe("global");
  });

  it("omits undefined query parameters", async () => {
    const body: PaginatedCollection<Resource> = {
      items: [],
      total: 100_000,
      page: 0,
      pageSize: 25,
      hasMore: true,
    };
    mockFetch(new Response(JSON.stringify(body), { status: 200 }));

    const client = createArafClient(baseUrl);
    await client.listResources("compute.server", { page: 0 });

    const { input } = lastCall();
    expect(input.searchParams.get("page")).toBe("0");
    expect(input.searchParams.has("pageSize")).toBe(false);
    expect(input.searchParams.has("projectId")).toBe(false);
    expect(input.searchParams.has("regionId")).toBe(false);
  });

  it("encodes resource type and id in action URLs", async () => {
    mockFetch(
      new Response(
        JSON.stringify({
          id: "op-1",
          action: "start",
          state: "pending",
          resourceId: "resource-0000000001",
          resourceType: "compute.server",
          projectId: null,
          regionId: null,
          initiatedBy: null,
          startedAt: null,
          updatedAt: null,
          correlationId: "corr-1",
          error: null,
        }),
        { status: 200 },
      ),
    );

    const client = createArafClient(baseUrl);
    const requestBody: ActionRequest = {
      actionId: "start",
      payload: { force: true },
    };
    await client.submitAction("compute.server", "resource-0000000001", requestBody);

    const { input, init } = lastCall();
    expect(init?.method).toBe("POST");
    expect(input.href).toBe(
      `${baseUrl}/api/v1/resources/compute.server/resource-0000000001/actions`,
    );
    expect(JSON.parse(init?.body as string)).toEqual(requestBody);
  });

  it("throws ArafApiError with parsed ProblemDetails on HTTP error", async () => {
    const problem = {
      type: "https://araf.o3k.io/errors/404",
      title: "Not found",
      status: 404,
      detail: "resource type not found",
      correlationId: "corr-error-1",
      operationId: null,
      resourceId: null,
    };
    mockFetch(
      new Response(JSON.stringify(problem), {
        status: 404,
        headers: { "content-type": "application/json" },
      }),
    );

    const client = createArafClient(baseUrl);

    try {
      await client.getResource("foo", "bar");
      expect.fail("expected an error");
    } catch (error) {
      expect(error).toBeInstanceOf(ArafApiError);
      const apiError = error as ArafApiError;
      expect(apiError.status).toBe(404);
      expect(apiError.correlationId).toBe("corr-error-1");
      expect(apiError.problem).toEqual(problem);
    }
  });

  it.each(["retryable", "unknownOutcome"] as const)(
    "accepts operation state %s from the BFF",
    async (state) => {
      const body: Operation = {
        id: "op-retryable-1",
        action: "start",
        state,
        resourceId: "resource-1",
        resourceType: "compute.server",
        projectId: "project-1",
        regionId: "global",
        initiatedBy: "user-1",
        startedAt: "2024-01-01T00:00:00Z",
        updatedAt: "2024-01-01T00:00:05Z",
        correlationId: "corr-1",
        error: {
          code: "upstream-error",
          title: "Upstream operation error",
          detail: "transient provider failure",
        },
        events: [],
      };
      mockFetch(new Response(JSON.stringify(body), { status: 200 }));

      const client = createArafClient(baseUrl);
      const result = await client.getOperation("op-retryable-1");

      expect(result.state).toBe(state);
    },
  );

  it("falls back to a synthetic ProblemDetails when the error body is not JSON", async () => {
    mockFetch(new Response("bad gateway", { status: 502 }));

    const client = createArafClient(baseUrl);

    try {
      await client.healthz();
      expect.fail("expected an error");
    } catch (error) {
      const apiError = error as ArafApiError;
      expect(apiError.status).toBe(502);
      expect(apiError.problem.status).toBe(502);
      expect(apiError.problem.title).toBe("Unexpected error");
    }
  });
});
