# Araf P4.7 production-release evidence

This record is the source of truth for the P4.7 decision. The live functional
run below was performed on 2026-09-15; the reviewed release head subsequently
changed when the frontend runtime security base was refreshed, so the affected
candidate artifacts and acceptance evidence are explicitly separated below.

## Candidate identity

| Item | Candidate |
| --- | --- |
| Functional-test source SHA | `a8d0d0e8fd2e7f5b34a6355c2674bd26c2aa6f7d` |
| Candidate build source SHA | `c5af7cb0f2ad943774b42466c8c3617dfc64f9c0` |
| Documentation/review HEAD | Documentation-only commits after the candidate build (final SHA is recorded in the handoff report) |
| Candidate version | `1.0.0-rc.2` |
| BFF image | `ghcr.io/o3kio/araf-bff@sha256:72081d8c634b9ca3a16b9de558ce0e56b1fd9d77603e5e235819c706409b24cf` |
| Tenant console image | `ghcr.io/o3kio/araf-tenant-console@sha256:13f6b50a19430d069c9138aa1ee1e149ef6722c6b6c79ce5ee932fd8a96c80ec` |
| Operator console image | `ghcr.io/o3kio/araf-operator-console@sha256:1f283914e3434565739d891c98c9796cfef9df2bd1e3b9d0d39bb81580ed526a` |
| Chart | `deploy/helm/araf` chart `0.1.0` (`appVersion: 0.0.0`; image digests are supplied by release values) |
| Registry | GitHub Container Registry (`ghcr.io/o3kio`) |
| Build metadata | OCI revision labels on all three images match `c5af7cb` and `1.0.0-rc.2` |

The live functional run used the original test SHA and local digests. The
attested rc.2 images were built from the corrected candidate SHA above; all
three published digests passed the pinned Trivy HIGH/CRITICAL scan, BuildKit
SBOM generation and GitHub OIDC provenance attestation. Subsequent commits only
update evidence text and do not alter those image inputs. The base nginx image
was smoke-tested read-only with the mounted generated-config directory, and
the packaging gate verifies the equivalent Helm/Compose mounts. A live
Prometheus/frontend attachment against the private rc.2 digests is still
required before exact-head approval; the prior live results are not silently
substituted.

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
surfaces. The original live deployment reported `1.0.0-rc.1`; the attested
candidate identity for approval is `1.0.0-rc.2` and the exact source SHA above.
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

The exact candidate ran a 45-second two-Tenant/two-Operator authenticated
soak: 72 requests, zero failures, repeated context/service/resource/metrics
reads, and successful OIDC sessions. Tenant RSS remained approximately
4.69–4.73 MiB and 4.80–4.81 MiB across replicas; the session store remained
bounded at 4,356 bytes and the compatibility journal remained empty because
the workload was read-only. This is a bounded development-host soak, not an
independently operated production pilot. The reusable OpenStack and fixture
baselines remain in [`performance-evidence.md`](performance-evidence.md).

## Upgrade, rollback and failed rollout

The previous BFF digest (`sha256:c1aabc13…`, source revision
`24a8b691…`) was started against the candidate durable state and returned
ready; it was then replaced by the exact candidate digest, which also returned
ready. A deliberately invalid session key caused a new replica to exit before
readiness while the healthy candidate continued serving (`/readyz` 200).
This validates the documented tested-state rollback and failed-rollout path;
it is not a claim for untested state formats or external Helm orchestration.

## Findings and decision

The following deviations are release-gate findings, not hidden limitations:

1. **HIGH —** the frontend runtime security fix changed the source after the
   live functional run; exact-head frontend acceptance has not yet been rerun
   against the published digests.
2. **MEDIUM —** the candidate soak was bounded to 45 seconds on the
   development host and was not an independently operated production pilot.
   Exact-candidate restart, rollback and failed-readiness recovery did pass.
3. **BOUNDED —** stable-release multi-node O3K HA/resilience certification is
   intentionally excluded and remains gated by #106.

## Review convergence

Five comprehensive reviews were performed. The final review included the
release-gate portability fix, the frontend runtime security and read-only nginx
mount fixes, the rc.2 artifact identity refresh, and a stale historical
packaging statement that was corrected. No unresolved B0/M0/L0 implementation
or documentation defect remains. The runtime change still invalidated the
prior frontend candidate evidence, and the private attested frontend digests
have not been exercised in the live harness. Exact-head clean pass #1 and
clean pass #2 (`B0/H0/M0/L0`) were therefore **not achieved**; the PR must not
be merged as a production release until the affected live gate is rerun and
the remaining pilot limitation is accepted or resolved.

### Verdict

**NO-GO — NOT PRODUCTION READY**

The original candidate is operationally demonstrable against the live
certified OpenStack/IdP profile, but the reviewed head still lacks exact-head
frontend acceptance and trusted artifact evidence; the lack of an
independently operated production pilot also remains. These findings prohibit
a production-ready claim. Do not publish a v1.0 tag. Issue #67 is not ready
for final approval until the findings are closed and the affected evidence is
rerun at one exact candidate HEAD.

`#106 remains OPEN — stable-release multi-node HA/O3K certification intentionally deferred.`
