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

Discovery payloads are bounded (64 KiB), identifiers are validated, schema
objects are bounded to 32 levels and 256 children per node, pagination is
cursor-bounded (100 pages), repeated cursors fail closed, and path segments
are encoded. Upstream errors remain structured; production adapters never
activate fixture descriptors as a fallback. Fixture data remains behind the
explicit fixture adapter for tests/development.

The validator accepts the merged O3K reference forms: namespaced action IDs
(`service:Action`) and bounded HTTPS schema/contract URLs (including JSON
Pointer fragments). This keeps the public O3K descriptors executable without
loosening schema safety.

Tenant geography is available at `/api/v1/regions` and `/api/v1/regions/:id/zones`;
the tenant shell renders only discovered regions (the legacy Global option is
disabled there). Operator aliases remain protected by the Operator BFF.

## Validation

Real process evidence used the O3K Rust checkout at commit
`21fe687c387a04f107b6e87fac04060b1c28e449`:

```text
O3K_LISTEN_ADDR=127.0.0.1:18080 O3K_PROVIDER=fake \
O3K_BOOTSTRAP_PASSWORD=<runtime-only> O3K_TOKEN_SIGNING_KEY=<runtime-only> \
O3K_LOCATIONS='[{"id":"eu-test-7","availability_domains":[{"id":"eu-test-7a"},{"id":"eu-test-7b"}]},{"id":"us-test-3","availability_domains":[{"id":"us-test-3a"}]}]' \
o3kd
```

The live Araf `O3kAdapter` process test used a server-side native token and
reported:

```text
context_project=eba29e2d-53de-461d-ae91-ede7402713cb
services=5 resource_types=19 descriptors=5 regions=2
service_ids=identity,image,volume,compute,network
region_ids=eu-test-7,us-test-3
```

The same response showed `volume.lifecycle_state=not_ready` (and its resource
types were `ready=false`). Araf retained the service and readiness state in the
discovery/catalog projection for truthful diagnostics, while omitting those
not-ready resource types from tenant descriptors so no production Volume route
is invented. No new O3K API was designed or required.
