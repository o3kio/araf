// Client provider
export {
  OperationsClientProvider,
  useOperationsClient,
  type OperationsClientProviderProps,
} from "./client/context";

// Types
export type {
  Operation,
  OperationState,
  OperationEvent,
  OperationError,
  PaginatedCollection,
  ListOperationsQuery,
} from "./types";

// Hooks
export { useOperations, type UseOperationsResult } from "./hooks/useOperations";
export { useOperation, type UseOperationResult } from "./hooks/useOperation";
export {
  useOperationTransport,
  startPolling,
  stopPolling,
  type OperationTransport,
} from "./hooks/useOperationTransport";

// Components
export { OperationStatus, type OperationStatusProps } from "./components/OperationStatus";
export { OperationTimeline, type OperationTimelineProps } from "./components/OperationTimeline";
export { OperationsListPage } from "./components/OperationsListPage";
export { OperationDetailPage } from "./components/OperationDetailPage";
