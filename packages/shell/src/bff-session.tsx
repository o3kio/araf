import { useEffect, useState, type ReactNode } from "react";
import { IdentityProvider } from "./identity/context";
import { ScopeProvider } from "./scope/context";
import type { Identity, Scope } from "./types";

export interface BffSessionContext {
  userId: string;
  userName: string;
  organizationId: string | null;
  projectId: string | null;
  regionId: string | null;
}

interface BffSessionProviderProps {
  loadContext: () => Promise<BffSessionContext>;
  children: ReactNode;
}

/**
 * Hydrates the shell from the authenticated BFF session. No identity or
 * scope is synthesized in the browser; an unavailable session is an error.
 */
export function BffSessionProvider({ loadContext, children }: BffSessionProviderProps) {
  const [context, setContext] = useState<BffSessionContext | null>(null);
  const [error, setError] = useState<unknown>(null);

  useEffect(() => {
    let cancelled = false;
    loadContext()
      .then((next) => {
        if (!cancelled) setContext(next);
      })
      .catch((reason: unknown) => {
        if (!cancelled) setError(reason);
      });
    return () => {
      cancelled = true;
    };
  }, [loadContext]);

  if (error) {
    return (
      <main role="alert" style={{ padding: "2rem" }}>
        <h1>Session unavailable</h1>
        <p>Sign in again to continue.</p>
      </main>
    );
  }
  if (!context) {
    return <main style={{ padding: "2rem" }}>Loading session…</main>;
  }

  const identity: Identity = { userId: context.userId, userName: context.userName };
  const scope: Scope = {
    organizationId: context.organizationId ?? undefined,
    projectId: context.projectId ?? undefined,
    regionId: context.regionId ?? undefined,
  };

  return (
    <IdentityProvider identity={identity} onChange={() => undefined}>
      <ScopeProvider scope={scope} onChange={() => undefined}>
        {children}
      </ScopeProvider>
    </IdentityProvider>
  );
}
