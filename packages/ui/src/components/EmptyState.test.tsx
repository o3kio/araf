import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { runAxe } from "../test/axe-helper";
import { describe, expect, it, vi } from "vitest";
import { EmptyState } from "./EmptyState";

describe("EmptyState", () => {
  it("renders title and description", () => {
    render(<EmptyState title="No resources" description="Create your first resource." />);
    expect(screen.getByRole("heading", { name: "No resources" })).toBeVisible();
    expect(screen.getByText("Create your first resource.")).toBeVisible();
  });

  it("renders action button and handles click", async () => {
    const onPress = vi.fn();
    render(<EmptyState action={{ label: "Create", onPress }} />);
    await userEvent.click(screen.getByRole("button", { name: "Create" }));
    expect(onPress).toHaveBeenCalledTimes(1);
  });

  it("is polite live region", () => {
    render(<EmptyState title="No resources" />);
    expect(screen.getByRole("status")).toHaveAttribute("aria-live", "polite");
  });

  it("has no accessibility violations", async () => {
    const { container } = render(
      <EmptyState
        title="No resources"
        description="Create your first resource."
        action={{ label: "Create", onPress: vi.fn() }}
      />,
    );
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
