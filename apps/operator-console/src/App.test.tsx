import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { App } from "./App";

describe("operator console bootstrap", () => {
  it("renders the operator surface heading", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Araf Operator Console" })).toBeVisible();
  });
});
