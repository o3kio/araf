import { useCallback, useState } from "react";
import type { ArafApiError, ApiCredential, CreateApiCredentialRequest } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseCreateApiCredentialResult {
  create: (payload: CreateApiCredentialRequest) => Promise<ApiCredential>;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  reset: () => void;
}

export function useCreateApiCredential(): UseCreateApiCredentialResult {
  const client = useGovernanceClient();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);

  const create = useCallback(
    async (payload: CreateApiCredentialRequest): Promise<ApiCredential> => {
      setLoading(true);
      setError(undefined);
      try {
        const result = await client.createApiCredential(payload);
        return result;
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

  return { create, loading, error, reset };
}
