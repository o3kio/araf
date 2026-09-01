import { useCallback, useEffect, useState } from "react";
import type {
  ArafApiError,
  ListOperatorOperationsQuery,
  Operation,
  PaginatedCollection,
} from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseOperatorOperationsResult {
  collection: PaginatedCollection<Operation> | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useOperatorOperations(
  query?: ListOperatorOperationsQuery,
): UseOperatorOperationsResult {
  const client = useOperatorPlatformClient();
  const [collection, setCollection] = useState<PaginatedCollection<Operation> | undefined>(
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
      .listOperatorOperations(query)
      .then((result) => {
        if (!cancelled) setCollection(result);
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
  }, [
    client,
    query?.page,
    query?.pageSize,
    query?.state,
    query?.action,
    query?.regionId,
    query?.accountId,
    query?.resourceType,
    refreshToken,
  ]);

  return { collection, loading, error, refresh };
}
