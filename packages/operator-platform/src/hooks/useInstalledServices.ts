import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, InstalledService } from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseInstalledServicesResult {
  services: InstalledService[] | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

/**
 * Fetch the operator-facing list of installed services from the BFF.
 */
export function useInstalledServices(): UseInstalledServicesResult {
  const client = useOperatorPlatformClient();
  const [services, setServices] = useState<InstalledService[] | undefined>(undefined);
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
      .listInstalledServices()
      .then((result) => {
        if (!cancelled) setServices(result);
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

  return { services, loading, error, refresh };
}
