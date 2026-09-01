import type { ArafClient } from "@araf/api-client";
import { createContext, useContext, type ReactNode } from "react";

const OperatorPlatformClientContext = createContext<ArafClient | null>(null);

export interface OperatorPlatformClientProviderProps {
  client: ArafClient;
  children: ReactNode;
}

/**
 * Provides the Araf API client used by the operator platform package.
 *
 * Keeping a separate context avoids coupling operator platform pages to the
 * resource/operations governance providers used by other console areas.
 */
export function OperatorPlatformClientProvider({
  client,
  children,
}: OperatorPlatformClientProviderProps) {
  return (
    <OperatorPlatformClientContext.Provider value={client}>
      {children}
    </OperatorPlatformClientContext.Provider>
  );
}

export function useOperatorPlatformClient(): ArafClient {
  const client = useContext(OperatorPlatformClientContext);
  if (!client) {
    throw new Error(
      "useOperatorPlatformClient must be used within an OperatorPlatformClientProvider",
    );
  }
  return client;
}
