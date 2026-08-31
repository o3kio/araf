import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
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
});
