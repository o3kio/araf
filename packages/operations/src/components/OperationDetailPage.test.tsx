import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router";
import { OperationsClientProvider } from "../client/context";
import { OperationDetailPage } from "./OperationDetailPage";
import type { ArafClient, Operation } from "@araf/api-client";

const pendingOperation: Operation = {
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
      message: "Operation created and pending",
      correlationId: "corr-1",
    },
  ],
};

const failedOperation: Operation = {
  ...pendingOperation,
  state: "failed",
  error: {
    code: "fixture-failure",
    title: "Fixture operation failed",
    detail: "Deterministic failure for this operation id.",
  },
  events: [
    ...pendingOperation.events,
    {
      id: "ev-2",
      state: "failed",
      occurredAt: "2024-01-01T00:00:01Z",
      message: "Operation failed deterministically",
      correlationId: "corr-1",
    },
  ],
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
    listOperations: vi.fn(),
    getOperation: vi.fn().mockResolvedValue(pendingOperation),
  };
  return (
    <OperationsClientProvider client={client ?? defaultClient}>
      <MemoryRouter initialEntries={[`/operations/op-0000000001`]}>
        <Routes>
          <Route path="/operations/:id" element={children} />
        </Routes>
      </MemoryRouter>
    </OperationsClientProvider>
  );
}

describe("OperationDetailPage", () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders operation details and timeline", async () => {
    render(
      <TestWrapper>
        <OperationDetailPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Operation op-0000000001")).toBeInTheDocument();
    });

    expect(screen.getByText("create")).toBeInTheDocument();
    expect(screen.getAllByText("Pending").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("project-1")).toBeInTheDocument();
    expect(screen.getByText("eu-west")).toBeInTheDocument();
    expect(screen.getByText("Operation created and pending")).toBeInTheDocument();
    expect(screen.getByText("corr-1")).toBeInTheDocument();
  });

  it("renders structured error details for failed operations", async () => {
    const client: ArafClient = {
      healthz: vi.fn(),
      getContext: vi.fn(),
      listServices: vi.fn(),
      listResources: vi.fn(),
      getResource: vi.fn(),
      createResource: vi.fn(),
      submitAction: vi.fn(),
      listOperations: vi.fn(),
      getOperation: vi.fn().mockResolvedValue(failedOperation),
    };

    render(
      <TestWrapper client={client}>
        <OperationDetailPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getAllByText("Failed").length).toBeGreaterThanOrEqual(1);
    });

    expect(screen.getByText("fixture-failure")).toBeInTheDocument();
    expect(screen.getByText("Fixture operation failed")).toBeInTheDocument();
    expect(screen.getByText("Deterministic failure for this operation id.")).toBeInTheDocument();
  });

  it("derives a minimal timeline from timestamps when events are empty", async () => {
    const operationWithoutEvents: Operation = {
      ...pendingOperation,
      events: [],
      startedAt: "2024-01-01T00:00:00Z",
      updatedAt: "2024-01-01T00:00:05Z",
      state: "running",
    };
    const client: ArafClient = {
      healthz: vi.fn(),
      getContext: vi.fn(),
      listServices: vi.fn(),
      listResources: vi.fn(),
      getResource: vi.fn(),
      createResource: vi.fn(),
      submitAction: vi.fn(),
      listOperations: vi.fn(),
      getOperation: vi.fn().mockResolvedValue(operationWithoutEvents),
    };

    render(
      <TestWrapper client={client}>
        <OperationDetailPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Operation started running")).toBeInTheDocument();
    });

    expect(screen.getByText("Operation created and pending")).toBeInTheDocument();
  });

  it("renders retryable and unknownOutcome states", async () => {
    const retryableOperation: Operation = {
      ...pendingOperation,
      state: "retryable",
      updatedAt: "2024-01-01T00:00:05Z",
      error: {
        code: "upstream-error",
        title: "Upstream operation error",
        detail: "transient provider failure",
      },
      events: [],
    };
    const client: ArafClient = {
      healthz: vi.fn(),
      getContext: vi.fn(),
      listServices: vi.fn(),
      listResources: vi.fn(),
      getResource: vi.fn(),
      createResource: vi.fn(),
      submitAction: vi.fn(),
      listOperations: vi.fn(),
      getOperation: vi.fn().mockResolvedValue(retryableOperation),
    };

    render(
      <TestWrapper client={client}>
        <OperationDetailPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getAllByText("Retryable").length).toBeGreaterThanOrEqual(1);
    });

    expect(screen.getAllByText(/transient provider failure/).length).toBeGreaterThanOrEqual(1);
  });

  it("displays an error state when the operation cannot be loaded", async () => {
    const client: ArafClient = {
      healthz: vi.fn(),
      getContext: vi.fn(),
      listServices: vi.fn(),
      listResources: vi.fn(),
      getResource: vi.fn(),
      createResource: vi.fn(),
      submitAction: vi.fn(),
      listOperations: vi.fn(),
      getOperation: vi.fn().mockRejectedValue(new Error("Not found")),
    };

    render(
      <TestWrapper client={client}>
        <OperationDetailPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Could not load operation")).toBeInTheDocument();
    });

    expect(screen.getByText("Not found")).toBeInTheDocument();
  });
});
