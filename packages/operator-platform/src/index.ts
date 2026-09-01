// Client provider
export {
  OperatorPlatformClientProvider,
  useOperatorPlatformClient,
  type OperatorPlatformClientProviderProps,
} from "./client/context";

// Capabilities
export { hasCapability } from "./capabilities";

// Hooks
export { usePlatformOverview, type UsePlatformOverviewResult } from "./hooks/usePlatformOverview";
export { useRegions, type UseRegionsResult } from "./hooks/useRegions";
export {
  useAvailabilityZones,
  type UseAvailabilityZonesResult,
} from "./hooks/useAvailabilityZones";
export { useProviderHealth, type UseProviderHealthResult } from "./hooks/useProviderHealth";
export { useCapacity, type UseCapacityResult } from "./hooks/useCapacity";
export { useCustomerAccounts, type UseCustomerAccountsResult } from "./hooks/useCustomerAccounts";
export { useAccountProjects, type UseAccountProjectsResult } from "./hooks/useAccountProjects";
export {
  useOperatorOperations,
  type UseOperatorOperationsResult,
} from "./hooks/useOperatorOperations";
export {
  useOperatorAuditEvents,
  type UseOperatorAuditEventsResult,
} from "./hooks/useOperatorAuditEvents";

// Pages
export { PlatformOverviewPage } from "./pages/PlatformOverviewPage";
export { RegionsPage } from "./pages/RegionsPage";
export { RegionDetailPage } from "./pages/RegionDetailPage";
export { ProviderHealthPage } from "./pages/ProviderHealthPage";
export { CapacityPage } from "./pages/CapacityPage";
export { AccountsPage } from "./pages/AccountsPage";
export { AccountProjectsPage } from "./pages/AccountProjectsPage";
export { OperatorOperationsPage } from "./pages/OperatorOperationsPage";
export { OperatorAuditPage } from "./pages/OperatorAuditPage";
