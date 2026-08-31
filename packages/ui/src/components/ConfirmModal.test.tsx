import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ConfirmModal } from "./ConfirmModal";

describe("ConfirmModal", () => {
  it("renders modal with title and content", () => {
    render(
      <ConfirmModal open title="Delete?" onConfirm={vi.fn()} onCancel={vi.fn()}>
        <p>Are you sure?</p>
      </ConfirmModal>,
    );
    expect(screen.getByRole("dialog")).toBeVisible();
    expect(screen.getByText("Delete?")).toBeVisible();
    expect(screen.getByText("Are you sure?")).toBeVisible();
  });

  it("calls onConfirm when confirm clicked", async () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmModal open title="Confirm" onConfirm={onConfirm} onCancel={vi.fn()}>
        <p>Proceed?</p>
      </ConfirmModal>,
    );
    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("calls onCancel when cancel clicked", async () => {
    const onCancel = vi.fn();
    render(
      <ConfirmModal open title="Cancel?" onConfirm={vi.fn()} onCancel={onCancel}>
        <p>Abort?</p>
      </ConfirmModal>,
    );
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});
