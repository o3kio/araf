import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { runAxe } from "../test/axe-helper";
import { describe, expect, it, vi } from "vitest";
import { DensityMode } from "./DensityMode";

describe("DensityMode", () => {
  it("renders the stable label", () => {
    render(<DensityMode density="comfortable" onChange={vi.fn()} />);
    expect(screen.getByLabelText(/Compact mode/)).toBeVisible();
  });

  it("renders the current density value", () => {
    render(<DensityMode density="comfortable" onChange={vi.fn()} />);
    expect(screen.getByText("(Comfortable)")).toBeVisible();
  });

  it("updates the displayed value when compact", () => {
    render(<DensityMode density="compact" onChange={vi.fn()} />);
    expect(screen.getByText("(Compact)")).toBeVisible();
  });

  it("calls onChange when toggled", async () => {
    const onChange = vi.fn();
    render(<DensityMode density="comfortable" onChange={onChange} />);
    await userEvent.click(screen.getByRole("checkbox", { name: /Compact mode/ }));
    expect(onChange).toHaveBeenCalledWith("compact");
  });

  it("has no accessibility violations", async () => {
    const { container } = render(<DensityMode density="comfortable" onChange={vi.fn()} />);
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
