# M2 implementation prompt — Tenant/Operator shells and explicit context

Implement **M2** after M1 and M0.

## Read first

- `AGENTS.md`
- `docs/product/information-architecture.md`
- `docs/architecture/overview.md`
- ADR 0001

## Goal

Build distinct Tenant and Operator application shells using shared packages while making scope/context obvious and preventing privilege-surface mixing.

## Required implementation

1. Implement a shared shell/navigation package using `packages/ui`.
2. Tenant Console:
   - persistent project/organization context,
   - region selector supporting an explicit `Global` state,
   - global search affordance placeholder,
   - Operations entry point,
   - identity/help area,
   - stable structural navigation plus descriptor-ready service area.
3. Operator Console:
   - distinct platform-oriented navigation,
   - no tenant-only assumptions,
   - visible operator/platform context.
4. Put selected context into bookmarkable URL/application state where appropriate. Refresh/back navigation must not silently change the effective project/region.
5. Add explicit guards so operator routes/components are not bundled/reachable through the tenant application route table.
6. Use fixture identity/context data only through a clearly named fixture provider; do not invent production auth.
7. Add responsive and keyboard-navigation tests for the shell.

## UX invariant

A user must not be able to perform a cloud action while being uncertain which project and region it targets.

## Non-goals

- real OIDC,
- real service discovery,
- global search backend,
- resource CRUD.

## Acceptance

- tenant/operator apps visibly differ by purpose but share the same design system,
- project/region switching is deterministic and testable,
- `Global` is modeled explicitly,
- direct route navigation retains scope,
- tenant application contains no operator route surface,
- critical shell navigation works with keyboard only.

Branch suggestion: `m2-console-shells`.
