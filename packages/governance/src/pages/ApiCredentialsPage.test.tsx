import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ApiCredentialsPage } from "./ApiCredentialsPage";
import { createMockClient, TestWrapper } from "../test/client";
import type { ApiCredential, PaginatedCollection } from "@araf/api-client";

const credentials: ApiCredential[] = [
  {
    id: "cred-1",
    name: "ci-runner",
    kind: "service-account",
    projectId: "project-1",
    createdAt: "2024-01-01T00:00:00Z",
    expiresAt: null,
  },
];

const collection: PaginatedCollection<ApiCredential> = {
  items: credentials,
  total: 1,
  page: 0,
  pageSize: 25,
  hasMore: false,
};

describe("ApiCredentialsPage", () => {
  it("renders API credentials and supports deletion", async () => {
    const listApiCredentials = vi.fn().mockResolvedValue(collection);
    const deleteApiCredential = vi.fn().mockResolvedValue(undefined);
    const client = createMockClient({
      listApiCredentials,
      deleteApiCredential,
    });

    render(
      <TestWrapper client={client}>
        <ApiCredentialsPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("ci-runner")).toBeInTheDocument();
    });

    const deleteButtons = screen.getAllByRole("button", { name: "Delete" });
    expect(deleteButtons.length).toBeGreaterThanOrEqual(1);
    const rowDeleteButton = deleteButtons[0];
    if (!rowDeleteButton) throw new Error("Delete button not found");
    await userEvent.click(rowDeleteButton);
    await waitFor(() => {
      expect(screen.getByText("Delete API credential")).toBeInTheDocument();
    });

    const confirmButtons = screen.getAllByRole("button", { name: "Delete" });
    const confirmDeleteButton = confirmButtons[confirmButtons.length - 1];
    if (!confirmDeleteButton) throw new Error("Confirm delete button not found");
    await userEvent.click(confirmDeleteButton);
    await waitFor(() => {
      expect(deleteApiCredential).toHaveBeenCalledWith("cred-1");
    });
  });

  it("shows the one-time secret after creation and does not persist it", async () => {
    const created: ApiCredential = {
      id: "cred-new",
      name: "new-credential",
      kind: "service-account",
      projectId: "project-1",
      createdAt: "2024-01-01T00:00:00Z",
      expiresAt: null,
      secret: "secret-one-time-value",
    };
    const createApiCredential = vi.fn().mockResolvedValue(created);
    const client = createMockClient({
      listApiCredentials: vi.fn().mockResolvedValue({ ...collection, items: [] }),
      createApiCredential,
    });

    render(
      <TestWrapper client={client}>
        <ApiCredentialsPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Create credential" })).toBeInTheDocument();
    });

    await userEvent.click(screen.getByRole("button", { name: "Create credential" }));
    await waitFor(() => {
      expect(screen.getByText("Create API credential")).toBeInTheDocument();
    });

    await userEvent.type(screen.getByLabelText("Name"), "new-credential");
    await userEvent.type(screen.getByLabelText("Project ID"), "project-1");
    const createButton = screen.getAllByRole("button", { name: "Create" })[0];
    if (!createButton) throw new Error("Create button not found");
    await userEvent.click(createButton);

    await waitFor(() => {
      expect(screen.getByText("API credential created")).toBeInTheDocument();
    });

    expect(screen.getByDisplayValue("secret-one-time-value")).toBeInTheDocument();
    expect(createApiCredential).toHaveBeenCalledWith(
      expect.objectContaining({ name: "new-credential", kind: "service-account" }),
    );
  });
});
