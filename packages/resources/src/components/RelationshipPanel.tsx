import { useEffect, useState } from "react";
import { Link } from "react-router";
import type { ArafApiError, Resource } from "@araf/api-client";
import { useResourceClient } from "../client/context";
import { getResourceField } from "../fields";
import type { RelationshipDescriptor } from "../descriptor";

export interface RelationshipPanelProps {
  resource: Resource;
  relationship: RelationshipDescriptor;
}

/**
 * Scope-safe relationship renderer.
 *
 * For to-one relationships this fetches the related resource by reading the
 * source property from the parent resource and links to it, preserving the
 * current scope in the URL.
 */
export function RelationshipPanel({ resource, relationship }: RelationshipPanelProps) {
  const client = useResourceClient();
  const [related, setRelated] = useState<Resource | undefined>(undefined);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);

  const rawValue = getResourceField(resource, `properties.${relationship.sourcePropertyKey}`);
  const relatedId = typeof rawValue === "string" ? rawValue : undefined;

  useEffect(() => {
    if (relationship.direction !== "to-one" || !relatedId) return;

    let cancelled = false;
    setLoading(true);
    setError(undefined);

    client
      .getResource(relationship.targetResourceType, relatedId)
      .then((result) => {
        if (!cancelled) setRelated(result);
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
  }, [client, relationship, relatedId]);

  if (!relatedId) {
    return <p>No {relationship.label.toLowerCase()} attached.</p>;
  }

  const encodedType = encodeURIComponent(relationship.targetResourceType);
  const encodedId = encodeURIComponent(relatedId);

  return (
    <div className="araf-relationship-panel">
      <h3>{relationship.label}</h3>
      {loading ? <p>Loading {relationship.label.toLowerCase()}...</p> : null}
      {error ? (
        <p role="alert">
          Could not load {relationship.label.toLowerCase()}: {error.message}
        </p>
      ) : null}
      {related ? (
        <p>
          <Link to={`/resources/${encodedType}/${encodedId}`}>{related.name}</Link>
          <span className="araf-resource-id"> ({related.id})</span>
        </p>
      ) : null}
      {!loading && !error && !related ? (
        <p>
          <Link to={`/resources/${encodedType}/${encodedId}`}>{relatedId}</Link>
        </p>
      ) : null}
    </div>
  );
}
