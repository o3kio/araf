import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AccountsPage } from "../AccountsPage";
import { createMockClient, TestWrapper } from "../../test/client";

const mockAccounts = {
  items: [
    {
      id: "account-001",
      name: "Customer One",
      status: "active",
      createdAt: "2026-01-01T00:00:00Z",
    },
  ],
  total: 1,
  page: 0,
  pageSize: 25,
  hasMore: false,
};

describe("AccountsPage", () => {
  it("renders the account list", async () => {
    const client = createMockClient({
      listCustomerAccounts: vi.fn().mockResolvedValue(mockAccounts),
    });

    render(
      <TestWrapper client={client}>
        <AccountsPage />
      </TestWrapper>,
    );

    expect(await screen.findByRole("heading", { name: "Accounts" })).toBeInTheDocument();
    expect(screen.getByText("Customer One")).toBeInTheDocument();
  });
});
