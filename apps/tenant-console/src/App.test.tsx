import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { App } from "./App";

describe("Tenant console shell", () => {
  afterEach(() => {
    window.history.replaceState(null, "", "/");
  });

  it("renders the tenant navigation and scope selectors", () => {
    render(<App />);
    const nav = screen.getByRole("navigation", { name: /Tenant navigation/i });
    expect(nav).toBeInTheDocument();
    expect(screen.getByLabelText(/Project/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Region/i)).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Home/i })).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Operations/i })).toBeInTheDocument();
  });

  it("shows the tenant identity in the top navigation", () => {
    render(<App />);
    expect(screen.getAllByText("Araf Tenant")[0]).toBeInTheDocument();
    expect(screen.getAllByText("Tenant User")[0]).toBeInTheDocument();
  });

  it("retains selected scope in the URL", async () => {
    window.history.replaceState(null, "", "/");
    render(<App />);

    await userEvent.selectOptions(screen.getByLabelText(/Project/i), "Beta");
    expect(window.location.search).toContain("project=project-beta");

    await userEvent.selectOptions(screen.getByLabelText(/Region/i), "eu-west");
    expect(window.location.search).toContain("region=eu-west");
  });

  it("restores scope from the URL on load", () => {
    window.history.replaceState(null, "", "/?project=project-beta&region=eu-west");
    render(<App />);
    expect(screen.getByLabelText(/Project/i)).toHaveValue("project-beta");
    expect(screen.getByLabelText(/Region/i)).toHaveValue("eu-west");
  });

  it("blocks operator routes from the tenant surface", () => {
    window.history.replaceState(null, "", "/operator/something");
    render(<App />);
    expect(screen.getByText(/Operator routes are not available/i)).toBeInTheDocument();
    expect(
      screen.queryByRole("navigation", { name: /Tenant navigation/i }),
    ).not.toBeInTheDocument();
  });
});
