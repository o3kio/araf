import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StatusIndicator } from "./StatusIndicator";

describe("StatusIndicator", () => {
  for (const type of ["success", "error", "warning", "info", "pending"] as const) {
    it(`renders ${type} status`, () => {
      render(<StatusIndicator type={type}>{type}</StatusIndicator>);
      expect(screen.getByText(type)).toBeVisible();
    });
  }

  it("has a status role for screen readers", () => {
    render(<StatusIndicator type="success">Healthy</StatusIndicator>);
    expect(screen.getByText("Healthy")).toBeVisible();
  });
});
