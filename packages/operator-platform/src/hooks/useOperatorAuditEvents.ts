import { useCallback, useEffect, useState } from "react";
import type {
  ArafApiError,
  ListOperatorAuditEventsQuery,
  OperatorAuditEvent,
  PaginatedCollection,
} from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseOperatorAuditEventsResult {
  collection: PaginatedCollection<OperatorAuditEvent> | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useOperatorAuditEvents(
  query?: ListOperatorAuditEventsQuery,
): UseOperatorAuditEventsResult {
  const client = useOperatorPlatformClient();
  const [collection, setCollection] = useState<PaginatedCollection<OperatorAuditEvent> | undefined>(
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
      .listOperatorAuditEvents(query)
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
    query?.action,
    query?.actor,
    query?.accountId,
    refreshToken,
  ]);

  return { collection, loading, error, refresh };
}
