# P4.3 HA, resilience and session durability evidence

## Software gates

The BFF session boundary uses an encrypted, file-locked store when running in
production. Each mutation reloads the latest atomically-renamed snapshot while
holding an inter-process lock, so separate replicas preserve sessions and
revocations without sticky in-memory state. Temporary snapshots use a UUID in
their filename to avoid collisions between containers that reuse process IDs.

The O3K client applies a 30-second total request timeout. It does not retry
mutations after a transport failure; retryability remains an authoritative O3K
Operation concern, preventing duplicate destructive actions during outages.

Validation:

```text
cargo test --manifest-path backend/Cargo.toml -p console-bff-core session::tests
PASS — session creation/expiry/rotation, encrypted replica merge, revocation,
and concurrent cross-replica writes
```

The local `tests/pilot-soak.sh` gate also runs two Tenant and two Operator
replicas, repeatedly exercises bounded resource and metrics requests, then
abruptly kills and restarts one replica of each surface. The fixture adapter is
explicitly development-only and does not establish a production cloud claim.

## Boundary and remaining evidence

The shared durable primitive and same-host restart continuity are covered. A
multi-host deployment test with shared network storage, rolling failure,
upstream timeout/5xx injection and recovery is still required before the
production release gate can be marked complete. Araf does not invent an O3K
failure or operation result to close that external-environment gap.

**P4.3 verdict: PARTIAL — software resilience gates pass; external multi-host
acceptance remains open.**
