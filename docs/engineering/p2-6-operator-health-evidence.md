# Araf P2.6 operator health and capacity evidence

Base Araf main: `8feb3b32442aba1ae70284aa3781d6950f2596ac`  
O3K authority: `21fe687c387a04f107b6e87fac04060b1c28e449`

The operator BFF now consumes the frozen O3K native diagnostics projection:
`/o3k/v1/operator/diagnostics`, bounded service/provider pages, and
`/o3k/v1/operator/diagnostics/capacity`. O3K remains authoritative for
status, reason, observation time, capacity and control-plane freshness.
Location discovery remains identity-only, so Araf reports regions and
availability domains as `unknown` rather than fabricating health or timestamps.
Unknown future diagnostic statuses and provider kinds fail closed to explicit
`unknown` values.

The adapter enforces the native `v1` contract, clamps diagnostics pages to the
O3K maximum, detects missing/repeated cursors and excessive page counts, and
maps upstream failures through the structured BFF error model. Installed
service readiness is joined to diagnostics rather than inferred from static
catalog lifecycle fields. Platform overview does not invent an Operations
count; canonical Operations remain the authority of #47.

Operator routes retain system/operator authorization and server-side token
custody. Normalized DTOs contain no reusable credentials. Fixture health,
capacity and location data remain explicit fixture-adapter inputs only; failed,
forbidden, malformed or unavailable production diagnostics never fall back to
fixtures.

Validation includes Rust format/check/clippy/test, frontend gates, Chromium
E2E, WireMock contract tests, and the ignored real-process gate
`tests/p2-6-real-operator-process.sh`, which runs the operator BFF against a
real converged O3K process using an accepted server-issued system/operator
token. No Operations Center, governance, diagnostics shell access or new O3K
API is implemented here.
