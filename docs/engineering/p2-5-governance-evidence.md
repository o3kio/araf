# Araf P2.5 governance evidence

This workstream consumes the frozen O3K northbound contracts at
`21fe687c387a04f107b6e87fac04060b1c28e449` through the server-side
`O3kAdapter`. Tenant quota and audit capabilities are discovered by probing
the canonical `/o3k/v1/quota` and `/o3k/v1/audit` readers. IAM administration
routes are operator-scoped in O3K, so tenant Projects, Users, Roles and API
credential pages remain capability-hidden in production rather than using the
former fixture authority.

The BFF normalizes bounded native quota and audit responses, preserves native
scope and correlation identity, rejects caller-supplied foreign project
filters, validates the native quota contract version and limit kinds, and
detects malformed or repeated audit cursors. Tokens remain server-side.

Fixture governance data remains available only through the explicit fixture
adapter used by deterministic tests/development; production O3K failures do
not fall back to it.

Validation evidence is recorded with the implementation PR and must include
Rust/frontend gates, browser tests, and a real Araf BFF → O3K process smoke
run before review approval.
