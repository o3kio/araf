# Araf P4.7 production-release evidence

This record is for the exact release candidate tested on 2026-09-15. It is
the source of truth for the P4.7 decision; historical evidence is linked only
where it remains relevant and is not silently substituted for candidate-level
evidence.

## Candidate identity

| Item | Candidate |
| --- | --- |
| Repository / source SHA | `o3kio/araf` / `a8d0d0e8fd2e7f5b34a6355c2674bd26c2aa6f7d` |
| Candidate version | `1.0.0-rc.1` |
| BFF image | `localhost:5001/araf-bff@sha256:c6fa854b8a9764d645b973d3df359a67d5701876bf70332a4c76927d508ce18a` |
| Tenant console image | `localhost:5001/araf-tenant@sha256:68ba1430d8bd4830172f7c11bf90f0e459c2eb724e06e505bdebfd00cda0879e` |
| Operator console image | `localhost:5001/araf-operator@sha256:350b6ea7d0ef0f709a7114b0592d72bf22beb561920b8cb7a06910a906d69732` |
| Chart | Repository Helm chart from the P4.5 packaging baseline; chart version is not changed by this candidate |
| Registry | Local OCI registry on the acceptance host (`localhost:5001`) |
| Build metadata | OCI revision labels match the source SHA and version on all three images |

The images were rebuilt from this SHA with BuildKit and loaded/pushed by
digest. This host has no cosign, Syft or Trivy installation; therefore a
trusted CI signature, SBOM attestation and vulnerability report are **not
proven for these local digests**. The candidate must not be promoted on the
basis of this local build alone.

## Release contract

OpenStack support is limited to the certified 2026.1 Gazpacho profile:
Keystone v3, Nova v2.1, Glance v2, Neutron v2 and Cinder v3. Swift/Object
Storage, public floating-IP workflows, volume attachment workflows and
provider-specific tenant UI are deferred and are not advertised.

O3K native functionality is supported only according to the documented
convergence/API profile. No stable O3K semantic release is available yet.
O3K is authoritative for native resource truth; OpenStack
`CompatibilityOperation` records are derived correlation/reconciliation
state, not an OpenStack authority.

`#106 remains OPEN — stable-release multi-node HA/O3K certification intentionally deferred.`

## Install and authentication

The candidate was started from the digest-pinned local OCI artifact, without
building source in the target runtime. Four non-root, read-only BFF
containers (two Tenant and two Operator) shared encrypted durable session and
CompatibilityOperation paths. The deployment used an HTTPS reverse proxy and
Keycloak 26.3 (`araf-p28`) as the real IdP.

`/healthz`, `/readyz`, `/version` and `/metrics` returned successfully on both
surfaces. `/version` reported `1.0.0-rc.1` and the exact source SHA above.
Tenant and Operator OIDC login, callback, logout/session cookies and re-login
were exercised over HTTPS. No provider token was present in browser-visible
responses.

## Functional journeys

The Tenant session discovered 30 capabilities, listed Keystone/Nova/Glance/
Neutron/Cinder resources and selected the server-authoritative project scope.
A real OpenStack network create returned a durable CompatibilityOperation and
the resulting network was verified in authoritative Neutron before cleanup.
An invalid compute create was rejected with a bounded error. The deferred
`object.storage.bucket` capability returned HTTP 501 and did not expand the
support contract.

The Operator session authenticated independently. A Tenant cookie sent to an
Operator route returned HTTP 401. Requests alternated across both replicas;
session and scope continuity remained intact after killing and restarting one
Tenant replica. This is Araf/OpenStack replica evidence only, not O3K HA
certification.

Existing matched-profile evidence remains in
[`openstack-2026.1-p3.9-evidence.md`](openstack-2026.1-p3.9-evidence.md) and
[`openstack-production-evidence.md`](openstack-production-evidence.md).
Those records cover the broader 2026.1 lifecycle matrix; this candidate run
adds live HTTPS/OIDC and digest identity but did not repeat every destructive
lifecycle action.

No O3K runtime was available on this host during P4.7. Existing functional
O3K convergence evidence is retained in
[`o3k-production-evidence.md`](o3k-production-evidence.md); it must not be
represented as a stable O3K release or as #106 evidence.

## Failures, security and observability

- Tenant-to-Operator access was rejected server-side (HTTP 401); this was not
  UI-only hiding.
- Missing Object Storage capability was distinguished from outage (HTTP 501).
- Invalid mutation returned a bounded problem response with correlation data;
  no automatic replay was attempted.
- HTTPS, OIDC state/PKCE, CSRF and cookie separation were exercised in the
  live candidate path. Existing release security negatives are recorded in
  [`security-release-evidence.md`](security-release-evidence.md).
- `/healthz`, `/readyz`, `/version`, `/metrics`, request IDs and correlation
  IDs were collected. The support path is browser → BFF → adapter →
  CompatibilityOperation/authoritative OpenStack resource.
- `tests/support-bundle-security.sh` passed with synthetic OIDC token-like,
  bearer, session-key and password markers. A live candidate bundle contained
  only version, readiness, metrics, environment names and a redaction notice;
  it contained no credentials, cookies, keys or tokens.

## Performance and pilot/soak

The candidate-level smoke exercised repeated authenticated context, service,
resource and metrics reads. The reusable bounded and OpenStack performance
baselines are recorded in [`performance-evidence.md`](performance-evidence.md)
(100-request OpenStack profile and 800-request two-Tenant/two-Operator
fixture soak). A production-like, independently operated soak with memory,
file-descriptor and journal-growth measurements was **not completed** for
these exact digests. Consequently no unqualified pilot/soak pass is claimed.

## Upgrade, rollback and failed rollout

The P4.5 package/upgrade/rollback checks passed previously and remain the
validated procedure: preflight, digest verification, state backup/checks,
Helm/container upgrade, readiness/smoke checks and rollback only across tested
state formats. A fresh exact-candidate N→N+1 upgrade and intentional failed
rollout were not repeated in this acceptance window. They remain release-gate
evidence to be attached before a GO decision.

## Findings and decision

The following deviations are release-gate findings, not hidden limitations:

1. **HIGH —** trusted CI provenance, SBOM attestation and vulnerability scan
   are not attached to the exact candidate digests.
2. **HIGH —** issue #61 remains open; production Prometheus attachment has not
   been proven for this candidate deployment.
3. **HIGH —** an independent production-like pilot/soak with resource-trend
   measurements, exact-candidate upgrade and failed-rollout recovery is not
   complete.
4. **BOUNDED —** stable-release multi-node O3K HA/resilience certification is
   intentionally excluded and remains gated by #106.

### Verdict

**NO-GO — NOT PRODUCTION READY**

The candidate is operationally demonstrable against the live certified
OpenStack/IdP profile, but the unresolved provenance, production observability
and exact-candidate pilot/upgrade gates prohibit a production-ready claim.
Do not publish a v1.0 tag. Issue #67 is not ready for final approval until
these findings are closed and the affected evidence is rerun at one exact
candidate HEAD.

`#106 remains OPEN — stable-release multi-node HA/O3K certification intentionally deferred.`
