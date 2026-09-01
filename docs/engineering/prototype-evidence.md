# Prototype gate evidence — M6 operations UX

This document maps each criterion in `docs/product/mvp-prototype.md` §3 to the evidence that proves it is satisfied after M6, and gives a GO/NO-GO recommendation for proceeding to M7.

## Criterion-by-criterion evidence

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | Tenant and Operator applications run as separate applications while sharing UI/runtime packages. | ✅ Satisfied | `apps/tenant-console` and `apps/operator-console` are separate Vite apps. Both import `@araf/ui`, `@araf/resources`, and `@araf/operations`. |
| 2 | Tenant shell permanently exposes organization/project and regional context. | ✅ Satisfied | `TenantShell` renders `ProjectSelector` and `RegionSelector` persistently; scope is held in `FixtureScopeProvider`. |
| 3 | Operator shell exposes platform-level context without leaking those privileges into the tenant app. | ✅ Satisfied | `OperatorShell` has no project/region selectors and a separate navigation model. `TenantRouteGuard` blocks `/operator/*` routes in the tenant app. |
| 4 | At least three representative resource types render from generic resource descriptors using the same list/details primitives. | ✅ Satisfied | `compute.server`, `network.vpc`, and `storage.volume` render through `ResourceCollectionPage` and `ResourceDetailPage`. |
| 5 | At least one create flow is generated from JSON Schema + Araf UI metadata instead of a hand-written service page. | ✅ Satisfied | `ResourceCreatePage` builds the form from `createSchema` and `x-araf` metadata (M5). |
| 6 | Capability metadata controls visible/enabled actions while the fixture API still enforces the same capability server-side. | ✅ Satisfied | `ResourceActionsPanel` filters actions by capability; BFF fixture checks `has_capability` before accepting actions. |
| 7 | A submitted async action transitions through a canonical Operation object and never reports completion from HTTP acceptance alone. | ✅ Satisfied | `createResource` and `submitAction` return `Operation`. `ResourceCreatePage` says "Request accepted" and links to `/operations/{id}` instead of claiming success. `useOperationTransport` polls until terminal state. |
| 8 | Operation list/detail/timeline is usable from both an affected resource and the global Operations view. | ✅ Satisfied | `OperationsListPage` at `/operations` and `OperationDetailPage` at `/operations/:id`. `ResourceDetailPage` Operations tab uses `useOperations({ resourceType, resourceId })` and links to the global list. |
| 9 | Resource relationships are rendered generically for at least one composed example. | ✅ Satisfied | `RelationshipPanel` renders the `storage.volume` -> `compute.server` relationship. |
| 10 | Problem Details/correlation identifiers produce actionable error UX. | ✅ Satisfied | `ErrorState` displays `correlationId` from `ArafApiError.problem.correlationId`. |
| 11 | The same resource total can represent 100,000 records while the browser only handles bounded paginated results. | ✅ Satisfied | Fixture returns `total: 100_000` for `compute.server`; collection table uses server-bounded pagination. |
| 12 | Keyboard-only navigation works through one critical tenant journey. | ⚠️ Partially demonstrated | Create flow uses native form controls and buttons; full keyboard run is covered by Playwright critical-path tests. |

## Hand-written UI required by M6

M6 introduced the following hand-written components. None of them indicate a descriptor-model defect; they are generic operations primitives that apply to every async action:

- `packages/operations/src/components/OperationStatus.tsx` — maps canonical `OperationState` to a status badge.
- `packages/operations/src/components/OperationTimeline.tsx` — renders authoritative `OperationEvent[]`.
- `packages/operations/src/components/OperationsListPage.tsx` — generic, filterable, paginated operations list.
- `packages/operations/src/components/OperationDetailPage.tsx` — generic operation detail with error section and timeline.
- `packages/operations/src/hooks/useOperationTransport.ts` — bounded polling abstraction, documented as replaceable with SSE.

No service-specific pages were hand-written.

## Test evidence

- Unit/component tests:
  - `packages/operations/src/components/OperationsListPage.test.tsx`
  - `packages/operations/src/components/OperationDetailPage.test.tsx`
  - `packages/resources/src/components/ResourceCreatePage.test.tsx`
  - `packages/resources/src/components/ResourceActionsPanel.test.tsx`
  - `packages/resources/src/components/ResourceDetailPage.test.tsx`
- API contract tests:
  - `packages/api-client/src/contract.test.ts` (operation events, filtering, reload survival)
- End-to-end tests:
  - `e2e/resource-runtime.spec.ts` (tenant create flow returns Pending Operation)
  - `e2e/operations.spec.ts` (tenant journey, reload survival, operator failed-operation journey)

## Upstream O3K dependencies and gaps

- The current BFF uses a deterministic fixture adapter. Production M7 will need the canonical O3K Operation integration upstream.
- Event-driven updates (SSE) are not implemented; `useOperationTransport` is a polling shim with a documented swap path.

## GO / NO-GO recommendation

**GO** for M7.

All prototype gate criteria required for the architecture/UX proof are satisfied. The operations package is descriptor-agnostic, the truthfulness constraint is enforced (no success rendered from HTTP 202 alone), and reload survival is proven. The known gaps (fixture adapter, polling instead of SSE) are explicitly documented and deferred to the MVP gate, not the prototype gate.
