# Production release evidence

This is the current evidence record for the Araf production-maturity program.
It is intentionally maintained before release-candidate acceptance so that
green repository tests are not confused with production readiness.

## Candidate

- Branch: `main`
- Release SHA under assessment: `f0365b8`
- Date: 2026-09-02
- Advertised production profiles: none; O3K integration is not yet accepted
- OpenStack production support: not advertised

## Gate results

| Gate | Result | Evidence |
| --- | --- | --- |
| Frontend typecheck | PASS | `pnpm -r --if-present typecheck` |
| Frontend unit/component tests | PASS | `pnpm test` |
| Rust format | PASS | `cargo fmt --all -- --check` |
| Rust clippy | PASS | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Rust check | PASS | `cargo check --workspace --all-targets --all-features` |
| Rust tests | PASS | `cargo test --workspace --all-features` |
| Browser CI wiring | PASS | Required `browser E2E (Chromium)` check is configured in `.github/workflows/ci.yml` |
| Main protection | PASS | `docs/production/main-protection.md`; live branch policy verified |
| Production HTTPS/config boundary | PARTIAL | Reference TLS proxy and fail-closed adapter configuration exist |
| Real O3K production E2E | NOT RUN | No accepted O3K identity/provider environment is available |
| OpenStack parity E2E | NOT APPLICABLE | OpenStack is not advertised |

## Verified security evidence

- Fixture adapters are rejected in production configuration.
- OIDC configuration is surface-specific and required in production.
- Production middleware does not inject fixture identity.
- Session cookies are opaque, `HttpOnly`, `Secure`, surface-bound and
  validated server-side.
- Callback state is short-lived and single-use.
- Cross-surface session-cookie isolation has regression coverage.
- Production CORS is restricted to the configured surface-specific console
  origin.
- Browser E2E, accessibility, CSRF and structured backend contract tests are
  present in the repository gates.

## Outstanding deviations and blockers

- The authoritative O3K identity contract does not currently provide a
  verified browser OIDC authorization-code, refresh, nonce/ID-token
  validation, or browser-session revocation flow for Araf to integrate.
- The production O3K adapter still has explicitly unsupported capabilities;
  no real Tenant or Operator production journey has been executed.
- Session persistence is in-memory and is not suitable for multi-replica
  production failover.
- Production origin allowlisting, readiness/metrics, load, multi-replica,
  upgrade/rollback, artifact provenance and pilot/soak evidence remain open.

## Decision

NO-GO — NOT PRODUCTION READY
