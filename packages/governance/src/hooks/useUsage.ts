import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, UsageSummary } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseUsageOptions {
  projectId?: string;
  resourceType?: string;
  since?: string;
  until?: string;
}

export interface UseUsageResult {
  summary: UsageSummary | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useUsage(options?: UseUsageOptions): UseUsageResult {
  const client = useGovernanceClient();
  const [summary, setSummary] = useState<UsageSummary | undefined>(undefined);
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
      .listUsage({
        projectId: options?.projectId,
        resourceType: options?.resourceType,
        since: options?.since,
        until: options?.until,
      })
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
  }, [
    client,
    options?.projectId,
    options?.resourceType,
    options?.since,
    options?.until,
    refreshToken,
  ]);

  return { summary, loading, error, refresh };
}
