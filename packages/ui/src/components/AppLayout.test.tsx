import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { runAxe } from "../test/axe-helper";
import { AppLayout } from "./AppLayout";

describe("AppLayout", () => {
  it("renders content", () => {
    render(<AppLayout content={<div>Page content</div>} />);
    expect(screen.getByText("Page content")).toBeVisible();
  });

  it("renders navigation when provided", () => {
    render(<AppLayout navigation={<nav>Nav</nav>} content={<div>Page content</div>} />);
    expect(screen.getByText("Nav")).toBeVisible();
  });

  it("renders breadcrumbs when provided", () => {
    render(
      <AppLayout breadcrumbs={<div>Home / Settings</div>} content={<div>Page content</div>} />,
    );
    expect(screen.getByText("Home / Settings")).toBeVisible();
  });

  it("has no accessibility violations", async () => {
    const { container } = render(
      <AppLayout
        navigation={<nav>Nav</nav>}
        breadcrumbs={<div>Home</div>}
        content={<div>Page content</div>}
      />,
    );
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
