# M6 implementation prompt — first-class Operations UX and prototype gate

Implement **M6** after M3/M4 and integrate with M5 actions.

## Read first

- `AGENTS.md`
- `docs/architecture/overview.md`
- `docs/product/information-architecture.md`
- `docs/architecture/o3k-integration-contract.md`

## Goal

Make truthful async cloud operation handling a defining Araf behavior and complete the prototype architecture gate.

## Required implementation

1. Implement `packages/operations` around the typed canonical Operation contract.
2. Add global Operations list with state/action/scope/resource/time and bounded filtering.
3. Add Operation details:
   - opaque ID,
   - state,
   - requested action/scope/resource,
   - timestamps,
   - structured failure details,
   - correlation ID,
   - related resources.
4. Render a timeline only from explicit fixture/upstream events. Never invent orchestration steps.
5. Link operations from resource details and action completion flows.
6. Implement update transport behind an abstraction that supports fixture polling now and SSE/event source later. If SSE is implemented in fixture mode, keep reconnection/backoff bounded and tested.
7. Demonstrate accepted -> running -> succeeded and accepted -> running -> failed scenarios.
8. Ensure a successful HTTP submission produces `Request accepted` UX, never `Resource created successfully`, until the Operation reaches the server-defined success state.
9. Add deep-link/reload tests for Operation details.
10. Complete the prototype journeys in `docs/product/mvp-prototype.md` and record prototype evidence in `docs/engineering/prototype-evidence.md`.

## Prototype exit review

Explicitly assess whether generic resources/forms/operations are good enough to proceed. List any cases where hand-written UI was required and whether that indicates a descriptor-model defect.

## Acceptance

- no async flow lies about completion,
- operation state survives page reload via server fixture,
- global/resource operation navigation works,
- failure UX retains structured/correlation evidence,
- prototype tenant and operator journeys pass Playwright,
- prototype evidence document gives a clear GO/NO-GO recommendation for M7.

Branch suggestion: `m6-operations-prototype-gate`.
