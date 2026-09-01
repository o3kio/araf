import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, AvailabilityZone } from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseAvailabilityZonesResult {
  zones: AvailabilityZone[] | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useAvailabilityZones(regionId: string): UseAvailabilityZonesResult {
  const client = useOperatorPlatformClient();
  const [zones, setZones] = useState<AvailabilityZone[] | undefined>(undefined);
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
      .listAvailabilityZones(regionId)
      .then((result) => {
        if (!cancelled) setZones(result);
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
  }, [client, regionId, refreshToken]);

  return { zones, loading, error, refresh };
}
