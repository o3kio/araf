import { render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { App } from "./App";

describe("Operator console shell", () => {
  afterEach(() => {
    window.history.replaceState(null, "", "/");
  });

  it("renders the operator navigation and platform context", () => {
    render(<App />);
    const nav = screen.getByRole("navigation", { name: /Operator navigation/i });
    expect(nav).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Overview/i })).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Regions/i })).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Health/i })).toBeInTheDocument();
  });

  it("shows the operator identity in the top navigation", () => {
    render(<App />);
    expect(screen.getAllByText("Araf Operator")[0]).toBeInTheDocument();
    expect(screen.getAllByText("Platform Operator")[0]).toBeInTheDocument();
  });

  it("redirects root to platform overview", () => {
    window.history.replaceState(null, "", "/");
    render(<App />);
    expect(screen.getByRole("heading", { name: /Platform overview/i })).toBeInTheDocument();
  });

  it("does not expose unsupported placeholder surfaces", () => {
    for (const path of ["/customers/projects", "/infrastructure/compute", "/governance/iam"]) {
      window.history.replaceState(null, "", path);
      const { unmount } = render(<App />);
      expect(screen.getByRole("heading", { name: /Not found/i })).toBeInTheDocument();
      expect(screen.queryByText(/implemented in later milestones/i)).not.toBeInTheDocument();
      unmount();
    }
  });
});
