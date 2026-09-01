// Client provider
export {
  GovernanceClientProvider,
  useGovernanceClient,
  type GovernanceClientProviderProps,
} from "./client/context";

// Capabilities
export { hasCapability } from "./capabilities";

// Hooks
export { useProjects, type UseProjectsResult, type UseProjectsOptions } from "./hooks/useProjects";
export { useProject, type UseProjectResult } from "./hooks/useProject";
export { useUsers, type UseUsersResult, type UseUsersOptions } from "./hooks/useUsers";
export { useUser, type UseUserResult } from "./hooks/useUser";
export { useRoles, type UseRolesResult, type UseRolesOptions } from "./hooks/useRoles";
export { useQuotas, type UseQuotasResult, type UseQuotasOptions } from "./hooks/useQuotas";
export { useUsage, type UseUsageResult, type UseUsageOptions } from "./hooks/useUsage";
export {
  useAuditEvents,
  type UseAuditEventsResult,
  type UseAuditEventsOptions,
} from "./hooks/useAuditEvents";
export {
  useApiCredentials,
  type UseApiCredentialsResult,
  type UseApiCredentialsOptions,
} from "./hooks/useApiCredentials";
export {
  useCreateApiCredential,
  type UseCreateApiCredentialResult,
} from "./hooks/useCreateApiCredential";
export {
  useDeleteApiCredential,
  type UseDeleteApiCredentialResult,
} from "./hooks/useDeleteApiCredential";

// Pages
export { ProjectsPage } from "./pages/ProjectsPage";
export { ProjectDetailPage } from "./pages/ProjectDetailPage";
export { UsersPage } from "./pages/UsersPage";
export { UserDetailPage } from "./pages/UserDetailPage";
export { QuotasPage } from "./pages/QuotasPage";
export { UsagePage } from "./pages/UsagePage";
export { AuditPage } from "./pages/AuditPage";
export { ApiCredentialsPage } from "./pages/ApiCredentialsPage";
