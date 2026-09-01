import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, AuditEvent, PaginatedCollection } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseAuditEventsOptions {
  projectId?: string;
  action?: string;
  actor?: string;
  since?: string;
  until?: string;
  page?: number;
  pageSize?: number;
}

export interface UseAuditEventsResult {
  collection: PaginatedCollection<AuditEvent> | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useAuditEvents(options?: UseAuditEventsOptions): UseAuditEventsResult {
  const client = useGovernanceClient();
  const [collection, setCollection] = useState<PaginatedCollection<AuditEvent> | undefined>(
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
      .listAuditEvents({
        projectId: options?.projectId,
        action: options?.action,
        actor: options?.actor,
        since: options?.since,
        until: options?.until,
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
  }, [
    client,
    options?.projectId,
    options?.action,
    options?.actor,
    options?.since,
    options?.until,
    options?.page,
    options?.pageSize,
    refreshToken,
  ]);

  return { collection, loading, error, refresh };
}
