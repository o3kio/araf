import { useCallback, useState } from "react";
import type { ArafApiError } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseDeleteApiCredentialResult {
  deleteCredential: (id: string) => Promise<void>;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  reset: () => void;
}

export function useDeleteApiCredential(): UseDeleteApiCredentialResult {
  const client = useGovernanceClient();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);

  const deleteCredential = useCallback(
    async (id: string): Promise<void> => {
      setLoading(true);
      setError(undefined);
      try {
        await client.deleteApiCredential(id);
      } catch (err: unknown) {
        const normalized = err instanceof Error ? err : new Error(String(err));
        setError(normalized);
        throw normalized;
      } finally {
        setLoading(false);
      }
    },
    [client],
  );

  const reset = useCallback(() => {
    setLoading(false);
    setError(undefined);
  }, []);

  return { deleteCredential, loading, error, reset };
}
