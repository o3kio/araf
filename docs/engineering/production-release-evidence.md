# Araf production release evidence

## Candidate

- Repository: `o3kio/araf`
- Candidate branch: `codex/p4-1-observability`
- Candidate source: `7e7882204b60232d4b027476dad33dd7a8f5986c`
- Artifact version: `0.0.0-dev` until a signed release tag is created
- Date: 2026-09-13

## Gate matrix

| Gate | Evidence | Result |
| --- | --- | --- |
| P2 native O3K | Post-P3 baseline and existing real O3K gate evidence | PASS (baseline) |
| P3 OpenStack core profile | `docs/engineering/openstack-production-evidence.md`, `target/p3-9-openstack-gate/result.env` | PASS (real Kolla 2024.2 Dalmatian profile) |
| P4.1 observability | `p4-1-observability-evidence.md`, `/metrics`, `/readyz`, correlation tests | PASS locally; real deployment attachment required |
| P4.2 performance/scale | `performance-evidence.md`, `tests/performance-bounded.sh` | PASS for bounded fixture; O3K/OpenStack load attachment required |
| P4.3 HA/session | durable store test and restart/replica topology | PASS for local durable primitive; multi-host shared-store test required |
| P4.4 security/supply chain | `security-release-evidence.md`, CI gate | PASS for deterministic checks; advisory/image scan attachment required |
| P4.5 packaging | `packaging-evidence.md`, Helm lint/template | PASS artifacts; clean external install/upgrade attachment required |
| P4.6 supportability | `docs/operations/operator-runbook.md`, redacted bundle script | PASS documentation gate |
| P4.7 pilot/soak | `tests/pilot-soak.sh`, `target/p3-9-openstack-gate/redacted-run.txt` | PASS for local HA soak and real OpenStack profile; full RC pilot still required |

## Local pilot

The local soak exercises two Tenant and two Operator BFF processes, repeated
bounded resource lists and metrics reads, then abruptly kills and restarts one
replica of each surface. The latest run completed in 9 seconds with 800
requests and recovery passing. The fixture adapter is explicitly labelled
development-only and cannot establish production cloud or identity claims.

Converged O3K process smoke gates also pass for discovery/collection, native
operations, governance, and metering (`tests/p2-3` through `tests/p2-5` and
`tests/p2-7`, with the local O3K process using its fake provider). The real
P3.9 harness also passed against the development host's Kolla-Ansible
OpenStack 2024.2 (Dalmatian), Keystone-backed Keycloak ingress and two Araf
surfaces. Its artifacts are redacted and record only IDs, capability count,
operation count and the selected profile.

The browser-critical Playwright suite passes locally (17 tests) against the
fixture profile, including tenant/operator navigation, resource actions,
operations, governance, and scope/isolation journeys.

## Supported profiles and deviations

The advertised OpenStack profile remains Keystone, Nova, Glance, Neutron and
Cinder only. Swift/Object Storage, floating IP and volume attachment are not
advertised without separate evidence. O3K remains authoritative for native
semantics; OpenStack CompatibilityOperations remain correlation/reconciliation
state only.

## Verdict

**NO-GO — NOT PRODUCTION READY**

This candidate must not be called v1.0 yet. The remaining release boundary is
a current-head O3K production run, multi-host durable-session/rolling-restart
evidence, signed OCI artifacts with external image/advisory scans, and a
representative production pilot/rollback. The real OpenStack evidence is
profile-scoped and does not waive those gaps or advertise Swift, floating IP,
or attachment workflows.
