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

## 2026-09-14 acceptance audit

Tested Araf baseline: `a0e1b11688837caa0d96c07399e3c231d07cbe30` (the merged
PR #103 main), with the journal hardening candidate on branch
`codex/p4-3-journal-ha`.

The development host is Linux 6.8 on x86_64 with Docker 29.8, Compose 5.5,
and KVM/libvirt available. The certified OpenStack deployment is the Kolla
2026.1 profile. No supported external O3K deployment/profile was available
during this audit. A disposable `o3kd` process (fake provider) was started by
the P12 IAM harness, but the production Araf hook correctly rejected its
plain-HTTP loopback URL (`production O3K_URL must be HTTPS, host-qualified,
and contain no credentials, query, or fragment`). The harness therefore did
not exercise Araf against that process, and this is recorded as a fail-closed
configuration result rather than O3K acceptance evidence. No shared
NFS/virtiofs/distributed filesystem was mounted. Therefore the
following observations are explicitly same-host evidence only:

- two Tenant and two Operator production-profile BFF processes used the same
  encrypted session store; alternating requests retained the authenticated
  scope and CSRF state;
- a Tenant and Operator replica were SIGKILLed and reconstructed; readiness
  returned and the shared session state remained available;
- fixture soak: 800 bounded requests across 2 Tenant + 2 Operator replicas,
  including abrupt kill/restart, passed;
- prior disposable O3K-agent diagnostic evidence recorded 200 requests at
  concurrency 16 and cookie/session/revocation checks, but it is not a
  current real O3K deployment and is not counted as production acceptance;
  the attempted P12 composition above likewise is not counted.

The journal audit found and fixes in the candidate branch: UUID/PID-scoped
temporary files, fail-closed malformed/read-error handling, newest-record
merging so stale replicas cannot overwrite authoritative state, and refreshes
before cross-replica CompatibilityOperation reads/reconciliation. The session
reader was also hardened to clear in-memory state and deny reads when the
durable authority is missing or corrupt, preventing stale-session use during a
partition. Regression coverage includes stale-writer protection,
cross-replica refresh, journal/session corruption and deletion failure, and
the existing bounded journal tests. Rust fmt, clippy (`-D warnings`),
workspace check/tests all pass on that candidate.

Not proven: two guest failure domains, a shared network session authority with
verified locking/rename semantics, real load-balanced Tenant/Operator guests,
rolling and VM-power-loss tests, real O3K timeout/connection-loss/5xx and
ambiguous mutation behavior, canonical O3K recovery, OpenStack outage
reconciliation under guest loss, network partition fail-closed behavior, and
the required repeated 100+ concurrent cross-node mutation sequence. Issue #63
therefore remains open; no physical-host HA claim is made.
