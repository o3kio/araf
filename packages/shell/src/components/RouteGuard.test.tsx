import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { TenantRouteGuard, isOperatorRoute } from "./RouteGuard";

describe("isOperatorRoute", () => {
  it.each([
    ["/operator", true],
    ["/platform/overview", true],
    ["/infrastructure/providers", true],
    ["/tenant", false],
    ["/", false],
    ["/operations", false],
  ])("%s -> %s", (path, expected) => {
    expect(isOperatorRoute(path)).toBe(expected);
  });
});

describe("TenantRouteGuard", () => {
  afterEach(() => {
    window.history.replaceState(null, "", "/");
  });

  it("renders children on tenant routes", () => {
    window.history.replaceState(null, "", "/tenant");
    render(
      <TenantRouteGuard fallback={<div data-testid="blocked" />}>
        <div data-testid="allowed" />
      </TenantRouteGuard>,
    );
    expect(screen.queryByTestId("allowed")).toBeInTheDocument();
    expect(screen.queryByTestId("blocked")).not.toBeInTheDocument();
  });

  it("renders fallback on operator routes", () => {
    window.history.replaceState(null, "", "/operator");
    render(
      <TenantRouteGuard fallback={<div data-testid="blocked" />}>
        <div data-testid="allowed" />
      </TenantRouteGuard>,
    );
    expect(screen.queryByTestId("allowed")).not.toBeInTheDocument();
    expect(screen.queryByTestId("blocked")).toBeInTheDocument();
  });
});
