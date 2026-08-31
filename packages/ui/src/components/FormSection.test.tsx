import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { runAxe } from "../test/axe-helper";
import { FormSection } from "./FormSection";

describe("FormSection", () => {
  it("renders title and description", () => {
    render(
      <FormSection title="Network" description="Configure network settings.">
        <div>content</div>
      </FormSection>,
    );
    expect(screen.getByRole("heading", { level: 2, name: "Network" })).toBeVisible();
    expect(screen.getByText("Configure network settings.")).toBeVisible();
  });

  it("has no accessibility violations", async () => {
    const { container } = render(
      <FormSection title="Network" description="Configure network settings.">
        <div>content</div>
      </FormSection>,
    );
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
