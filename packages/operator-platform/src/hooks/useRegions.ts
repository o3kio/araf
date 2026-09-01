import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, Region } from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseRegionsResult {
  regions: Region[] | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useRegions(): UseRegionsResult {
  const client = useOperatorPlatformClient();
  const [regions, setRegions] = useState<Region[] | undefined>(undefined);
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
      .listRegions()
      .then((result) => {
        if (!cancelled) setRegions(result);
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

  return { regions, loading, error, refresh };
}
