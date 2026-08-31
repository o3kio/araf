import { render } from "@testing-library/react";
import { runAxe } from "../test/axe-helper";
import { describe, expect, it } from "vitest";
import { ArafThemeProvider } from "./ArafThemeProvider";

describe("ArafThemeProvider", () => {
  it("applies the araf-theme class to documentElement", () => {
    render(<ArafThemeProvider>content</ArafThemeProvider>);
    expect(document.documentElement.classList.contains("araf-theme")).toBe(true);
  });

  it("applies compact density class to body", () => {
    render(<ArafThemeProvider density="compact">content</ArafThemeProvider>);
    expect(document.body.classList.contains("araf-density-compact")).toBe(true);
  });

  it("cleans up classes on unmount", () => {
    const { unmount } = render(<ArafThemeProvider density="compact">content</ArafThemeProvider>);
    unmount();
    expect(document.documentElement.classList.contains("araf-theme")).toBe(false);
    expect(document.body.classList.contains("araf-density-compact")).toBe(false);
  });

  it("keeps theme class alive when multiple providers are mounted", () => {
    const { unmount: unmountA } = render(<ArafThemeProvider>provider a</ArafThemeProvider>);
    const { unmount: unmountB } = render(<ArafThemeProvider>provider b</ArafThemeProvider>);
    expect(document.documentElement.classList.contains("araf-theme")).toBe(true);
    unmountA();
    expect(document.documentElement.classList.contains("araf-theme")).toBe(true);
    unmountB();
    expect(document.documentElement.classList.contains("araf-theme")).toBe(false);
  });

  it("has no accessibility violations", async () => {
    const { container } = render(<ArafThemeProvider>content</ArafThemeProvider>);
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
