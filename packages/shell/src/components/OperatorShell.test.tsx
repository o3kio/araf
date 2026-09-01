import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OperatorShell } from "./OperatorShell";
import { IdentityProvider } from "../identity/context";
import type { Identity } from "../types";

function Harness({ identity, children }: { identity: Identity; children?: React.ReactNode }) {
  return (
    <IdentityProvider identity={identity} onChange={() => undefined}>
      {children}
    </IdentityProvider>
  );
}

describe("OperatorShell", () => {
  it("renders platform navigation and content", () => {
    render(
      <Harness identity={{ userId: "op1", userName: "Operator" }}>
        <OperatorShell
          navigationItems={[
            { id: "overview", type: "link", text: "Overview", href: "/platform/overview" },
          ]}
          activeHref="/platform/overview"
        >
          <div data-testid="operator-content">Platform content</div>
        </OperatorShell>
      </Harness>,
    );

    expect(screen.getByRole("navigation", { name: /Operator navigation/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Overview/i })).toBeInTheDocument();
    expect(screen.getByTestId("operator-content")).toBeInTheDocument();
    expect(screen.getByTestId("operator-context")).toBeInTheDocument();
  });
});
