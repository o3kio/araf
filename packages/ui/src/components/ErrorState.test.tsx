import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { axe } from "jest-axe";
import { describe, expect, it, vi } from "vitest";
import { ErrorState } from "./ErrorState";

describe("ErrorState", () => {
  it("renders error with title and message", () => {
    render(<ErrorState title="Failed" message="Something went wrong." />);
    expect(screen.getByText("Failed")).toBeVisible();
    expect(screen.getByText("Something went wrong.")).toBeVisible();
  });

  it("renders correlation ID", () => {
    render(<ErrorState title="Error" correlationId="abc-123" />);
    expect(screen.getByText(/abc-123/)).toBeVisible();
  });

  it("calls onRetry when retry button clicked", async () => {
    const onRetry = vi.fn();
    render(<ErrorState title="Error" onRetry={onRetry} />);
    await userEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("has no accessibility violations", async () => {
    const { container } = render(
      <ErrorState
        title="Failed"
        message="Something went wrong."
        correlationId="abc-123"
        onRetry={vi.fn()}
      />,
    );
    expect(await axe(container)).toHaveNoViolations();
  });
});
