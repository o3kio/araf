# Araf production release evidence

## Candidate

- Repository: `o3kio/araf`
- Candidate branch: `codex/p4-1-observability`
- Candidate source (implementation): `24a8b691a7c447ce001271519713d5b322757eb8`
- Artifact version: `0.1.0-rc` (locally signed/attested candidate; trusted CI provenance remains gated)
- Date: 2026-09-13

## Gate matrix

| Gate | Evidence | Result |
| --- | --- | --- |
| P2 native O3K | Post-P3 baseline and existing real O3K gate evidence | PASS (baseline) |
| P12 IAM/current-process identity | `p12-iam-real-idp-p4-evidence.md` (O3K `p12-iam-7-real-idp.sh` with Araf P12-IAM.8 hook) | PASS for the current-process external-IdP journey; this is not a production deployment claim |
| P3 OpenStack core profile | `docs/engineering/openstack-production-evidence.md`, `target/p3-9-openstack-gate/result.env` | PASS at exact commit `92d4013`; current-head rerun blocked by torn-down Kolla services |
| P4.1 observability | `p4-1-observability-evidence.md`, `/metrics`, `/readyz`, correlation tests | PASS locally; real deployment attachment required |
| P4.2 performance/scale | `performance-evidence.md`, `tests/performance-bounded.sh` | PASS for bounded fixture; O3K/OpenStack load attachment required |
| P4.3 HA/session | encrypted file-locked store tests and restart/replica topology | PASS for the shared durable primitive and concurrent-writer recovery; multi-host rolling-failure test required |
| P4.4 security/supply chain | `security-release-evidence.md`, CI gate, pinned Trivy/Syft/cosign artifacts | PASS local scan/SBOM/signature; MEDIUM base refresh/risk acceptance and trusted CI provenance required |
| P4.5 packaging | current-head OCI builds/digests, Helm lint/template, Compose validation | PASS build and packaging checks; clean external install/upgrade attachment required |
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
external-IdP boundary was also exercised on 2026-09-13: O3K's
`p12-iam-7-real-idp.sh` harness passed IAM.7 and IAM.8 with an ephemeral
Keycloak realm, a current O3K `o3kd` process, Araf's real Tenant BFF process,
opaque-cookie session custody, scope discovery/selection, a resource request,
and logout. This closes the process-level identity/session evidence gap, but
the harness uses a disposable development HTTP topology and fake provider;
it is not evidence of a production O3K deployment, HTTPS termination, or
multi-host failover.

The real P3.9 harness passed at exact Araf commit `92d4013` against the development
host's Kolla-Ansible OpenStack 2024.2 (Dalmatian), Keystone-backed Keycloak
ingress and two Araf surfaces. A later current-head rerun returned `NO-GO`
because the Kolla services had been torn down and the Keystone proxy returned
502; no cloud capability claim is inferred from that failed rerun. Its
successful artifacts are redacted and record only IDs, capability count,
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
evidence, trusted CI provenance attestations, and a representative production
pilot/rollback. The real OpenStack evidence is profile-scoped and does not
waive those gaps or advertise Swift, floating IP, or attachment workflows.
