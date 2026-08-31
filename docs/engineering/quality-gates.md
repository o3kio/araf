# Engineering quality gates

## 1. Principle

Araf is control-plane software. "The page works" is not sufficient evidence. Every phase must leave deterministic build, test, security and accessibility evidence.

## 2. Workspace baseline

M0 must pin and document supported toolchains rather than relying on developer-global state.

Expected shape:

- pnpm workspace for frontend/packages,
- TypeScript strict mode,
- React + Vite applications,
- Rust workspace for BFF crates,
- lockfiles committed,
- reproducible CI commands,
- no network-loaded runtime dependencies.

Exact Node/Rust versions should be pinned to currently supported stable/LTS toolchains during M0 after checking ecosystem compatibility.

M0 pins (verified against ecosystem compatibility at implementation time):

- Node.js **22 LTS** (22.23.2 via `.nvmrc`; Node 20 reached end of life in April 2026),
- pnpm **10.14.0** via Corepack (`packageManager`),
- Rust **1.95.0** stable via `rust-toolchain.toml` with `rustfmt`/`clippy` components,
- TypeScript ~5.9, Vite 7, Vitest 3, React 19, axum 0.8/tokio 1 — locked by `pnpm-lock.yaml`/`Cargo.lock`.

## 3. Required CI categories

Every PR should run applicable gates:

### Frontend
- format check,
- lint,
- TypeScript typecheck,
- unit/component tests,
- production build.

### Rust
- `cargo fmt --all -- --check`,
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
- `cargo check --workspace --all-targets --all-features`,
- `cargo test --workspace --all-features`.

### End to end
- Playwright critical paths,
- Chromium + Firefox + WebKit coverage for release-critical flows by MVP gate.

### Contract/security
- descriptor/schema validation tests,
- cross-scope negative tests,
- security-header/session tests,
- dependency scanning/SBOM generation when packaging is introduced.

## 4. Test pyramid

### Unit
Pure formatting/state/schema helpers.

### Component
Resource table/detail/action components with accessible interaction tests.

### Contract
Fixture descriptors, generated client types, BFF/O3K adapter behavior and Problem Details.

### Integration
BFF + deterministic fixture adapter; later BFF + supported O3K test environment.

### E2E
Critical tenant/operator journeys only; avoid turning every field into a brittle browser test.

## 5. Accessibility gates

Target: WCAG 2.2 AA for critical production workflows.

Minimum MVP evidence:

- keyboard-only tenant core journey,
- keyboard-only operator diagnostic journey,
- focus visibility/order,
- accessible names and error association,
- modal/drawer focus management,
- no serious/critical automated accessibility violations on critical pages,
- manual review for cases automation cannot validate.

Do not hide accessibility failures behind test exclusions without a tracked issue and bounded rationale.

## 6. Performance/scale gates

The most important early scale rule is bounded data handling, not synthetic Lighthouse scores.

Requirements:

- collections are server paginated,
- fixture adapter can report >=100,000 total resources while returning bounded pages,
- table/filter UI remains responsive with a normal page payload,
- route bundles are code split by application/service area where useful,
- no accidental import of both console applications into a single browser bundle,
- no N+1 per-row API requests for list pages.

Performance budgets may be tightened after a measured production-like baseline exists; do not invent arbitrary network SLAs in the prototype.

## 7. Visual regression

Once M1 stabilizes design primitives, add screenshot/visual checks for a small set of canonical surfaces:

- tenant shell/home,
- resource list,
- resource details,
- create flow,
- operation details,
- operator overview.

Visual tests supplement semantic tests; they do not replace them.

## 8. Security gates

M12 must prove:

- no browser-stored cloud tokens,
- separate tenant/operator session namespaces,
- CSRF rejection,
- cross-project access rejection,
- operator endpoints reject tenant session,
- strict CSP compatible with shipped assets,
- untrusted resource strings cannot create executable markup,
- descriptor validation rejects executable/unknown dangerous content,
- sensitive log redaction.

## 9. Definition of done for every implementation issue

An issue is complete only when:

1. acceptance criteria pass,
2. relevant automated tests exist,
3. docs/ADRs are updated if behavior changed,
4. no architecture invariant was bypassed,
5. final implementation report identifies upstream gaps and deferrals,
6. CI is green.
