import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, PaginatedCollection, Role } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseRolesOptions {
  page?: number;
  pageSize?: number;
}

export interface UseRolesResult {
  collection: PaginatedCollection<Role> | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useRoles(options?: UseRolesOptions): UseRolesResult {
  const client = useGovernanceClient();
  const [collection, setCollection] = useState<PaginatedCollection<Role> | undefined>(undefined);
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
      .listRoles({ page: options?.page, pageSize: options?.pageSize })
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
