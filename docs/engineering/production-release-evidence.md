# Araf production release evidence

## Candidate

- Repository: `o3kio/araf`
- Candidate branch: `codex/target-openstack-2026-1`
- Candidate source (implementation): `24a8b691a7c447ce001271519713d5b322757eb8`
- Artifact version: `0.1.0-rc` (locally signed/attested candidate; trusted CI provenance remains gated)
- Target OpenStack profile: **2025.1 or later**, using matching Kolla-Ansible
  tooling and service images
- Date: 2026-09-13

## Gate matrix

| Gate | Evidence | Result |
| --- | --- | --- |
| P2 native O3K | Post-P3 baseline and existing real O3K gate evidence | PASS (baseline) |
| P12 IAM/current-process identity | `p12-iam-real-idp-p4-evidence.md` (O3K `p12-iam-7-real-idp.sh` with Araf P12-IAM.8 hook) | PASS for the current-process external-IdP journey; this is not a production deployment claim |
| P3 OpenStack core profile | `docs/engineering/openstack-production-evidence.md`, disposable Kolla 2025.1 harness artifact | PASS for the current supported profile on a clean single-host Kolla 2025.1 deployment; multi-host and external-environment evidence remain open |
| P4.1 observability | `p4-1-observability-evidence.md`, `/metrics`, `/readyz`, correlation tests | PASS locally; real deployment attachment required |
| P4.2 performance/scale | `performance-evidence.md`, `tests/performance-bounded.sh` | PASS for bounded fixture; O3K/OpenStack load attachment required |
| P4.3 HA/session | encrypted file-locked store tests, restart/replica topology, and current-head O3K replica smoke | PASS for the shared durable primitive, concurrent-writer recovery, and same-host O3K replica continuity; multi-host rolling-failure test required |
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

The real P3.9 harness passed at the current candidate implementation source
(`24a8b691a7c447ce001271519713d5b322757eb8`) against a clean disposable
Kolla-Ansible OpenStack **2025.1** deployment, a Keystone-backed Keycloak
ingress, and both Araf surfaces. The run exercised real image/flavor/network/
subnet/volume/server resources, asynchronous Nova lifecycle actions, invalid
input, quotas, deletion, project-scoped direct-ID isolation, and persisted
compatibility operations. Its redacted result is retained at
`/tmp/araf-p4-openstack-2025/evidence5/redacted-run.txt` and records only
opaque IDs, capability count, operation count, and the selected profile. This
is a single-host support-profile result; it does not certify multi-host
failover, external installation, or a production pilot.

On current head `22a93894b89a659a5eb62decccf6e3f90852a6c3`, a fresh disposable
O3K TestLab (`agent` provider, O3K source `157fde108c5e0a9c6567f596d88a6abbb55b2aaf`)
was provisioned and connected to production-profile Tenant and Operator BFFs
through the deployment's HTTPS reverse proxy. Real Keycloak federation and
audience validation passed; Alice and Bob discovered and selected their
server-authoritative scopes, created/polled/deleted a network operation, and
could not list, show, delete, or read the other tenant's resource/operation.
Opaque secure session cookies, credential-free context/resource payloads,
bounded collections, and the Operator profile/tenant denial journey also
passed. The redacted diagnostic result is retained at
`/tmp/araf-p4-o3k-current-head-2035/evidence-project-a5/harness.redacted.json`.

This is not a production-readiness pass: the agent lab has no region, image or
compute inventory, so those capabilities were recorded as upstream gaps. The
deployment-owned legacy harness also assumes a reserved bootstrap project ID;
the original unmodified run therefore stopped at `native_token_exchange`
without treating that mismatch as a product success. Multi-host failover,
external clean-install, and production-pilot evidence remain required.

The same current-head O3K topology also ran two Tenant BFF replicas sharing
the encrypted durable session file. Login and scope/CSRF selection on replica A
were accepted by replica B; an abrupt replica-B kill produced an observed
outage, restart restored readiness and the scoped session, and logout on B was
visible as revocation on A. This proves same-host durable-session continuity,
not multi-host storage or network-failure tolerance; the redacted result is
`/tmp/araf-p4-o3k-current-head-2035/ha-o3k-redacted.json`.

The browser-critical Playwright suite passes locally (17 tests) against the
fixture profile, including tenant/operator navigation, resource actions,
operations, governance, and scope/isolation journeys.

On 2026-09-13, a disposable clean-install smoke used the release registry's
digest-pinned BFF image (`sha256:c1aabc13…`) as a non-root, read-only container
with a durable encrypted session volume. Through the HTTPS ingress, the clean
image completed real Keycloak OIDC login, server-side project selection,
descriptor/context reads, and a native O3K network create/detail/delete with
the canonical operation returned in the response. The same volume was then
started with the preceding release digest (`sha256:3e4ebd06…`) and returned to
the candidate digest; both revisions passed health/readiness, OIDC login,
scope selection and native resource reads. This is useful packaging and
rollback evidence on one host, but it is not the required clean external
environment, multi-host rolling upgrade, trusted-provenance or pilot gate.

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
