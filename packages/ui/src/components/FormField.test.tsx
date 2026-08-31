import { render, screen } from "@testing-library/react";
import { runAxe } from "../test/axe-helper";
import { describe, expect, it } from "vitest";
import { FormField } from "./FormField";

describe("FormField", () => {
  it("renders label and child input", () => {
    render(
      <FormField label="Project" id="project">
        <input id="project" type="text" />
      </FormField>,
    );
    expect(screen.getByLabelText("Project")).toBeVisible();
  });

  it("renders error text", () => {
    render(
      <FormField label="Name" errorText="Required">
        <input type="text" />
      </FormField>,
    );
    expect(screen.getByText("Required")).toBeVisible();
  });

  it("has no accessibility violations", async () => {
    const { container } = render(
      <FormField label="Project" id="project">
        <input id="project" type="text" />
      </FormField>,
    );
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
