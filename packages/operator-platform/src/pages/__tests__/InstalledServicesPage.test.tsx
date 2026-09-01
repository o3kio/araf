import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { InstalledServicesPage } from "../InstalledServicesPage";
import { createMockClient, TestWrapper } from "../../test/client";
import type { InstalledService, DiscoveredResourceType } from "@araf/api-client";

const mockServices: InstalledService[] = [
  {
    id: "compute",
    namespace: "o3k.io",
    name: "Compute",
    version: "1.0.0",
    ownership: null,
    lifecycleState: "stable",
    health: "healthy",
    resourceTypes: ["compute.server"],
    installedAt: "2026-01-01T00:00:00Z",
  },
];

const mockResourceTypes: DiscoveredResourceType[] = [
  {
    namespace: "o3k.io",
    name: "server",
    serviceId: "compute",
    schemaVersion: "v1",
    collection: "servers",
    scope: "project",
    ready: true,
    lifecycleActions: {},
  },
];

describe("InstalledServicesPage", () => {
  it("renders installed services and discovered resource types", async () => {
    const client = createMockClient({
      listInstalledServices: vi.fn().mockResolvedValue(mockServices),
      listDiscoveredResourceTypes: vi.fn().mockResolvedValue(mockResourceTypes),
    });

    render(
      <TestWrapper client={client}>
        <InstalledServicesPage />
      </TestWrapper>,
    );

    expect(await screen.findByRole("heading", { name: "Installed services" })).toBeInTheDocument();
    expect(screen.getByText("Compute")).toBeInTheDocument();
    expect(screen.getByText("healthy")).toBeInTheDocument();
    expect(screen.getByText("server")).toBeInTheDocument();
  });

  it("displays error state when loading fails", async () => {
    const client = createMockClient({
      listInstalledServices: vi.fn().mockRejectedValue(new Error("Upstream error")),
      listDiscoveredResourceTypes: vi.fn().mockResolvedValue([]),
    });

    render(
      <TestWrapper client={client}>
        <InstalledServicesPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Could not load services")).toBeInTheDocument();
    });

    expect(screen.getByText("Upstream error")).toBeInTheDocument();
  });
});
