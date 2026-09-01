import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { ProjectsPage } from "./ProjectsPage";
import { createMockClient, TestWrapper, sessionContext } from "../test/client";
import { runAxe } from "../test/axe-helper";
import type { PaginatedCollection, Project } from "@araf/api-client";

const projects: Project[] = [
  {
    id: "project-1",
    name: "Project 1",
    organizationId: "org-1",
    status: "active",
    createdAt: "2024-01-01T00:00:00Z",
    updatedAt: "2024-01-01T00:01:00Z",
  },
  {
    id: "project-2",
    name: "Project 2",
    organizationId: "org-1",
    status: "active",
    createdAt: "2024-01-02T00:00:00Z",
    updatedAt: "2024-01-02T00:01:00Z",
  },
];

const collection: PaginatedCollection<Project> = {
  items: projects,
  total: 2,
  page: 0,
  pageSize: 25,
  hasMore: false,
};

describe("ProjectsPage", () => {
  it("renders projects and links to detail pages", async () => {
    const client = createMockClient({
      listProjects: vi.fn().mockResolvedValue(collection),
    });

    render(
      <TestWrapper client={client}>
        <ProjectsPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "Project 1" })).toHaveAttribute(
        "href",
        "/organization/projects/project-1",
      );
    });

    expect(screen.getByRole("link", { name: "Project 2" })).toHaveAttribute(
      "href",
      "/organization/projects/project-2",
    );
    expect(screen.getByRole("navigation", { name: "Pagination" })).toBeInTheDocument();
  });

  it("shows an error state when loading fails", async () => {
    const client = createMockClient({
      listProjects: vi.fn().mockRejectedValue(new Error("Network error")),
    });

    render(
      <TestWrapper client={client}>
        <ProjectsPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Could not load projects")).toBeInTheDocument();
    });

    expect(screen.getByText("Network error")).toBeInTheDocument();
  });

  it("hides the list when the session lacks the list capability", async () => {
    const client = createMockClient({
      getContext: vi.fn().mockResolvedValue({
        ...sessionContext,
        capabilities: [],
      }),
    });

    render(
      <TestWrapper client={client}>
        <ProjectsPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Access denied")).toBeInTheDocument();
    });
  });

  it("has no accessibility violations", async () => {
    const client = createMockClient({
      listProjects: vi.fn().mockResolvedValue(collection),
    });

    const { container } = render(
      <TestWrapper client={client}>
        <ProjectsPage />
      </TestWrapper>,
    );

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "Project 1" })).toBeInTheDocument();
    });

    expect(await runAxe(container)).toHaveNoViolations();
  });
});
