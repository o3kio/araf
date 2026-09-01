import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, PlatformOverview } from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UsePlatformOverviewResult {
  overview: PlatformOverview | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function usePlatformOverview(): UsePlatformOverviewResult {
  const client = useOperatorPlatformClient();
  const [overview, setOverview] = useState<PlatformOverview | undefined>(undefined);
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
      .getPlatformOverview()
      .then((result) => {
        if (!cancelled) setOverview(result);
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

  return { overview, loading, error, refresh };
}
