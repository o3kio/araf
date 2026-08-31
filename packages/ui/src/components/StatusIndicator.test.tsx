import { render, screen } from "@testing-library/react";
import { axe } from "jest-axe";
import { describe, expect, it } from "vitest";
import { StatusIndicator } from "./StatusIndicator";

describe("StatusIndicator", () => {
  for (const type of ["success", "error", "warning", "info", "pending"] as const) {
    it(`renders ${type} status`, () => {
      render(<StatusIndicator type={type}>{type}</StatusIndicator>);
      expect(screen.getByText(type)).toBeVisible();
    });
  }

  it("has no accessibility violations", async () => {
    const { container } = render(<StatusIndicator type="success">Healthy</StatusIndicator>);
    expect(await axe(container)).toHaveNoViolations();
  });
});
