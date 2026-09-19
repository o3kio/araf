# O3K PP.3 Araf release-preparation evidence

Status: **INCOMPLETE — candidate publication and real O3K smoke are pending.**

This record is deliberately separate from the historical P4 evidence. It does
not turn a controlled backend or a development O3K checkout into a release
claim. There is no active O3K deployment available for this candidate.

## Release identity

| Field | Value |
| --- | --- |
| Branch | `release/araf-o3k-pp3-prep` |
| Protected `main` audit SHA | `871f2b5534cf4f6eb1cb2a04d74e0f0275ca7613` |
| Candidate authority | `backend/RELEASE_VERSION` |
| Intended next prerelease | `v1.0.0-rc.14` (recheck registry before tag) |
| GitHub Release | pending merge/tag/publication |
| OCI packages | currently private; public visibility is a publication gate |
| Primary deployment | digest-pinned Docker Compose release |
| Runtime tested | Docker Engine 29.8.0, Compose 5.5.1, linux/amd64 |

The four runtime components are `tenant-console`, `operator-console`,
`tenant-bff` and `operator-bff`. The two BFFs intentionally share one OCI
image repository/digest and run separate commands and security/session
surfaces.

## Evidence matrix

| Check | Result |
| --- | --- |
| Release manifest generation/identity gate | PASS |
| Compatibility metadata and fixture-disabled assertion | PASS |
| Compose source-build/floating-tag gate | PASS |
| Helm lint and digest syntax | PASS |
| Package/upgrade/rollback static gate | PASS |
| Security release gate, dependency inventories, secret patterns | PASS |
| Rust format | PASS |
| Core Rust tests (79), contract tests (62), O3K adapter (15), OpenStack adapter (6) | PASS |
| Published exact candidate images and post-publish labels | PENDING |
| Public GitHub prerelease and anonymous artifact re-resolution | PENDING |
| SBOM/provenance verification for candidate digests | PENDING |
| Clean source-free deployment | PENDING |
| Same-version rerun/recreation | PENDING |
| Restart/reboot smoke | PENDING |
| Scoped cleanup with foreign canaries | PENDING |
| Real O3K adapter against compatible O3K environment | BLOCKED: no active environment |

## Compatibility and security boundary

The embedded contract is `o3k.io/v1` at `/o3k/v1`, with a historical P2
tested source reference. It intentionally has no stable O3K release range.
`ARAF_O3K_API_CONTRACT` is validated at startup; a mismatch fails closed.
OpenStack remains bounded to the documented 2026.1 core profile. Fixture mode
is forbidden in release builds and production profile selection. Browser
credentials remain opaque cookies; OIDC/O3K credentials remain server-side.

`/healthz` is process liveness and `/readyz` means configured adapter
construction. Neither endpoint claims that O3K is healthy or ready. Tenant and
Operator routes, cookies, OIDC clients and stores remain separate.

## Findings

BLOCKER: candidate has not been published from final protected source; GHCR
packages are private until publication; no active O3K deployment exists.

HIGH: no candidate-specific provenance/SBOM/digest or clean deployment result
can be recorded before publication.

MEDIUM: live cross-candidate rollback, multi-node O3K HA and the O3K delete
response-semantics investigation remain outside this task and are tracked by
#113/#106.

LOW: candidate's public package/release URLs are not yet known.

No release artifact is allowed to claim Ready for PP.3 until all pending gates
are rerun against the exact public candidate. The next candidate rule is
immutable: fix source, consume the failed version, publish a successor.
