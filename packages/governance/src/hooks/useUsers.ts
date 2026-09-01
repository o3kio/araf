import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, PaginatedCollection, User } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseUsersOptions {
  page?: number;
  pageSize?: number;
}

export interface UseUsersResult {
  collection: PaginatedCollection<User> | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useUsers(options?: UseUsersOptions): UseUsersResult {
  const client = useGovernanceClient();
  const [collection, setCollection] = useState<PaginatedCollection<User> | undefined>(undefined);
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
      .listUsers({ page: options?.page, pageSize: options?.pageSize })
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
  }, [client, options?.page, options?.pageSize, refreshToken]);

  return { collection, loading, error, refresh };
}
