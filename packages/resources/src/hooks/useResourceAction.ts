import { useCallback, useState } from "react";
import type { ArafApiError, Operation } from "@araf/api-client";
import { useResourceClient } from "../client/context";

export interface UseResourceActionResult {
  readonly submit: (actionId: string, payload?: unknown) => Promise<Operation | undefined>;
  readonly loading: boolean;
  readonly error: ArafApiError | Error | undefined;
  readonly operation: Operation | undefined;
}

/**
 * Hook to submit an action on a specific resource.
 *
 * Returns the canonical Operation returned by the BFF. The caller is responsible
 * for checking `requiredCapability` before enabling the action.
 */
export function useResourceAction(resourceType: string, id: string): UseResourceActionResult {
  const client = useResourceClient();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [operation, setOperation] = useState<Operation | undefined>(undefined);

  const submit = useCallback(
    async (actionId: string, payload?: unknown): Promise<Operation | undefined> => {
      setLoading(true);
      setError(undefined);
      setOperation(undefined);

      try {
        const result = await client.submitAction(resourceType, id, { actionId, payload });
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
    [client, resourceType, id],
  );

  return { submit, loading, error, operation };
}
