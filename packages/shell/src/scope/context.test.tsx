import { render, screen, renderHook } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it } from "vitest";
import { ScopeProvider, useScope } from "./context";
import type { Scope } from "../types";

function TestHarness({
  initialScope,
  children,
}: {
  initialScope: Scope;
  children?: React.ReactNode;
}) {
  const [scope, setScope] = useState<Scope>(initialScope);
  return (
    <ScopeProvider scope={scope} onChange={setScope}>
      {children}
    </ScopeProvider>
  );
}

function ScopeReader() {
  const { scope, setScope } = useScope();
  return (
    <div>
      <span data-testid="project">{scope.projectId ?? "none"}</span>
      <span data-testid="region">{scope.regionId ?? "none"}</span>
      <button
        type="button"
        onClick={() => {
          setScope((prev) => ({ ...prev, projectId: "p2", projectName: "Beta" }));
        }}
      >
        Change project
      </button>
    </div>
  );
}

describe("ScopeProvider", () => {
  it("provides the initial scope", () => {
    render(
      <TestHarness initialScope={{ projectId: "p1", projectName: "Alpha", regionId: "global" }}>
        <ScopeReader />
      </TestHarness>,
    );

    expect(screen.getByTestId("project")).toHaveTextContent("p1");
    expect(screen.getByTestId("region")).toHaveTextContent("global");
  });

  it("updates scope through setScope", async () => {
    render(
      <TestHarness initialScope={{ projectId: "p1", projectName: "Alpha", regionId: "global" }}>
        <ScopeReader />
      </TestHarness>,
    );

    await userEvent.click(screen.getByRole("button", { name: /Change project/i }));

    expect(screen.getByTestId("project")).toHaveTextContent("p2");
  });

  it("throws when useScope is called outside a provider", () => {
    expect(() => renderHook(() => useScope())).toThrow(/ScopeProvider/);
  });
});
