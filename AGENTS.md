# Araf agent instructions

These rules apply to coding agents and LLMs working in this repository.

## Read first

Before changing code, read:

1. the GitHub issue being implemented,
2. its matching file under `prompts/`,
3. `docs/product/mvp-prototype.md`,
4. `docs/architecture/overview.md`,
5. `docs/architecture/o3k-integration-contract.md`,
6. `docs/security/threat-model.md`,
7. `docs/engineering/quality-gates.md`.

If an issue conflicts with an ADR, stop and report the conflict instead of silently changing architecture.

## Architectural invariants

- Araf is **not** the authority for O3K cloud semantics. The O3K control plane is authoritative.
- Never invent a production O3K endpoint, resource state, permission or operation result just to make the UI work.
- If an upstream capability is absent, document the gap and keep fixture/demo behavior clearly separated from production adapters.
- Tenant and Operator consoles share packages but have separate application, BFF, authentication and deployment boundaries.
- Do not store O3K access tokens, refresh tokens or session identifiers in `localStorage` or `sessionStorage`.
- Do not expose provider-specific implementation details in ordinary tenant resource models.
- Do not import the underlying third-party component library outside `packages/ui`.
- Do not add arbitrary remotely executed JavaScript as a service-extension mechanism.
- Normal service UX must be descriptor/schema driven. Custom service pages are exceptions and require an explicit issue/ADR.
- Async cloud actions must surface canonical O3K Operations; a `202 Accepted` is not a completed action.
- UI capability checks improve usability only. Server-side authorization remains mandatory.
- No console-only privileged cloud behavior. If the UI can perform a cloud action, there must be an authoritative O3K API operation behind it.
- Cloud collections use server-side pagination/filtering/sorting. Never fetch an unbounded resource inventory into the browser.

## Scope discipline

Implement only the current issue. Do not opportunistically add post-MVP features such as marketplace support, arbitrary plugins, full billing, migration center, AI control, graphical console, or additional managed services.

## Engineering expectations

Every change must preserve or add appropriate:

- TypeScript strict typing,
- Rust formatting/clippy cleanliness,
- unit/component tests,
- contract tests for descriptor/API boundaries,
- Playwright tests for user-critical flows when applicable,
- accessibility coverage for changed interactive UI,
- threat-model tests when security boundaries are touched.

Do not disable lints, type checks, CSP protections or tests to make CI pass without an explicit documented reason.

## Final implementation report

Every implementation prompt must finish with a report containing:

- branch and commit,
- files changed,
- architectural decisions made,
- tests executed and results,
- security/accessibility implications,
- upstream O3K dependencies or gaps,
- deferred items/non-goals,
- whether the issue acceptance criteria are fully satisfied.
