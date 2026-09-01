import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, User } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseUserResult {
  user: User | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useUser(id: string | undefined): UseUserResult {
  const client = useGovernanceClient();
  const [user, setUser] = useState<User | undefined>(undefined);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [refreshToken, setRefreshToken] = useState(0);

  const refresh = useCallback(() => {
    setRefreshToken((t) => t + 1);
  }, []);

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    setLoading(true);
    setError(undefined);

    client
      .getUser(id)
      .then((result) => {
        if (!cancelled) setUser(result);
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

  return { user, loading, error, refresh };
}
