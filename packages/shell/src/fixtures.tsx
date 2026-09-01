import { useState, type ReactNode } from "react";
import { ScopeProvider } from "./scope/context";
import { IdentityProvider } from "./identity/context";
import { useScopeFromUrl, useSyncScopeToUrl, mergeScopeWithUrl } from "./url";
import type { Identity, Scope } from "./types";

export interface FixtureScopeProviderProps {
  initialScope: Scope;
  children: ReactNode;
}

/**
 * Fixture-only scope provider.
 *
 * Seeds scope state from `initialScope` and merges URL parameters so that
 * refresh/back navigation keeps the selected project/region. Real production
 * scope will be sourced from the authenticated session and O3K control plane.
 */
export function FixtureScopeProvider({ initialScope, children }: FixtureScopeProviderProps) {
  const urlScope = useScopeFromUrl();
  const [scope, setScope] = useState<Scope>(() => mergeScopeWithUrl(initialScope, urlScope));

  useSyncScopeToUrl(scope);

  return (
    <ScopeProvider scope={scope} onChange={setScope}>
      {children}
    </ScopeProvider>
  );
}

export interface FixtureIdentityProviderProps {
  initialIdentity: Identity;
  children: ReactNode;
}

/**
 * Fixture-only identity provider.
 *
 * Accepts a hard-coded identity. Production identity comes from the BFF session.
 */
export function FixtureIdentityProvider({
  initialIdentity,
  children,
}: FixtureIdentityProviderProps) {
  const [identity, setIdentity] = useState<Identity>(initialIdentity);

  return (
    <IdentityProvider identity={identity} onChange={setIdentity}>
      {children}
    </IdentityProvider>
  );
}
