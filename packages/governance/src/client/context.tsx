import type { ArafClient } from "@araf/api-client";
import { createContext, useContext, type ReactNode } from "react";

const GovernanceClientContext = createContext<ArafClient | null>(null);

export interface GovernanceClientProviderProps {
  client: ArafClient;
  children: ReactNode;
}

/**
 * Provides the Araf API client used by the governance package.
 *
 * The same client instance is typically also provided to @araf/resources and
 * @araf/operations; keeping a separate context here avoids a circular package
 * dependency.
 */
export function GovernanceClientProvider({ client, children }: GovernanceClientProviderProps) {
  return (
    <GovernanceClientContext.Provider value={client}>{children}</GovernanceClientContext.Provider>
  );
}

export function useGovernanceClient(): ArafClient {
  const client = useContext(GovernanceClientContext);
  if (!client) {
    throw new Error("useGovernanceClient must be used within a GovernanceClientProvider");
  }
  return client;
}
