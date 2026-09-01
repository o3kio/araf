import { render, screen, renderHook } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it } from "vitest";
import { IdentityProvider, useIdentity } from "./context";
import type { Identity } from "../types";

function TestHarness({
  initialIdentity,
  children,
}: {
  initialIdentity: Identity;
  children?: React.ReactNode;
}) {
  const [identity, setIdentity] = useState<Identity>(initialIdentity);
  return (
    <IdentityProvider identity={identity} onChange={setIdentity}>
      {children}
    </IdentityProvider>
  );
}

function IdentityReader() {
  const { identity, setIdentity } = useIdentity();
  return (
    <div>
      <span data-testid="user">{identity.userName}</span>
      <button
        type="button"
        onClick={() => {
          setIdentity((prev) => ({ ...prev, userName: "Updated User" }));
        }}
      >
        Rename
      </button>
    </div>
  );
}

describe("IdentityProvider", () => {
  it("provides the initial identity", () => {
    render(
      <TestHarness initialIdentity={{ userId: "u1", userName: "Alice" }}>
        <IdentityReader />
      </TestHarness>,
    );

    expect(screen.getByTestId("user")).toHaveTextContent("Alice");
  });

  it("updates identity through setIdentity", async () => {
    render(
      <TestHarness initialIdentity={{ userId: "u1", userName: "Alice" }}>
        <IdentityReader />
      </TestHarness>,
    );

    await userEvent.click(screen.getByRole("button", { name: /Rename/i }));

    expect(screen.getByTestId("user")).toHaveTextContent("Updated User");
  });

  it("throws when useIdentity is called outside a provider", () => {
    expect(() => renderHook(() => useIdentity())).toThrow(/IdentityProvider/);
  });
});
