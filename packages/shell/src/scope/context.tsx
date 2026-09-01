import { createContext, useContext, useMemo, type ReactNode } from "react";
import type { Scope } from "../types";

export interface ScopeContextValue {
  scope: Scope;
  setScope: (scope: Scope | ((prev: Scope) => Scope)) => void;
}

const ScopeContext = createContext<ScopeContextValue | null>(null);

export interface ScopeProviderProps {
  scope: Scope;
  onChange: (scope: Scope) => void;
  children: ReactNode;
}

/**
 * Provides the current project/region scope to the console.
 *
 * This provider intentionally holds only UI scope state. It does not enforce
 * authorization; server-side checks in the BFF and O3K remain authoritative.
 */
export function ScopeProvider({ scope, onChange, children }: ScopeProviderProps) {
  const value = useMemo<ScopeContextValue>(
    () => ({
      scope,
      setScope: (update) => {
        const next = typeof update === "function" ? update(scope) : update;
        onChange(next);
      },
    }),
    [scope, onChange],
  );

  return <ScopeContext.Provider value={value}>{children}</ScopeContext.Provider>;
}

export function useScope(): ScopeContextValue {
  const ctx = useContext(ScopeContext);
  if (!ctx) {
    throw new Error("useScope must be used within a ScopeProvider");
  }
  return ctx;
}
