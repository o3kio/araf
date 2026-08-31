# M9 implementation prompt — operator platform, regions and provider health

Implement **M9** for the Operator Console.

## Read first

- `AGENTS.md`
- ADR 0001
- `docs/product/information-architecture.md`
- `docs/security/threat-model.md`
- current O3K operator/provider/region contracts.

## Goal

Create an operator experience for platform/customer/resource oversight without data-plane shell access and without exposing operator capabilities through the Tenant Console.

## Required implementation

1. Inspect upstream O3K contracts before choosing writable operator flows.
2. Implement the supported subset of:
   - platform overview,
   - regions/AZs,
   - provider/service health,
   - normalized capacity summaries,
   - accounts/organizations/projects,
   - global/filtered Operations,
   - relevant audit evidence.
3. Provider-specific diagnostics may be visible to authorized operators, but must be clearly separated from tenant resource vocabulary.
4. Show data freshness/observation timestamps for health/capacity when available.
5. Never derive authoritative platform capacity by scraping provider-specific browser data or direct data-plane endpoints.
6. Do not add SSH/kubectl/provider shell escape hatches.
7. Confirm tenant session cannot access Operator BFF routes and operator application routes are absent from Tenant Console.
8. Risky writable operator actions should only be implemented if upstream authorization/audit semantics are mature; otherwise prefer read-only MVP views and record the dependency.

## Acceptance

- operator can identify region/provider degradation and inspect associated operations without data-plane access,
- organization/project lookup is scope/authorization safe,
- tenant credentials/session cannot use operator endpoints,
- capacity/health is based on authoritative O3K control-plane data,
- no unsupported privileged action is fabricated.

Branch suggestion: `m9-operator-platform`.
