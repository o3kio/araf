import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { axe } from "jest-axe";
import { describe, expect, it, vi } from "vitest";
import { Toast } from "./Toast";

describe("Toast", () => {
  it("renders toast messages", () => {
    render(
      <Toast items={[{ id: "1", type: "success", message: "Created" }]} onDismiss={vi.fn()} />,
    );
    expect(screen.getByText("Created")).toBeVisible();
  });

  it("renders toast header when provided", () => {
    render(
      <Toast
        items={[{ id: "1", type: "error", message: "Failed", header: "Error" }]}
        onDismiss={vi.fn()}
      />,
    );
    expect(screen.getByText("Error")).toBeVisible();
  });

  it("calls onDismiss when dismissed", async () => {
    const onDismiss = vi.fn();
    render(<Toast items={[{ id: "t1", type: "info", message: "Note" }]} onDismiss={onDismiss} />);
    const dismissBtn = screen.getByRole("button", { name: /dismiss notification/i });
    await userEvent.click(dismissBtn);
    expect(onDismiss).toHaveBeenCalledWith("t1");
  });

  it("renders region for alerts", () => {
    render(
      <Toast items={[{ id: "1", type: "warning", message: "Caution" }]} onDismiss={vi.fn()} />,
    );
    expect(screen.getByRole("region")).toBeVisible();
  });

  it("has no accessibility violations", async () => {
    const { container } = render(
      <Toast items={[{ id: "1", type: "info", message: "Note" }]} onDismiss={vi.fn()} />,
    );
    expect(await axe(container)).toHaveNoViolations();
  });
});
