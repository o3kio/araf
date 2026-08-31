import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AppLayout } from "./AppLayout";

describe("AppLayout", () => {
  it("renders content", () => {
    render(<AppLayout content={<main>Page content</main>} />);
    expect(screen.getByText("Page content")).toBeVisible();
  });

  it("renders navigation when provided", () => {
    render(<AppLayout navigation={<nav>Nav</nav>} content={<main>Page content</main>} />);
    expect(screen.getByText("Nav")).toBeVisible();
  });

  it("renders breadcrumbs when provided", () => {
    render(
      <AppLayout breadcrumbs={<div>Home / Settings</div>} content={<main>Page content</main>} />,
    );
    expect(screen.getByText("Home / Settings")).toBeVisible();
  });
});
