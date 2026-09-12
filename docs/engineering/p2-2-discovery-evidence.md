# Araf #45 discovery evidence

This note records the P2.2 implementation boundary. It is intentionally
discovery-only; resource lifecycle convergence remains #46.

## Authority and identity

- Base main: `dfe55a5a5c40233d44e1929358d72bbbfc951740`
- O3K dependency: `21fe687c387a04f107b6e87fac04060b1c28e449` (#907/#928)
- Araf uses the existing server-side P12-IAM scope/AuthContext path. Browser
  project IDs are never used as authority.

## Discovery flow

The O3K adapter consumes `/o3k/v1/services`, `/resource-types`, versioned
`/resource-schemas/{namespace}/{collection}/{version}`, and `/regions` through
the BFF. Resource list/detail/create/delete/action dispatch resolves the
canonical namespace/collection from the current resource-type discovery on
each request. Araf adds only labels, grouping, icons, and table presentation.

Resource descriptors carry O3K schema/version, placement, region/AZ metadata,
and action metadata. Create forms are rendered only when an advertised create
operation has a safely extracted authoritative `spec` schema. Actions are
validated against discovery before dispatch, and every dispatched mutation
returns the canonical O3K operation.

Discovery payloads are bounded (64 KiB), identifiers are validated, pagination
is cursor-bounded (100 pages), repeated cursors fail closed, and path
segments are encoded. Upstream errors remain structured; production adapters
never activate fixture descriptors as a fallback. Fixture data remains behind
the explicit fixture adapter for tests/development.

Tenant geography is available at `/api/v1/regions` and `/api/v1/regions/:id/zones`;
the tenant shell renders only discovered regions (the legacy Global option is
disabled there). Operator aliases remain protected by the Operator BFF.

## Validation

The final report records exact-head frontend, Rust, browser, and real-O3K
process validation. No new O3K API was designed or required.
