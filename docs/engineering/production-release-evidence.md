# Araf P4.7 production-release evidence

This record is the source of truth for the P4.7 decision. The live functional
run below was performed on 2026-09-15 against `1.0.0-rc.1`. The reviewed
`1.0.0-rc.2` artifacts were built after the frontend runtime security base was
refreshed. RC1 live results are not evidence that the RC2 frontend artifacts
were exercised; the two candidate identities and evidence are separated below.

## Candidate identity

| Item | Candidate |
| --- | --- |
| RC1 functional-test source SHA | `a8d0d0e8fd2e7f5b34a6355c2674bd26c2aa6f7d` |
| Candidate build source SHA | `c5af7cb0f2ad943774b42466c8c3617dfc64f9c0` |
| Candidate version | `1.0.0-rc.2` |
| BFF image | `ghcr.io/o3kio/araf-bff@sha256:72081d8c634b9ca3a16b9de558ce0e56b1fd9d77603e5e235819c706409b24cf` |
| Tenant console image | `ghcr.io/o3kio/araf-tenant-console@sha256:13f6b50a19430d069c9138aa1ee1e149ef6722c6b6c79ce5ee932fd8a96c80ec` |
| Operator console image | `ghcr.io/o3kio/araf-operator-console@sha256:1f283914e3434565739d891c98c9796cfef9df2bd1e3b9d0d39bb81580ed526a` |
| Chart | `deploy/helm/araf` chart `0.1.0` (`appVersion: 0.0.0`; image digests are supplied by release values) |
| Registry | GitHub Container Registry (`ghcr.io/o3kio`) |
| Build metadata | OCI revision labels on all three images match `c5af7cb` and `1.0.0-rc.2` |

The live functional run used the RC1 source SHA and local RC1 digests. The
attested RC2 images were built from the corrected candidate SHA above; all
three published digests passed the pinned Trivy HIGH/CRITICAL scan, BuildKit
SBOM generation and GitHub OIDC provenance attestation. Subsequent commits only
update evidence text and do not alter those image inputs. The base nginx image
was smoke-tested read-only with the mounted generated-config directory, and
the packaging gate verifies the equivalent Helm/Compose mounts. A local
rebuild of the RC2 tenant frontend source started read-only and served the
console with the expected security headers, but its image digest was
`sha256:2fc2e328811337daee6f40a9306a15fa1318021c2f5152292052edb9bebd1e45`,
not the attested RC2 digest above. It is source-level smoke evidence only. The
published RC2 frontend digests remain unexercised in the live HTTPS harness;
the RC1 live results are not substituted for them.

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

The RC1 live deployment was started from digest-pinned local OCI artifacts,
without building source in the target runtime. Four non-root, read-only BFF
containers (two Tenant and two Operator) shared encrypted durable session and
CompatibilityOperation paths. The deployment used an HTTPS reverse proxy and
Keycloak 26.3 (`araf-p28`) as the real IdP.

`/healthz`, `/readyz`, `/version` and `/metrics` returned successfully on both
surfaces. The live deployment reported `1.0.0-rc.1`. Tenant and Operator OIDC
login, callback, logout/session cookies and re-login were exercised over
HTTPS. No provider token was present in browser-visible responses. The
attested `1.0.0-rc.2` frontend digests have not completed this install/auth
journey; only the local source rebuild smoke noted above was run.

## Functional journeys

On RC1, the Tenant session discovered 30 capabilities, listed
Keystone/Nova/Glance/Neutron/Cinder resources and selected the
server-authoritative project scope.
A real OpenStack network create returned a durable CompatibilityOperation and
the resulting network was verified in authoritative Neutron before cleanup.
An invalid compute create was rejected with a bounded error. The deferred
`object.storage.bucket` capability returned HTTP 501 and did not expand the
support contract.

On RC1, the Operator session authenticated independently. A Tenant cookie
sent to an Operator route returned HTTP 401. Requests alternated across both
replicas; session and scope continuity remained intact after killing and
restarting one Tenant replica. This is Araf/OpenStack replica evidence only,
not O3K HA certification.

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
  RC1 live path. Existing release security negatives are recorded in
  [`security-release-evidence.md`](security-release-evidence.md); they were not
  all repeated against the RC2 frontend digests.
- `/healthz`, `/readyz`, `/version`, `/metrics`, request IDs and correlation
  IDs were collected. The support path is browser → BFF → adapter →
  CompatibilityOperation/authoritative OpenStack resource.
- `tests/support-bundle-security.sh` passed with synthetic OIDC token-like,
  bearer, session-key and password markers. The RC1 live bundle contained
  only version, readiness, metrics, environment names and a redaction notice;
  it contained no credentials, cookies, keys or tokens.

## Performance and pilot/soak

The RC1 deployment ran a 45-second two-Tenant/two-Operator authenticated
soak: 72 requests, zero failures, repeated context/service/resource/metrics
reads, and successful OIDC sessions. It did not include mutations, Operation
polling, or an injected backend outage/recovery within the soak workload.
Tenant RSS remained approximately
4.69–4.73 MiB and 4.80–4.81 MiB across replicas; the session store remained
bounded at 4,356 bytes and the compatibility journal remained empty because
the workload was read-only. This is a bounded development-host soak. The
reusable OpenStack and fixture baselines remain in
[`performance-evidence.md`](performance-evidence.md).

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
   RC1 live functional run. The published RC2 frontend digests have not been
   installed or exercised in the live HTTPS harness. The locally rebuilt
   tenant image is a different digest and does not close this finding. Pulling
   the published digest returned `unauthorized` because this environment's
   credential lacks package-read access; no registry permissions or
   credentials were changed to work around that restriction.
2. **MEDIUM —** the 45-second RC1 soak was read-heavy and did not cover the
   required mutation, Operation-polling, or injected backend-failure/recovery
   workload in the soak itself. Separate restart, rollback and failed-readiness
   checks passed, but they do not substitute for that representative workload.
3. **BOUNDED —** stable-release multi-node O3K HA/resilience certification is
   intentionally excluded and remains gated by #106.

## Review convergence

Six comprehensive reviews have been performed through 2026-09-16. This latest
review found that several RC1 live results were described as candidate/RC2
results; the evidence above now labels those runs as RC1 and records the
different-digest RC2 source smoke separately. It also identifies the soak's
missing mutation, Operation-polling and backend-failure/recovery workload.
Earlier reviews fixed the release-gate portability issue, frontend runtime
security and read-only nginx mounts, candidate identity, and a stale
packaging statement. The remaining HIGH and MEDIUM release findings are
listed above. No clean pass #1 or #2 (`B0/H0/M0/L0`) has occurred on this
corrected evidence head; CI and review convergence must restart after this
change. The PR must not be merged as a production release until the candidate
artifact and representative soak gates are completed and reviewed.

### Verdict

**NO-GO — NOT PRODUCTION READY**

RC1 is operationally demonstrable against the live certified OpenStack/IdP
profile, but RC2 still lacks live acceptance of its exact frontend artifacts.
The bounded read-heavy soak also lacks the required mutation, Operation-polling
and backend-failure/recovery coverage. These findings prohibit a
production-ready claim.
Do not publish a v1.0 tag. Issue #67 is not ready for final approval until the
findings are closed and the affected evidence is rerun at one exact candidate
HEAD.

`#106 remains OPEN — stable-release multi-node HA/O3K certification intentionally deferred.`
