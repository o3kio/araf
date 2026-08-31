# M0 implementation prompt — repository bootstrap and deterministic quality gates

You are implementing **M0** for Araf, the O3K cloud console.

## Read first

- `AGENTS.md`
- `docs/product/mvp-prototype.md`
- `docs/architecture/overview.md`
- `docs/engineering/quality-gates.md`
- ADRs under `docs/adr/`

## Goal

Create the smallest production-quality mixed TypeScript/Rust workspace that can support two console applications, shared packages and two Rust BFF binaries without prematurely implementing product features.

## Required implementation

1. Establish a pnpm workspace with:
   - `apps/tenant-console`
   - `apps/operator-console`
   - `packages/ui`
   - placeholder packages for shell/api-client/resources/schema-runtime/operations/fixtures only where needed to prove workspace boundaries.
2. Create minimal React + TypeScript + Vite apps that render distinct tenant/operator bootstrap pages.
3. Establish a Rust workspace under `backend/` with:
   - shared BFF core crate,
   - tenant BFF binary,
   - operator BFF binary.
   Minimal health endpoints are enough.
4. Pin currently supported Node/pnpm/Rust toolchains after verifying ecosystem compatibility. Document the choice; do not blindly use versions from this prompt.
5. Configure strict TypeScript, formatting, linting, unit-test infrastructure and production builds.
6. Configure Rust fmt/clippy/check/test gates with warnings denied in CI.
7. Add deterministic CI covering both ecosystems.
8. Add repository scripts/commands documented in README/CONTRIBUTING-style developer notes.
9. Ensure tenant and operator application builds do not accidentally import each other.

## Non-goals

- no authentication,
- no Cloudscape/resource pages yet,
- no O3K API calls,
- no Redux/global state framework without evidence,
- no Docker/Kubernetes deployment stack beyond what is needed for a minimal smoke test.

## Acceptance

- fresh checkout can install/build/test using documented commands,
- both Vite apps run independently,
- both Rust BFF binaries run independently,
- TypeScript is strict and CI fails on type/lint/test errors,
- Rust fmt/clippy/check/test passes,
- workspace boundaries match ADR 0001,
- no cloud tokens or fake cloud APIs have been introduced.

Use branch suggestion: `m0-repository-bootstrap`.
