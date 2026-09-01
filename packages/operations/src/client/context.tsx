import type { ArafClient } from "@araf/api-client";
import { createContext, useContext, type ReactNode } from "react";

const OperationsClientContext = createContext<ArafClient | null>(null);

export interface OperationsClientProviderProps {
  client: ArafClient;
  children: ReactNode;
}

/**
 * Provides the Araf API client used by the operations package.
 *
 * The same client instance is typically also provided to @araf/resources via
 * ResourceClientProvider; keeping a separate context here avoids a circular
 * package dependency.
 */
export function OperationsClientProvider({ client, children }: OperationsClientProviderProps) {
  return (
    <OperationsClientContext.Provider value={client}>{children}</OperationsClientContext.Provider>
  );
}

export function useOperationsClient(): ArafClient {
  const client = useContext(OperationsClientContext);
  if (!client) {
    throw new Error("useOperationsClient must be used within an OperationsClientProvider");
  }
  return client;
}
