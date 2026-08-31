import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { BootstrapSurface } from "./BootstrapSurface";

describe("BootstrapSurface", () => {
  it("renders an accessible heading and description", () => {
    render(<BootstrapSurface title="Araf Tenant Console" description="Self-service surface" />);

    expect(screen.getByRole("heading", { level: 1, name: "Araf Tenant Console" })).toBeVisible();
    expect(screen.getByText("Self-service surface")).toBeVisible();
  });

  it("renders optional children inside main landmark", () => {
    render(
      <BootstrapSurface title="t" description="d">
        <p>extra</p>
      </BootstrapSurface>,
    );

    expect(screen.getByRole("main")).toHaveTextContent("extra");
  });
});
