import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, ProviderHealth } from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseProviderHealthResult {
  providers: ProviderHealth[] | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useProviderHealth(): UseProviderHealthResult {
  const client = useOperatorPlatformClient();
  const [providers, setProviders] = useState<ProviderHealth[] | undefined>(undefined);
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
      .listProviderHealth()
      .then((result) => {
        if (!cancelled) setProviders(result);
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

  return { providers, loading, error, refresh };
}
