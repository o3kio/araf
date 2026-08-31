import { render, screen } from "@testing-library/react";
import { runAxe } from "../test/axe-helper";
import { describe, expect, it } from "vitest";
import { LoadingState } from "./LoadingState";

describe("LoadingState", () => {
  it("renders loading message", () => {
    render(<LoadingState message="Fetching data..." />);
    expect(screen.getByText("Fetching data...")).toBeVisible();
  });

  it("renders default message", () => {
    render(<LoadingState />);
    expect(screen.getByText("Loading...")).toBeVisible();
  });

  it("announces loading as a status region", () => {
    render(<LoadingState message="Fetching data..." />);
    expect(screen.getByRole("status")).toHaveAttribute("aria-live", "polite");
  });

  it("has no accessibility violations", async () => {
    const { container } = render(<LoadingState message="Fetching data..." />);
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
