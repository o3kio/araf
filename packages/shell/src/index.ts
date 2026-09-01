// Types
export type {
  Scope,
  Identity,
  RegionId,
  OrganizationOption,
  ProjectOption,
  RegionOption,
} from "./types";

// Scope / identity contexts
export { ScopeProvider, useScope, type ScopeProviderProps } from "./scope/context";
export { IdentityProvider, useIdentity, type IdentityProviderProps } from "./identity/context";

// URL state
export {
  getScopeFromUrl,
  syncScopeToUrl,
  useScopeFromUrl,
  useSyncScopeToUrl,
  mergeScopeWithUrl,
  PROJECT_PARAM,
  REGION_PARAM,
  ORGANIZATION_PARAM,
} from "./url";

// Fixture providers
export {
  FixtureScopeProvider,
  FixtureIdentityProvider,
  type FixtureScopeProviderProps,
  type FixtureIdentityProviderProps,
} from "./fixtures";

// Shell components
export {
  TenantShell,
  type TenantShellProps,
  type TenantNavigationItem,
} from "./components/TenantShell";
export {
  OperatorShell,
  type OperatorShellProps,
  type OperatorNavigationItem,
} from "./components/OperatorShell";
export { ProjectSelector, type ProjectSelectorProps } from "./components/ProjectSelector";
export { RegionSelector, type RegionSelectorProps } from "./components/RegionSelector";
export { ScopeDisplay, type ScopeDisplayProps } from "./components/ScopeDisplay";
export { OperationsNavItem, type OperationsNavItemProps } from "./components/OperationsNavItem";
export {
  TenantRouteGuard,
  isOperatorRoute,
  type TenantRouteGuardProps,
} from "./components/RouteGuard";
