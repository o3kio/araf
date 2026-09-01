import type { ResourceStatus } from "@araf/api-client";
import type { ArafStatusType } from "@araf/ui";

/**
 * Map authoritative BFF resource status to the UI status vocabulary.
 *
 * The runtime does not invent lifecycle states; it only translates the canonical
 * O3K-facing status enum into presentation labels.
 */
export function mapResourceStatus(status: ResourceStatus): {
  type: ArafStatusType;
  label: string;
} {
  switch (status) {
    case "ready":
      return { type: "success", label: "Ready" };
    case "busy":
      return { type: "pending", label: "Busy" };
    case "error":
      return { type: "error", label: "Error" };
    case "unknown":
    default:
      return { type: "info", label: "Unknown" };
  }
}
