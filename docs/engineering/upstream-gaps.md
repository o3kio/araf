# Upstream O3K gaps

This file records required O3K capabilities that are not yet confirmed by the authoritative upstream API. Each entry describes the contract Araf needs, why a console-only workaround would be unsafe or incorrect, the Araf feature that is blocked without it, and the acceptable fallback for the current phase.

Implementation phases must add new gaps here instead of inventing production O3K endpoints, URL paths, status codes, or payload fields.

## M3-O3K-001: Session and token introspection contract

- **Gap id:** `M3-O3K-001`
- **Required O3K contract:** An authoritative O3K session/token introspection contract that the Rust BFF can use to validate the browser's opaque session and resolve the caller's identity, active organization, project, region, and token expiry.
- **Why Araf M3 needs it:** `Upstream::context` returns `SessionContext`, which is the source of truth for scope selection, capability evaluation, and cross-scope rejection. Without an upstream contract, the BFF cannot distinguish valid sessions from guessed session identifiers or derive the correct authorization context.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Production authentication adapter, real session lifecycle, and tenant/operator session separation.
- **Acceptable fallback:** Continue returning a deterministic fixture `SessionContext` from the fixture adapter for M3-M6. Do not invent a production token introspection endpoint.

## M3-O3K-002: OIDC/OAuth token and logout contract

- **Gap id:** `M3-O3K-002`
- **Required O3K contract:** O3K-owned OIDC/OAuth endpoints for authorization-code exchange, token refresh, and logout/session revocation, discoverable and usable by a confidential Rust BFF client.
- **Why Araf M3 needs it:** ADR 0002 places the BFF in the confidential OIDC client role so that the browser never holds reusable cloud tokens. Implementing this without an authoritative O3K identity contract would make the BFF a second identity provider.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Production login, logout, session rotation, and server-side token storage.
- **Acceptable fallback:** Fixture adapter with no OIDC exchange; defer production auth integration until the O3K identity contract is available.

## M3-O3K-003: Canonical Operation representation and lifecycle contract

- **Gap id:** `M3-O3K-003`
- **Required O3K contract:** A canonical O3K Operation representation with stable identity, action/type, state machine (pending, running, succeeded, failed/cancelled), timestamps, structured failure reason, requested scope/resource, and correlation/request identifiers.
- **Why Araf M3 needs it:** `Upstream::submit_action` must return an Operation, and `list_operations`/`get_operation` must surface Operation state. The frontend must never infer completion from an accepted HTTP response; it must observe authoritative Operation state.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Real asynchronous action tracking, operations polling, and operation detail pages.
- **Acceptable fallback:** Continue returning deterministic fixture `Operation` objects for M3-M6. Polling may be used initially if no event/SSE contract exists, but only against a real Operation contract when available.

## M3-O3K-004: Paginated resource list contract with server-side filtering and sorting

- **Gap id:** `M3-O3K-004`
- **Required O3K contract:** Server-bounded resource collection endpoints for each supported resource type (e.g., Compute Server, Network, Volume) that accept page/cursor bounds and return stable totals, continuation, and support server-side filtering and sorting.
- **Why Araf M3 needs it:** `Upstream::list_resources` returns a `PaginatedCollection<Resource>` and the quality gates forbid fetching an unbounded inventory into the browser. Araf cannot safely emulate pagination by downloading all resources and filtering client-side.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Production resource grids, search, and filter surfaces for all MVP resource types.
- **Acceptable fallback:** Fixture adapter returns bounded deterministic pages with synthetic totals for M3-M6.

## M3-O3K-005: Single resource detail contract

- **Gap id:** `M3-O3K-005`
- **Required O3K contract:** An authoritative per-resource-type detail contract that returns a resource by its opaque stable identifier and rejects cross-scope access at the upstream layer.
- **Why Araf M3 needs it:** `Upstream::get_resource` fetches a single `Resource` by id. The BFF must not manufacture resource state or rely on client-side scope checks alone; cross-scope rejection must be enforced upstream.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Production resource detail pages and relationship drill-down.
- **Acceptable fallback:** Fixture adapter returns deterministic resources and rejects unknown identifiers.

## M3-O3K-006: ServiceManifest and service descriptor discovery contract

- **Gap id:** `M3-O3K-006`
- **Required O3K contract:** An O3K ServiceManifest or equivalent discovery contract that describes available services, resource types, list columns, filters, details fields, supported actions, input schemas, required capabilities, and relationships.
- **Why Araf M3 needs it:** `Upstream::services` returns `ServiceDescriptor` and the architecture requires normal service UX to be descriptor/schema-driven. Araf must not hard-code provider-specific resource models or invent service topology.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Dynamic, schema-driven service catalog and resource pages.
- **Acceptable fallback:** Continue using committed fixture descriptors under `contracts/fixtures` for M3-M6.

## M3-O3K-007: Capability evaluation contract

- **Gap id:** `M3-O3K-007`
- **Required O3K contract:** An authoritative capability or permission evaluation contract (or evaluated capabilities embedded in session/resource responses) that tells the BFF which actions the caller may perform on which resource types.
- **Why Araf M3 needs it:** `SessionContext::capabilities` drives action visibility and enablement. The security model forbids inferring authorization from role names alone, and server-side authorization is mandatory on every authoritative operation.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Server-authoritative action enablement/hiding and capability-based UI gating.
- **Acceptable fallback:** Fixture adapter returns a fixed list of capabilities for the fixture user.

## M3-O3K-008: Action submission contract

- **Gap id:** `M3-O3K-008`
- **Required O3K contract:** Authoritative per-resource-type mutation contracts for each supported action (create, delete, etc.) that accept action inputs and return a canonical Operation.
- **Why Araf M3 needs it:** `Upstream::submit_action` accepts an `ActionRequest` and must translate it into an authoritative cloud mutation. The BFF must not become a second cloud-control API by inventing mutation semantics or resource state transitions.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Production create/delete/update actions on resources.
- **Acceptable fallback:** Fixture action adapter accepts known action ids and returns deterministic Operations.

## M3-O3K-009: Structured Problem Details and error contract

- **Gap id:** `M3-O3K-009`
- **Required O3K contract:** An upstream Problem Details or structured error contract that preserves stable machine-readable type/code, human-safe title/detail, correlation identifier, operation/resource linkage, and explicit retryability.
- **Why Araf M3 needs it:** The BFF must translate upstream errors into `ApiError` without parsing human-readable strings or inventing error codes. Retryability, authorization decisions, and user messaging must be grounded in upstream semantics.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Accurate, safe error surfaces and retry behavior in production.
- **Acceptable fallback:** Fixture adapter returns deterministic errors with the same problem-detail shape used by tests.

## M3-O3K-010: Scope enumeration contract

- **Gap id:** `M3-O3K-010`
- **Required O3K contract:** Authoritative contracts to enumerate organizations, projects, and regions available to the caller, including canonical identifiers and display names.
- **Why Araf M3 needs it:** `SessionContext` and `Resource` carry organization, project, and region identifiers. The UI needs a scope switcher and the BFF must validate that resource requests belong to the active scope.
- **Current status:** Not confirmed.
- **Blocked Araf feature:** Production scope switcher and cross-scope validation surfaces.
- **Acceptable fallback:** Fixture adapter uses fixed scope identifiers (`org-fixture`, `project-fixture`, `global`).

## M3-O3K-011: Audit, metering, and usage contract

- **Gap id:** `M3-O3K-011`
- **Required O3K contract:** O3K audit/metering endpoints or event streams that expose usage, quota, and audit records linked to canonical Operation and resource identities.
- **Why Araf M3 needs it:** Operations must be reachable globally and from related resources, and future MVP milestones (M11-M12) require usage/audit evidence. Araf must not fabricate billing, quota, or audit records.
- **Current status:** Not available for M3.
- **Blocked Araf feature:** Usage dashboards, quota surfaces, and long-term audit trails.
- **Acceptable fallback:** Defer to M11/M12; fixture `Operation` objects provide synthetic correlation identifiers for UI development only.

## M7-O3K-001: Native OpenAPI specification for `/o3k/v1`

- **Gap id:** `M7-O3K-001`
- **Required O3K contract:** A published OpenAPI or protobuf-based service definition for the native `/o3k/v1` HTTP surface so Araf can generate types and keep the BFF adapter in sync with authoritative O3K routes.
- **Why Araf M7 needs it:** M7 requires a real O3K tenant adapter. The native routes are defined only in `crates/o3k-api/src/lib.rs` (`/o3k/v1/services`, `/o3k/v1/resource-types`, `/o3k/v1/compute/servers`, `/o3k/v1/operations/{id}`, generic `/{namespace}/{collection}`). Only `contracts/openapi/bootstrap.yaml` exists and it does not cover the native surface.
- **Current status:** Confirmed missing.
- **Blocked Araf feature:** Generated native API client; guaranteed route/payload parity with O3K main.
- **Acceptable fallback:** Hand-write a narrow O3K native client in the BFF based on direct inspection of `crates/o3k-api`, `crates/o3k-native-api`, and `contracts/native-resource-envelope-v1.schema.json`. Document every manually-derived type and keep the surface minimal.

## M7-O3K-002: Operation list/search endpoint

- **Gap id:** `M7-O3K-002`
- **Required O3K contract:** A server-bounded `GET /o3k/v1/operations` endpoint that returns operations visible to the caller, filterable by state, action, resource type, resource id, scope, and time bounds.
- **Why Araf M7 needs it:** Araf M6 built a global Operations list page (`OperationsListPage`) and the prototype gate requires global/resource operation navigation. The O3K native API only exposes `GET /o3k/v1/operations/{id}`.
- **Current status:** Confirmed missing.
- **Blocked Araf feature:** Global Operations list backed by real O3K data.
- **Acceptable fallback:** The M7 `O3kAdapter::list_operations` explicitly returns `501 Not Implemented` with a clear Problem Details message. Keep the global Operations list as a fixture-only page in M7 and document the gap; operation detail pages can use real `GET /o3k/v1/operations/{id}`.

## M7-O3K-003: Operation event timeline

- **Gap id:** `M7-O3K-003`
- **Required O3K contract:** An authoritative Operation event stream or `events` array on the canonical `Operation` model that records state transitions, progress messages, and failure details with timestamps and correlation IDs.
- **Why Araf M7 needs it:** Araf M6 renders an `OperationTimeline` from `OperationEvent[]`. O3K's canonical `Operation` (`crates/o3k-kernel/src/operation.rs`) only carries `created_at`, `started_at`, `finished_at`, and a single `error` string.
- **Current status:** Confirmed missing.
- **Blocked Araf feature:** Rich, truthful operation progress timeline backed by O3K.
- **Acceptable fallback:** Implemented in M7: the BFF derives a minimal `events` array from O3K timestamps and `error`, and the frontend `OperationTimeline` falls back to the same derivation when the BFF returns no events. No intermediate orchestration steps are invented; the derivation is surfaced as presentation translation, not authoritative events.

## M7-O3K-004: Native resource list filtering and sorting

- **Gap id:** `M7-O3K-004`
- **Required O3K contract:** Native list endpoints that support server-side filtering by name/status/scope and stable sorting, returning bounded pages plus a total count.
- **Why Araf M7 needs it:** Araf's generic `ResourceCollectionPage` exposes descriptor-driven filters and sortable columns. The native `/o3k/v1` surface uses cursor-based pagination (`limit`/`cursor`) with HMAC-authenticated opaque cursors, default limit 50, max 200, and no field filtering or sorting.
- **Current status:** Confirmed missing.
- **Blocked Araf feature:** Server-bounded filtering/sorting in real O3K-backed resource grids.
- **Acceptable fallback:** Implemented in M7: the adapter maps Araf `page`/`pageSize` to O3K `limit` cursor pagination, ignores unsupported filter/sort params, and returns a synthetic `total` (`items.len() + 1` when O3K provides a `next_cursor`). Continue using the fixture adapter to demonstrate full server-side filtering/sorting behavior.

## M7-O3K-005: Native create/delete for volume and network

- **Gap id:** `M7-O3K-005`
- **Required O3K contract:** Concrete native routes for creating and deleting volumes and address-realms/networks that return canonical Operations.
- **Why Araf M7 needs it:** M7 targets Compute, Network, and Volume. `crates/o3k-api/src/lib.rs` mounts only `list`/`show` for `/o3k/v1/volume/volumes` and `/o3k/v1/network/address-realms`. Create/delete must go through the generic manifest-driven `/{namespace}/{collection}` path, which requires correct `kind` and manifest registration.
- **Current status:** Confirmed partially — concrete compute server create/delete routes exist; volume/network concrete routes are read-only and the generic manifest path is not validated end-to-end for Araf's use case.
- **Blocked Araf feature:** Direct, type-safe create/delete UX for volume and network through the native API.
- **Acceptable fallback:** Implement compute server create/delete as the primary real mutation in M7. Keep volume/network create/delete behind the fixture adapter for the prototype/MVP boundary until the native generic path is validated end-to-end. See also M7-O3K-008 for compute server start/stop.

## M7-O3K-008: Native compute server start/stop routes

- **Gap id:** `M7-O3K-008`
- **Required O3K contract:** Concrete native routes for starting and stopping compute servers that return canonical Operations (e.g. `POST /o3k/v1/compute/servers/{id}/start` and `/stop`, or a generic action submission path).
- **Why Araf M7 needs it:** Araf's descriptor for `compute.server` advertises `start` and `stop` actions. O3K's manifest defines `compute:StartServer` and `compute:StopServer` actions, but the native HTTP surface (`crates/o3k-api/src/lib.rs`) binds only `GET` and `DELETE` to `/o3k/v1/compute/servers/{id}`. There is no native start/stop endpoint.
- **Current status:** Confirmed missing.
- **Blocked Araf feature:** Real O3K-backed start/stop actions in the tenant console.
- **Acceptable fallback:** The M7 `O3kAdapter` returns `501 Not Implemented` for `start`/`stop` actions with a clear Problem Details message that references this gap. Users can still perform create/delete on compute servers. Start/stop remains available in the fixture adapter.

## M7-O3K-006: Multi-region and availability zone enumeration

- **Gap id:** `M7-O3K-006`
- **Required O3K contract:** Dynamic endpoint to enumerate regions and availability zones available to the caller.
- **Why Araf M7 needs it:** The tenant shell has a region selector. O3K's registry hard-codes `RegionOne` (`crates/o3k-kernel/src/registry.rs`) and there is no live enumeration API.
- **Current status:** Confirmed missing.
- **Blocked Araf feature:** Live region/availability-zone selection.
- **Acceptable fallback:** Continue using the fixture/static region list in the console for M7; document that region selection is presentation-only and O3K currently operates in a single region.

## M7-O3K-007: IAM management endpoints

- **Gap id:** `M7-O3K-007`
- **Required O3K contract:** O3K endpoints to manage users, projects, roles, domains, and organization membership.
- **Why Araf M7 needs it:** Future M8 governance work and operator platform work require project/user management. O3K identity routes are limited to token issue/validate/check and native `/identity/me`.
- **Current status:** Confirmed missing.
- **Blocked Araf feature:** Tenant governance and operator identity management surfaces.
- **Acceptable fallback:** Keep governance pages as placeholders/fixtures for M7; defer to M8 after confirming the upstream IAM contract.
