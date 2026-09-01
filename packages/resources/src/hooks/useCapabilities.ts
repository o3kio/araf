import { useEffect, useState } from "react";
import type { ArafApiError, Capability } from "@araf/api-client";
import { useResourceClient } from "../client/context";

export interface UseCapabilitiesResult {
  readonly capabilities: readonly Capability[];
  readonly loading: boolean;
  readonly error: ArafApiError | Error | undefined;
}

/**
 * Fetch and cache the session capabilities from the BFF context.
 *
 * This keeps the resources package decoupled from shell internals; it reads
 * identity/capabilities through the same API client used for resource calls.
 */
export function useCapabilities(): UseCapabilitiesResult {
  const client = useResourceClient();
  const [capabilities, setCapabilities] = useState<readonly Capability[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(undefined);

    client
      .getContext()
      .then((ctx) => {
        if (!cancelled) setCapabilities(ctx.capabilities);
      })
      .catch((err: unknown) => {
        if (!cancelled) setError(err instanceof Error ? err : new Error(String(err)));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [client]);

  return { capabilities, loading, error };
}
