import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, DiscoveredResourceType } from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseDiscoveredResourceTypesResult {
  resourceTypes: DiscoveredResourceType[] | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

/**
 * Fetch the operator-facing list of discovered resource types from the BFF.
 */
export function useDiscoveredResourceTypes(): UseDiscoveredResourceTypesResult {
  const client = useOperatorPlatformClient();
  const [resourceTypes, setResourceTypes] = useState<DiscoveredResourceType[] | undefined>(
    undefined,
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [refreshToken, setRefreshToken] = useState(0);

  const refresh = useCallback(() => {
    setRefreshToken((t) => t + 1);
  }, []);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(undefined);

    client
      .listDiscoveredResourceTypes()
      .then((result) => {
        if (!cancelled) setResourceTypes(result);
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
  }, [client, refreshToken]);

  return { resourceTypes, loading, error, refresh };
}
