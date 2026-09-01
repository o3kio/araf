import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PlatformOverviewPage } from "../PlatformOverviewPage";
import { createMockClient, TestWrapper } from "../../test/client";

const mockOverview = {
  regionStatusSummary: [
    { status: "healthy" as const, count: 3 },
    { status: "degraded" as const, count: 1 },
  ],
  providerStatusSummary: [{ status: "healthy" as const, count: 10 }],
  activeOperationsCount: 5,
  recentAlerts: [
    {
      id: "alert-1",
      severity: "warning" as const,
      message: "Test alert",
      occurredAt: "2026-01-01T00:00:00Z",
    },
  ],
  dataFreshnessAt: "2026-01-01T00:00:00Z",
};

describe("PlatformOverviewPage", () => {
  it("renders platform overview data", async () => {
    const client = createMockClient({
      getPlatformOverview: vi.fn().mockResolvedValue(mockOverview),
    });

    render(
      <TestWrapper client={client}>
        <PlatformOverviewPage />
      </TestWrapper>,
    );

    expect(await screen.findByRole("heading", { name: "Platform overview" })).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
    expect(screen.getByText("Test alert")).toBeInTheDocument();
  });

  it("renders an error state when loading fails", async () => {
    const client = createMockClient({
      getPlatformOverview: vi.fn().mockRejectedValue(new Error("Failed to load")),
    });

    render(
      <TestWrapper client={client}>
        <PlatformOverviewPage />
      </TestWrapper>,
    );

    expect(await screen.findByText("Could not load platform overview")).toBeInTheDocument();
  });
});
