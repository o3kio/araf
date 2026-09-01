import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, Operation } from "@araf/api-client";
import { useOperationsClient } from "../client/context";

export interface UseOperationResult {
  operation: Operation | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

/**
 * Fetch a single canonical O3K Operation by id.
 *
 * The returned `refresh` function can be used to poll for state transitions.
 */
export function useOperation(id: string | undefined): UseOperationResult {
  const client = useOperationsClient();
  const [operation, setOperation] = useState<Operation | undefined>(undefined);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [refreshToken, setRefreshToken] = useState(0);

  useEffect(() => {
    if (!id) return;

    let cancelled = false;
    setLoading(true);
    setError(undefined);

    client
      .getOperation(id)
      .then((result) => {
        if (!cancelled) setOperation(result);
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
  }, [client, id, refreshToken]);

  const refresh = useCallback(() => {
    setRefreshToken((t) => t + 1);
  }, []);

  return { operation, loading, error, refresh };
}
