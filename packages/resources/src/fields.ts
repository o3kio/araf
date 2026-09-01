import type { Resource } from "@araf/api-client";

/**
 * Safely read a field from a resource, including dotted `properties.*` paths.
 *
 * Returns `undefined` when the field is missing so callers can decide how to
 * render absent values. Never throws for malformed paths.
 */
export function getResourceField(resource: Resource, field: string): unknown {
  if (field.startsWith("properties.")) {
    const key = field.slice("properties.".length);
    if (key.length === 0) return undefined;
    return resource.properties?.[key];
  }

  switch (field) {
    case "id":
      return resource.id;
    case "name":
      return resource.name;
    case "resourceType":
      return resource.resourceType;
    case "projectId":
      return resource.projectId;
    case "regionId":
      return resource.regionId;
    case "status":
      return resource.status;
    case "createdAt":
      return resource.createdAt;
    case "updatedAt":
      return resource.updatedAt;
    default:
      return undefined;
  }
}

/**
 * Coerce a field value to a display string without executing any embedded code.
 */
export function formatResourceField(value: unknown): string {
  if (value === undefined || value === null) return "—";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}
