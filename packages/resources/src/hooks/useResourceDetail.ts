import { useEffect, useState } from "react";
import type { ArafApiError, Resource } from "@araf/api-client";
import { useResourceClient } from "../client/context";

export interface UseResourceDetailResult {
  resource: Resource | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

/**
 * Fetch a single resource by type and id.
 */
export function useResourceDetail(resourceType: string, id: string): UseResourceDetailResult {
  const client = useResourceClient();
  const [resource, setResource] = useState<Resource | undefined>(undefined);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [refreshToken, setRefreshToken] = useState(0);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(undefined);

    client
      .getResource(resourceType, id)
      .then((result) => {
        if (!cancelled) setResource(result);
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
  }, [client, resourceType, id, refreshToken]);

  const refresh = () => {
    setRefreshToken((t) => t + 1);
  };

  return { resource, loading, error, refresh };
}
