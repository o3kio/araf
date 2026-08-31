# M4 implementation prompt — generic resource list/detail/relationship runtime

Implement **M4** after M1, M2 and M3.

## Read first

- `AGENTS.md`
- `docs/architecture/resource-runtime.md`
- `docs/product/information-architecture.md`
- `docs/architecture/o3k-integration-contract.md`
- ADR 0003

## Goal

Prove that Araf can render high-quality cloud resources from descriptors rather than hard-coded per-service screens.

## Required implementation

1. Define a versioned frontend descriptor schema for **presentation/runtime needs only**; do not redefine O3K cloud semantics.
2. Implement generic resource collection page:
   - bounded server pagination,
   - filters/sorting supported by fixture contract,
   - status display,
   - loading/empty/error states,
   - bookmarkable filters where sensible.
3. Implement generic resource details page:
   - overview/configuration sections,
   - opaque resource ID,
   - lifecycle/health presentation from descriptor/server data,
   - relationship section,
   - Operations link/placeholder.
4. Implement generic relationship rendering with scope-safe fetch behavior.
5. Prove the runtime with at least three meaningfully different fixture resource types. At least one must have parent/child relationships.
6. Ensure unknown/unsupported descriptor fields fail safely and visibly in development rather than silently executing behavior.
7. Preserve server Problem Details and correlation IDs in errors.
8. Add component/contract/Playwright tests covering the generic flow.

## Critical constraint

Do **not** implement three separate Compute/Network/Volume pages and call that generic. The same runtime component path must render the representative resources from descriptors.

## Non-goals

- create/edit forms (M5),
- real O3K services (M7),
- custom topology/graph editor,
- arbitrary descriptor scripts.

## Acceptance

- three resource types render without service-specific page components,
- pagination is server bounded,
- relationship navigation preserves scope,
- untrusted resource strings are safely rendered,
- Problem Details/correlation UX works,
- generic runtime is demonstrably usable at prototype quality.

Branch suggestion: `m4-generic-resource-runtime`.
