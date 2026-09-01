import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, CustomerAccount, PaginatedCollection } from "@araf/api-client";
import { useOperatorPlatformClient } from "../client/context";

export interface UseCustomerAccountsResult {
  collection: PaginatedCollection<CustomerAccount> | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useCustomerAccounts(): UseCustomerAccountsResult {
  const client = useOperatorPlatformClient();
  const [collection, setCollection] = useState<PaginatedCollection<CustomerAccount> | undefined>(
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
      .listCustomerAccounts()
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
  }, [client, refreshToken]);

  return { collection, loading, error, refresh };
}
