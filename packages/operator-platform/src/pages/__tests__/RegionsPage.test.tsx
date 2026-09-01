import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RegionsPage } from "../RegionsPage";
import { createMockClient, TestWrapper } from "../../test/client";

const mockRegions = [
  {
    id: "eu-west",
    name: "EU West",
    status: "healthy" as const,
    azs: [{ id: "eu-west-az-1", name: "AZ 1", regionId: "eu-west", status: "healthy" as const }],
    updatedAt: "2026-01-01T00:00:00Z",
  },
];

describe("RegionsPage", () => {
  it("renders the region list", async () => {
    const client = createMockClient({
      listRegions: vi.fn().mockResolvedValue(mockRegions),
    });

    render(
      <TestWrapper client={client}>
        <RegionsPage />
      </TestWrapper>,
    );

    expect(await screen.findByRole("heading", { name: "Regions" })).toBeInTheDocument();
    expect(screen.getByText("EU West")).toBeInTheDocument();
  });
});
