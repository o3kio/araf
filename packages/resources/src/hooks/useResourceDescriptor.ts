import { useEffect, useMemo, useState } from "react";
import type { ArafApiError } from "@araf/api-client";
import { useResourceClient } from "../client/context";
import { validateDescriptor, type ResourceDescriptor } from "../descriptor";

export interface UseResourceDescriptorResult {
  descriptor: ResourceDescriptor | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
}

/**
 * Fetch the services catalog and return the validated descriptor for the given
 * resource type. Unknown descriptor fields fail safely and visibly in development.
 */
export function useResourceDescriptor(resourceType: string): UseResourceDescriptorResult {
  const client = useResourceClient();
  const [services, setServices] = useState<
    { id: string; resourceTypes: ResourceDescriptor[] }[] | undefined
  >(undefined);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(undefined);

    client
      .listServices()
      .then((fetched) => {
        if (cancelled) return;
        const mapped = fetched.map((service) => ({
          id: service.id,
          resourceTypes: service.resourceTypes.map((rt) => {
            if (process.env.NODE_ENV !== "production") {
              validateDescriptor(rt);
            }
            return rt;
          }),
        }));
        setServices(mapped);
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setError(err instanceof Error ? err : new Error(String(err)));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [client]);

  const descriptor = useMemo(() => {
    if (!services) return undefined;
    for (const service of services) {
      for (const rt of service.resourceTypes) {
        if (rt.id === resourceType) return rt;
      }
    }
    return undefined;
  }, [services, resourceType]);

  return { descriptor, loading, error };
}
