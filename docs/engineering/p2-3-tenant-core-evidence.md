# Araf #46 tenant core API closure evidence

This note records the P2.3 implementation boundary. It closes the generic
tenant lifecycle integration for capabilities advertised by O3K; Operations
Center UX remains #47.

## Identity and authority

- Base main: `5e54fe5118d03f69e70a9af036626fc970599dfa`
- Branch: `p2-3-tenant-core-api`
- O3K dependency: `21fe687c387a04f107b6e87fac04060b1c28e449`
- No new O3K API was designed or required. Araf consumes the frozen native
  CRUD, action, relationship, Operation, quota and discovery contracts.
- Scope/AuthContext remains server-side and is never taken from browser route,
  query or body fields. O3K remains semantic and authorization authority;
  Araf owns only normalized view models and presentation metadata.

## Runtime coverage

The BFF generic resource route now supports bounded list/show plus PUT and
DELETE mutation paths. O3K native collection/instance routes are selected from
current resource-type discovery for Compute, Network, Volume and Image (when
advertised and ready). Mutations carry bounded idempotency keys, update
generation preconditions, and return the canonical O3K Operation. Action-panel
dispatch uses the same discovered action map; delete/update cannot be reached
through an undiscovered capability.

The normalized resource model retains O3K `metadata.generation`. Operation
states, correlation IDs and structured upstream errors remain authoritative;
the fixture adapter returns explicit not-implemented responses for update and
does not become a production fallback.

## Safety and reliability

- Server-side credential custody and CSRF middleware remain unchanged.
- Mutation headers are bounded and ASCII-only; request bodies remain subject to
  the existing BFF limit.
- Pagination remains cursor-based and bounded (Araf does not enumerate all
  pages to render the first page).
- O3K quota/conflict/forbidden/not-found responses pass through the structured
  Problem Details model; no optimistic resource is invented.
- Discovery-derived capability projection is UI-only. Every mutation is
  re-authorized and capability-checked by O3K on dispatch.

## Validation

The following gates passed on the final implementation candidate:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-features
pnpm format:check
pnpm lint
pnpm typecheck
pnpm test
pnpm build
pnpm e2e
git diff --check
```

WireMock contract coverage proves native create/delete/update request
translation, generation preconditions, Operation correlation and scope
isolation. A real `o3kd` process from the O3K convergence checkout was started
with the fake provider and password-issued native token; `/healthz`,
`/o3k/v1/services`, `/o3k/v1/resource-types` and `/o3k/v1/identity/me` were
read through the production O3K routes. The observed profile advertised
Compute, Network and Image as ready, while Volume was truthfully `not_ready`;
Araf therefore exposed only the supported discovered subset and did not
fabricate a Volume route.

## Deferred and non-goals

Full provider lifecycle breadth beyond advertised contracts, Operations Center
UX (#47), governance/quota/Audit UI (#48), diagnostics (#49), metering (#50),
and OpenStack support remain outside #46.

Remaining correctness deviations: **None**.
