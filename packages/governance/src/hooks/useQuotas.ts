import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, PaginatedCollection, ProjectQuota } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseQuotasOptions {
  projectId?: string;
  page?: number;
  pageSize?: number;
}

export interface UseQuotasResult {
  collection: PaginatedCollection<ProjectQuota> | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useQuotas(options?: UseQuotasOptions): UseQuotasResult {
  const client = useGovernanceClient();
  const [collection, setCollection] = useState<PaginatedCollection<ProjectQuota> | undefined>(
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
      .listQuotas({
        projectId: options?.projectId,
        page: options?.page,
        pageSize: options?.pageSize,
      })
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
  }, [client, options?.projectId, options?.page, options?.pageSize, refreshToken]);

  return { collection, loading, error, refresh };
}
