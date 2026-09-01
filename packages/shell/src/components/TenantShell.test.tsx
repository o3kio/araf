import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TenantShell } from "./TenantShell";
import { ScopeProvider } from "../scope/context";
import { IdentityProvider } from "../identity/context";
import type { Scope, Identity } from "../types";

function Harness({
  scope,
  onChange,
  identity,
  children,
}: {
  scope: Scope;
  onChange: (scope: Scope) => void;
  identity: Identity;
  children?: React.ReactNode;
}) {
  return (
    <IdentityProvider identity={identity} onChange={() => undefined}>
      <ScopeProvider scope={scope} onChange={onChange}>
        {children}
      </ScopeProvider>
    </IdentityProvider>
  );
}

describe("TenantShell", () => {
  it("renders navigation, scope selectors and content", () => {
    const onChange = vi.fn();
    render(
      <Harness
        scope={{ projectId: "p1", regionId: "global" }}
        onChange={onChange}
        identity={{ userId: "u1", userName: "Alice" }}
      >
        <TenantShell
          navigationItems={[{ id: "home", type: "link", text: "Home", href: "/" }]}
          activeHref="/"
          projects={[{ id: "p1", name: "Alpha" }]}
          regions={[{ id: "eu-west", name: "EU West" }]}
        >
          <div data-testid="tenant-content">Tenant content</div>
        </TenantShell>
      </Harness>,
    );

    expect(screen.getByRole("navigation", { name: /Tenant navigation/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Home/i })).toBeInTheDocument();
    expect(screen.getByTestId("tenant-content")).toBeInTheDocument();
    expect(screen.getByLabelText(/Project/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Region/i)).toBeInTheDocument();
  });

  it("changes project via keyboard", async () => {
    const onChange = vi.fn();
    render(
      <Harness
        scope={{ projectId: "p1", regionId: "global" }}
        onChange={onChange}
        identity={{ userId: "u1", userName: "Alice" }}
      >
        <TenantShell
          navigationItems={[]}
          projects={[
            { id: "p1", name: "Alpha" },
            { id: "p2", name: "Beta" },
          ]}
          regions={[]}
        >
          <div>content</div>
        </TenantShell>
      </Harness>,
    );

    await userEvent.selectOptions(screen.getByLabelText(/Project/i), "p2");
    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ projectId: "p2", projectName: "Beta" }),
    );
  });
});
