import { render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

describe("Operator console shell", () => {
  async function renderApp() {
    render(<App />);
    await screen.findAllByText("Platform Operator");
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
                surface: "operator-bff",
                userId: "operator-user",
                userName: "Platform Operator",
                organizationId: null,
                projectId: null,
                regionId: null,
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

  it("renders the operator navigation and platform context", async () => {
    await renderApp();
    const nav = screen.getByRole("navigation", { name: /Operator navigation/i });
    expect(nav).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Overview/i })).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Regions/i })).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Health/i })).toBeInTheDocument();
  });

  it("shows the operator identity in the top navigation", async () => {
    await renderApp();
    expect(screen.getAllByText("Araf Operator")[0]).toBeInTheDocument();
    expect(screen.getAllByText("Platform Operator")[0]).toBeInTheDocument();
  });

  it("redirects root to platform overview", async () => {
    window.history.replaceState(null, "", "/");
    await renderApp();
    expect(screen.getByRole("heading", { name: /Platform overview/i })).toBeInTheDocument();
  });

  it("does not expose unsupported placeholder surfaces", async () => {
    for (const path of ["/customers/projects", "/infrastructure/compute", "/governance/iam"]) {
      window.history.replaceState(null, "", path);
      const { unmount } = render(<App />);
      await screen.findAllByText("Platform Operator");
      expect(screen.getByRole("heading", { name: /Not found/i })).toBeInTheDocument();
      expect(screen.queryByText(/implemented in later milestones/i)).not.toBeInTheDocument();
      unmount();
    }
  });
});
