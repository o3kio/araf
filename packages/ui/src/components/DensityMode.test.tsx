import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { axe } from "jest-axe";
import { describe, expect, it, vi } from "vitest";
import { DensityMode } from "./DensityMode";

describe("DensityMode", () => {
  it("renders comfortable label when density is comfortable", () => {
    render(<DensityMode density="comfortable" onChange={vi.fn()} />);
    expect(screen.getByText("Comfortable")).toBeVisible();
  });

  it("renders compact label when density is compact", () => {
    render(<DensityMode density="compact" onChange={vi.fn()} />);
    expect(screen.getByText("Compact")).toBeVisible();
  });

  it("calls onChange when toggled", async () => {
    const onChange = vi.fn();
    render(<DensityMode density="comfortable" onChange={onChange} />);
    await userEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledWith("compact");
  });

  it("has no accessibility violations", async () => {
    const { container } = render(<DensityMode density="comfortable" onChange={vi.fn()} />);
    expect(await axe(container)).toHaveNoViolations();
  });
});
