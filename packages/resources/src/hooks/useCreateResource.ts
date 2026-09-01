import { useCallback, useState } from "react";
import type { ArafApiError, Operation } from "@araf/api-client";
import { useResourceClient } from "../client/context";

export interface UseCreateResourceResult {
  readonly create: (payload: unknown) => Promise<Operation | undefined>;
  readonly loading: boolean;
  readonly error: ArafApiError | Error | undefined;
  readonly operation: Operation | undefined;
}

/**
 * Hook to create a resource of the given type.
 *
 * Returns the canonical Operation returned by the BFF. The caller is responsible
 * for checking `createCapability` before enabling the action.
 */
export function useCreateResource(resourceType: string): UseCreateResourceResult {
  const client = useResourceClient();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [operation, setOperation] = useState<Operation | undefined>(undefined);

  const create = useCallback(
    async (payload: unknown): Promise<Operation | undefined> => {
      setLoading(true);
      setError(undefined);
      setOperation(undefined);

      try {
        const result = await client.createResource(resourceType, payload);
        setOperation(result);
        return result;
      } catch (err: unknown) {
        const normalized = err instanceof Error ? err : new Error(String(err));
        setError(normalized);
        return undefined;
      } finally {
        setLoading(false);
      }
    },
    [client, resourceType],
  );

  return { create, loading, error, operation };
}
