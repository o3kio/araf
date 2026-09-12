# Araf #47 Operations Center evidence

This note records the P2.4 boundary: Araf's tenant Operations views are a
bounded projection of canonical O3K Operation list/show data. It does not build
the Operations Center as a second authority and does not implement governance,
audit, diagnostics or metering surfaces.

## Identity and authority

- Base main: `133a4b9ca4b8946fca524178ba7434b083aaf96f`
- Branch: `p2-4-operations-center`
- O3K dependency: `21fe687c387a04f107b6e87fac04060b1c28e449`
- O3K #907 is merged and complete. No new O3K API or contract change was
  designed or required.
- O3K owns operation identity, ownership, ordering, state and authorization.
  Araf owns only the normalized BFF model, filters, links and presentation.

## Operation runtime

`O3kAdapter` consumes `GET /o3k/v1/operations` and
`GET /o3k/v1/operations/{id}` using the server-held session token. Native
opaque cursors are never exposed as browser authority: the adapter performs a
bounded cursor walk (maximum 100 pages), detects missing/repeated cursors, and
maps the result to Araf's stable page DTO. Native ordering is preserved; the
adapter applies the existing filter DTO over the bounded stream because the
frozen O3K list contract intentionally exposes only `limit` and `cursor`.

Operation JSON identifiers accept the current structured O3K objects
(`ActionId`, `ResourceType`, `OwnershipScope`) and the historical string form,
then normalize to Araf strings. State/error/timestamp mapping remains
authoritative, with no fabricated completion timeline. Missing region metadata
is not inferred.

Mutations already return canonical O3K Operation IDs. Create/action panels,
resource details and the global Operations list link to the durable Operation
detail route; detail links back to the resource when a canonical target is
present. Polling stops at terminal states and reloads perform fresh BFF reads.

## Safety and boundaries

- The native O3K list/show routes enforce effective project scope and conceal
  foreign operations as not-found. Araf does not accept a browser project ID as
  authority and does not maintain an operation cache.
- OIDC/O3K credentials remain server-side; the browser receives only the
  normalized Operation DTO. Existing CSRF and bounded mutation-header/body
  protections remain active.
- Fixture operations remain available only through explicit fixture/test
  adapters. Production O3K discovery failures are surfaced as structured BFF
  errors and never trigger fixture fallback.
- Operator-wide audit/governance functionality is not substituted here; no
  unsupported global operator Operation endpoint is invented.

## Real O3K process evidence

`tests/p2-4-real-operations-process.sh` starts the real `o3kd` binary from the
O3K convergence checkout with the fake provider, token authentication and
native cursor HMAC configuration. The ignored integration test then drives the
real Araf tenant BFF: it creates a Compute resource, finds the resulting
durable Operation through the native list route, shows it, reads it again to
prove reload reconstruction, and cleans up the Network and Compute resources.
No fixture adapter, manually injected handler or test-only O3K route is used.

## Validation

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
tests/p2-4-real-operations-process.sh
git diff --check
```

The WireMock contract suite covers structured native operation identifiers,
bounded cursor propagation, filter mapping and operation-state/error mapping.
The Chromium suite covers global list/detail navigation, terminal polling and
reload survival in deterministic fixture mode. The real-process gate covers
the production adapter/router path described above.

Deferred/non-goals: a global Operations Center redesign, governance/quota/Audit
UI (#48), diagnostics (#49), metering (#50), provider lifecycle expansion
(#46), and OpenStack support.

Remaining correctness deviations: **None**.
