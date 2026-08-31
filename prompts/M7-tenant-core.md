# M7 implementation prompt — real O3K tenant core-service integration

Implement **M7** only after the M6 prototype gate is GO.

## Read first

- `AGENTS.md`
- `docs/engineering/prototype-evidence.md`
- `docs/architecture/o3k-integration-contract.md`
- `docs/product/mvp-prototype.md`
- current O3K native API/specification and relevant upstream tests before writing adapters.

## Goal

Bind the proven generic runtime to real supported O3K native contracts for a representative tenant cloud-service set without introducing compatibility/provider jargon.

## Required implementation

1. Inspect the current O3K repository/API at implementation time. Do not rely on historical endpoint assumptions from planning.
2. Implement production BFF adapters only for confirmed native capabilities, prioritizing:
   - Compute Server,
   - Network,
   - Volume,
   - Image selection/read where supported,
   - required related resources/endpoints.
3. Prefer generated/mechanically derived API clients/types from authoritative specs where possible.
4. Map native ServiceManifest/resource metadata into the generic runtime; where upstream metadata is insufficient, document the exact gap rather than permanently hard-code a second resource contract.
5. Support list/show and only those mutations whose upstream Operation/auth semantics are confirmed.
6. Preserve canonical resource IDs, scope, Problem Details and Operation IDs.
7. Add integration tests against a deterministic O3K test environment or recorded contract harness. Fixture fallback is allowed for developer/demo mode but never masquerades as production success.
8. Verify provider implementation names are absent from ordinary tenant UX unless O3K intentionally exposes a customer-facing offering attribute.
9. Add upstream gaps to `docs/engineering/upstream-gaps.md`.

## Stop condition

If a required safe production contract does not exist upstream, do not invent it in the BFF. Mark that acceptance item blocked by an explicit O3K dependency while completing the remaining safe subset.

## Acceptance

- representative real O3K resources render through the same generic runtime used by fixtures,
- at least one real mutation returns/tracks canonical O3K Operation if supported upstream,
- no OpenStack service names/provider internals leak into native tenant UX,
- scope isolation tests exist,
- fixture/production adapter separation is explicit.

Branch suggestion: `m7-o3k-tenant-core`.
