import type { Capability } from "@araf/api-client";

/**
 * Check whether a capability list includes the requested resource type/action.
 */
export function hasCapability(
  capabilities: readonly Capability[],
  resourceType: string,
  action: string,
): boolean {
  return capabilities.some((c) => c.resourceType === resourceType && c.action === action);
}
