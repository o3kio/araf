import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Header } from "./Header";

describe("Header", () => {
  it("renders heading text", () => {
    render(<Header>Resources</Header>);
    expect(screen.getByRole("heading", { level: 1, name: "Resources" })).toBeVisible();
  });

  it("renders description", () => {
    render(<Header description="Manage resources">Resources</Header>);
    expect(screen.getByText("Manage resources")).toBeVisible();
  });

  it("supports h2 variant", () => {
    render(<Header variant="h2">Section</Header>);
    expect(screen.getByRole("heading", { level: 2, name: "Section" })).toBeVisible();
  });
});
