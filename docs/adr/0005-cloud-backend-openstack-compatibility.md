# ADR 0005: CloudBackend and OpenStack compatibility boundary

## Status

Accepted for P3 implementation (2026-09-13). This ADR freezes the boundary;
it does not constitute a production support claim for any OpenStack release.

## Decision

The Rust `console-bff-core::upstream::Upstream` contract is the Araf
`CloudBackend` request boundary. `cloud_backend::CloudBackend` adds explicit
backend provenance (`O3k` or `OpenStack`) without changing the
backend-neutral resource, scope, capability, governance or usage DTOs. The
existing `O3kAdapter` remains authoritative for native O3K semantics. The
`OpenStackAdapter` translates Keystone v3, Nova v2.1, Glance v2, Neutron v2
and Cinder v3 responses behind this boundary. React code selects no provider
and contains no provider-specific branches.

Capabilities are observed from configured service endpoints and successful
upstream discovery. Missing endpoints are unavailable and are not simulated.
Authorization is always delegated to Keystone/OpenStack on the authoritative
request; capability metadata is UX guidance only.

## Identity and credentials

Browser sessions contain only opaque cookies. OpenStack bearer tokens are
accepted from server configuration (`OPENSTACK_TOKEN`) or the server-side
session slot and are sent only by the BFF. Project selection is validated by
`GET /v3/auth/projects` and stored in the server-side session. No token,
password or session identifier is placed in browser storage or ordinary
resource DTOs.

## Compatibility operations

OpenStack has no universal operation resource. A mutation returns an Araf
`Operation` whose ID is prefixed `openstack-compat-`, with state `running`
until the durable compatibility journal observes an authoritative terminal
resource condition. The object is correlation/reconciliation state only;
resource reads always win over it. Journal retention is bounded, restart
reconciliation is explicit, and `unknownOutcome` is used when authoritative
state cannot establish success or failure. It never invents fine-grained
steps.

## Configuration and rollout

`ARAF_UPSTREAM_ADAPTER=openstack` is explicit. Production requires an HTTPS,
credential-free `OPENSTACK_AUTH_URL`; the adapter does not fall back to O3K or
fixtures. Service endpoints are optional and are capability-gated:
`OPENSTACK_COMPUTE_URL`, `OPENSTACK_IMAGE_URL`, `OPENSTACK_NETWORK_URL`,
`OPENSTACK_VOLUME_URL`, and optional `OPENSTACK_OBJECT_STORAGE_URL`.

## Consequences

The generic Araf runtime is reusable across backends, while provider-specific
diagnostics remain server-side. A real deployment, supported-version matrix,
quota behavior and restart durability still require P3.2-P3.9 evidence.
