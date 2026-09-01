import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, ServiceCatalogEntry } from "@araf/api-client";
import { useResourceClient } from "../client/context";

export interface UseServiceCatalogResult {
  entries: ServiceCatalogEntry[] | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

/**
 * Fetch the tenant-facing service catalog from the BFF.
 */
export function useServiceCatalog(): UseServiceCatalogResult {
  const client = useResourceClient();
  const [entries, setEntries] = useState<ServiceCatalogEntry[] | undefined>(undefined);
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
      .listServiceCatalog()
      .then((result) => {
        if (!cancelled) setEntries(result);
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

  return { entries, loading, error, refresh };
}
