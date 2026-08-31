# ADR 0004: O3K-owned UI API over third-party component primitives

Status: Accepted for prototype/MVP planning

## Context

Enterprise cloud consoles need mature accessibility, tables, forms, navigation and resource-management interaction patterns. Building every primitive from scratch increases delivery/security/accessibility risk, but binding application code directly to a third-party component API creates long-lived vendor/framework coupling.

## Decision

Create `packages/ui` as the only package allowed to import the chosen underlying enterprise component library. Application/runtime packages consume O3K/Araf components and design tokens.

The initial implementation should evaluate/use Cloudscape-compatible primitives because of their cloud resource-management maturity, but Araf owns its public UI abstraction and visual identity.

## Consequences

Positive:
- leverage mature components/accessibility,
- consistent product vocabulary,
- future replacement/theming boundary,
- third-party APIs do not leak throughout application code.

Cost:
- wrapper design work,
- wrappers must not merely rename every underlying component without adding a stable product contract.
