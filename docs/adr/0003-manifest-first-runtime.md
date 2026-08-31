# ADR 0003: Manifest-first generic resource runtime

Status: Accepted for prototype/MVP planning

## Context

O3K is designed to add services/providers over time. Hand-written screens and navigation for each service would make the dashboard a scaling bottleneck and couple product UX to implementation teams.

## Decision

Normal service integration is descriptor/schema driven. Service/resource metadata defines list fields, details, actions, capabilities, relationships and form schemas. Araf owns presentation metadata only where required.

Custom frontend modules are an explicit exception requiring justification.

## Consequences

Positive:
- service extensibility,
- consistent UX,
- smaller long-term frontend maintenance cost,
- machine-readable action model for future automation/agents.

Risks:
- over-generalized forms can produce poor UX,
- descriptor schema can become an accidental programming language.

Mitigation:
- prototype gate explicitly tests usability,
- no arbitrary JavaScript expressions in descriptors,
- keep a bounded reviewed custom-UI escape hatch.
