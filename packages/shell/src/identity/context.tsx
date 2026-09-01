import { createContext, useContext, useMemo, type ReactNode } from "react";
import type { Identity } from "../types";

export interface IdentityContextValue {
  identity: Identity;
  setIdentity: (identity: Identity | ((prev: Identity) => Identity)) => void;
}

const IdentityContext = createContext<IdentityContextValue | null>(null);

export interface IdentityProviderProps {
  identity: Identity;
  onChange: (identity: Identity) => void;
  children: ReactNode;
}

/**
 * Provides the current user identity to the console.
 *
 * This is a UI projection. Real authentication and session ownership live in
 * the Rust BFF (M3/M12); this context only surfaces identity information that
 * the BFF chooses to expose to the frontend.
 */
export function IdentityProvider({ identity, onChange, children }: IdentityProviderProps) {
  const value = useMemo<IdentityContextValue>(
    () => ({
      identity,
      setIdentity: (update) => {
        const next = typeof update === "function" ? update(identity) : update;
        onChange(next);
      },
    }),
    [identity, onChange],
  );

  return <IdentityContext.Provider value={value}>{children}</IdentityContext.Provider>;
}

export function useIdentity(): IdentityContextValue {
  const ctx = useContext(IdentityContext);
  if (!ctx) {
    throw new Error("useIdentity must be used within an IdentityProvider");
  }
  return ctx;
}
