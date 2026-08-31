# ADR 0001: Shared platform, separate Tenant and Operator console surfaces

Status: Accepted for prototype/MVP planning

## Context

Araf must support self-service tenants and privileged cloud operators. A single browser application that merely hides operator navigation would enlarge the public attack surface and make session/privilege separation harder to reason about. Completely separate codebases would duplicate design/runtime work and drift.

## Decision

Build two applications from one shared UI/runtime platform:

- Tenant Console + Tenant BFF
- Operator Console + Operator BFF

They share packages but use separate deployment origins, OIDC clients, session namespaces and route exposure.

## Consequences

Positive:
- clearer trust boundary,
- public tenant surface need not expose operator routes,
- reusable design/runtime,
- deployers can place operator surface on a management network.

Cost:
- two application builds/deployments,
- explicit shared-package discipline,
- end-to-end tests for both surfaces.
