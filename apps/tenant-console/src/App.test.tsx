import { render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

describe("Tenant console shell", () => {
  async function renderApp() {
    render(<App />);
    await screen.findAllByText("Tenant User");
  }

  beforeEach(() => {
    vi.stubGlobal(
      "fetch",
      vi.fn((input: RequestInfo | URL) => {
        const requestUrl =
          typeof input === "string" ? input : input instanceof URL ? input.toString() : input.url;
        if (requestUrl.includes("/api/v1/context")) {
          return Promise.resolve(
            new Response(
              JSON.stringify({
                surface: "tenant-bff",
                userId: "tenant-user",
                userName: "Tenant User",
                organizationId: "org-1",
                projectId: "project-1",
                regionId: "eu-west",
                capabilities: [],
              }),
              { status: 200, headers: { "content-type": "application/json" } },
            ),
          );
        }
        return Promise.resolve(new Response("not found", { status: 404 }));
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    window.history.replaceState(null, "", "/");
  });

  it("renders the tenant navigation and scope selectors", async () => {
    await renderApp();
    const nav = screen.getByRole("navigation", { name: /Tenant navigation/i });
    expect(nav).toBeInTheDocument();
    expect(screen.getByLabelText(/Project/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Region/i)).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Home/i })).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Operations/i })).toBeInTheDocument();
  });

  it("shows the tenant identity in the top navigation", async () => {
    await renderApp();
    expect(screen.getAllByText("Araf Tenant")[0]).toBeInTheDocument();
    expect(screen.getAllByText("Tenant User")[0]).toBeInTheDocument();
  });

  it("exposes only the current authoritative scope", async () => {
    window.history.replaceState(null, "", "/");
    await renderApp();

    expect(screen.getByLabelText(/Project/i)).toHaveValue("project-1");
    expect(screen.getByLabelText(/Region/i)).toHaveValue("eu-west");
    expect(screen.getByRole("option", { name: /project-1/i })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: /eu-west/i })).toBeInTheDocument();
  });

  it("restores scope from the URL on load", async () => {
    window.history.replaceState(null, "", "/?project=project-1&region=eu-west");
    await renderApp();
    expect(screen.getByLabelText(/Project/i)).toHaveValue("project-1");
    expect(screen.getByLabelText(/Region/i)).toHaveValue("eu-west");
  });

  it("blocks operator routes from the tenant surface", async () => {
    window.history.replaceState(null, "", "/operator/something");
    render(<App />);
    await screen.findByText(/Operator routes are not available/i);
    expect(screen.getByText(/Operator routes are not available/i)).toBeInTheDocument();
    expect(
      screen.queryByRole("navigation", { name: /Tenant navigation/i }),
    ).not.toBeInTheDocument();
  });

  it("does not expose an unsupported services placeholder route", async () => {
    window.history.replaceState(null, "", "/services/unsupported");
    render(<App />);
    await screen.findAllByText("Tenant User");
    expect(screen.getByRole("heading", { name: /Not found/i })).toBeInTheDocument();
    expect(screen.queryByText(/implemented in later milestones/i)).not.toBeInTheDocument();
  });
});
