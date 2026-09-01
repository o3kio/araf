import type { ArafClient } from "@araf/api-client";
import { createContext, useContext, type ReactNode } from "react";

const ResourceClientContext = createContext<ArafClient | null>(null);

export interface ResourceClientProviderProps {
  client: ArafClient;
  children: ReactNode;
}

/**
 * Provides the Araf API client used by the generic resource runtime.
 */
export function ResourceClientProvider({ client, children }: ResourceClientProviderProps) {
  return <ResourceClientContext.Provider value={client}>{children}</ResourceClientContext.Provider>;
}

export function useResourceClient(): ArafClient {
  const client = useContext(ResourceClientContext);
  if (!client) {
    throw new Error("useResourceClient must be used within a ResourceClientProvider");
  }
  return client;
}
