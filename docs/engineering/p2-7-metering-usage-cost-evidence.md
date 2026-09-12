# Araf P2.7 metering, usage and bounded cost evidence

## Identity

- Base main: `599ddc341e0a20f3cf46fb58a5b4422408fcfa19`
- Branch: `p2-7-metering-usage-cost`
- O3K authority: `21fe687c387a04f107b6e87fac04060b1c28e449`

## Authority and implementation

Production usage and quota calls are made by the Rust BFF's `O3kAdapter` using
the frozen O3K native contracts. Meter definitions come from
`GET /o3k/v1/metering/definitions`; bounded usage comes from
`GET /o3k/v1/metering/usage`; quota remains `GET /o3k/v1/quota`. The adapter
uses the server-side effective scope from `/identity/me`, filters to
tenant-visible definitions, propagates opaque definition cursors with a hard
page bound, and rejects scope/key/shape mismatches.

Usage timestamps are RFC3339 and the requested range is bounded and UTC-hour
aligned. Quantities remain exact, bounded decimal strings (three fractional
digits); no floating-point arithmetic or Araf-side billing is performed.
O3K currently provides no pricing contract, so the tenant page explicitly
renders cost as unavailable. `complete`, `partial`, `unavailable`, and unknown
future statuses remain visible rather than being flattened to zero.

The legacy hourly `records` shape is retained solely for explicit fixture/test
mode. A failed, malformed, unauthorized, or unavailable production O3K call
returns a structured error and never activates fixture data.

## Security and runtime bounds

- Browser responses contain normalized public meter data only; bearer/session
  credentials remain server-side.
- Project/scope query values cannot override the authenticated O3K scope.
- Native usage is limited to eight meter keys and a bounded date range.
- Definition pagination detects missing/repeated cursors and stops at 100 pages.
- Response bodies use the existing bounded O3K client limits.

## Validation and integration

WireMock contract tests cover definition pagination, repeatable meter query
parameters, exact-decimal/timestamp normalization, tenant scope rejection,
unknown statuses, and production no-fallback behavior. Fixture tests continue
to cover deterministic legacy usage rendering. A dedicated real-process gate
(`tests/p2-7-real-metering-process.sh`) exercises Araf BFF → real O3K when
`O3K_URL` and `O3K_TOKEN` are supplied; it is not replaced by a fixture route.

## Non-goals and deviations

This workstream does not add pricing, billing, cost estimates, audit UX,
Operations Center UX, or new O3K APIs. No O3K contract defect was identified.

