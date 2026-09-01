// Client provider
export {
  ResourceClientProvider,
  useResourceClient,
  type ResourceClientProviderProps,
} from "./client/context";

// Core runtime primitives
export { validateDescriptor, type ResourceDescriptor } from "./descriptor";
export { getResourceField, formatResourceField } from "./fields";
export { mapResourceStatus } from "./status";

// Hooks
export {
  useResourceDescriptor,
  type UseResourceDescriptorResult,
} from "./hooks/useResourceDescriptor";
export {
  useResourceCollection,
  type CollectionQuery,
  type UseResourceCollectionResult,
} from "./hooks/useResourceCollection";
export { useResourceDetail, type UseResourceDetailResult } from "./hooks/useResourceDetail";
export { useCreateResource, type UseCreateResourceResult } from "./hooks/useCreateResource";
export { useResourceAction, type UseResourceActionResult } from "./hooks/useResourceAction";
export { useCapabilities, type UseCapabilitiesResult } from "./hooks/useCapabilities";
export { useServiceCatalog, type UseServiceCatalogResult } from "./hooks/useServiceCatalog";

// Components
export {
  ResourceCollectionPage,
  type ResourceCollectionPageProps,
} from "./components/ResourceCollectionPage";
export { ResourceDetailPage, type ResourceDetailPageProps } from "./components/ResourceDetailPage";
export { RelationshipPanel, type RelationshipPanelProps } from "./components/RelationshipPanel";
export { ResourceLandingPage } from "./components/ResourceLandingPage";
export { ResourceCreatePage, type ResourceCreatePageProps } from "./components/ResourceCreatePage";
export {
  ResourceActionsPanel,
  type ResourceActionsPanelProps,
} from "./components/ResourceActionsPanel";
export { ServiceCatalogPage } from "./components/ServiceCatalogPage";
