/**
 * Araf shell — shared scope and identity types for Tenant and Operator consoles.
 *
 * These types are UI-facing projections. They do not invent O3K authority;
 * authoritative organization, project, region and identity semantics remain
 * owned by the O3K control plane and are consumed through the BFF in later
 * milestones.
 */

/** Region identifier. The literal `"global"` represents global scope. */
export type RegionId = string;

export interface Scope {
  /** Organization or account identifier, when applicable. */
  organizationId?: string;
  /** Human-readable organization label. */
  organizationName?: string;
  /** Project identifier within the current organization. */
  projectId?: string;
  /** Human-readable project label. */
  projectName?: string;
  /** Region identifier, or the literal string `"global"` for global scope. */
  regionId?: RegionId;
  /** Human-readable region label. */
  regionName?: string;
}

export interface Identity {
  /** Stable user identifier. */
  userId: string;
  /** Display name. */
  userName: string;
  /** Contact identifier, when available. */
  email?: string;
}

export interface OrganizationOption {
  id: string;
  name: string;
}

export interface ProjectOption {
  id: string;
  name: string;
  organizationId?: string;
}

export interface RegionOption {
  id: RegionId;
  name: string;
}
