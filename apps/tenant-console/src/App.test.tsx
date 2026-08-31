import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { App } from "./App";

describe("tenant console bootstrap", () => {
  it("renders the tenant surface heading", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Araf Tenant Console" })).toBeVisible();
  });
});
