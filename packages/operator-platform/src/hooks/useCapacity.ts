import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, CapacitySummary } from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseCapacityResult {
  summary: CapacitySummary[] | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useCapacity(): UseCapacityResult {
  const client = useOperatorPlatformClient();
  const [summary, setSummary] = useState<CapacitySummary[] | undefined>(undefined);
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
      .getCapacitySummary()
      .then((result) => {
        if (!cancelled) setSummary(result);
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

  return { summary, loading, error, refresh };
}
