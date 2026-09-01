import { Header, LoadingState, ErrorState } from "@araf/ui";
import { Link } from "react-router";
import { useEffect, useState } from "react";
import type { ArafApiError, ServiceDescriptor } from "@araf/api-client";
import { useResourceClient } from "../client/context";

/**
 * Landing page that lists discovered resource types from the BFF service catalog.
 */
export function ResourceLandingPage() {
  const client = useResourceClient();
  const [services, setServices] = useState<ServiceDescriptor[] | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(undefined);

    client
      .listServices()
      .then((result) => {
        if (!cancelled) setServices(result);
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
  }, [client]);

  return (
    <section aria-label="Resources">
      <Header variant="h1" headingLevel="h1">
        Resources
      </Header>

      {error ? (
        <ErrorState
          title="Could not load resource catalog"
          message={error.message}
          correlationId={
            error instanceof Error && "correlationId" in error ? error.correlationId : undefined
          }
        />
      ) : null}

      {loading ? <LoadingState message="Loading resource catalog..." /> : null}

      {!loading && !error && services ? (
        <div style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}>
          {services.map((service) => (
            <section key={service.id} aria-labelledby={`service-${service.id}`}>
              <h2 id={`service-${service.id}`}>{service.name}</h2>
              <ul>
                {service.resourceTypes.map((rt) => (
                  <li key={rt.id}>
                    <Link to={`/resources/${encodeURIComponent(rt.id)}`}>{rt.pluralName}</Link>
                    <span className="araf-resource-type-id"> ({rt.id})</span>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      ) : null}
    </section>
  );
}
