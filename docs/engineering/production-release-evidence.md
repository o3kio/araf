# Araf P4.7 production-release evidence

This record is the source of truth for the P4.7 decision. RC1 live functional
results below were collected on 2026-09-15. Exact published RC2 OCI digests
were pulled and exercised in an isolated HTTPS/IdP/OpenStack harness on
2026-09-16; that evidence is recorded separately below. RC1 results do not
close RC2 acceptance criteria.

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

The live functional run on 2026-09-15 used the RC1 source SHA and RC1 digests.
The attested RC2 images were built from the corrected candidate SHA above;
all three published digests passed the pinned Trivy HIGH/CRITICAL scan,
BuildKit SBOM generation and GitHub OIDC provenance attestation. The RC2
harness pulled and ran these exact GHCR digests; it did not rebuild or
substitute images. A prior local rebuild of the RC2 tenant frontend source
(`sha256:2fc2e328811337daee6f40a9306a15fa1318021c2f5152292052edb9bebd1e45`)
was a different digest and is not used as RC2 artifact evidence. The base
nginx read-only smoke and Helm/Compose mount checks remain packaging evidence,
not live RC2 Helm deployment evidence.

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

## Exact RC2 live harness — 2026-09-16

### Deployment identity and topology

The test pulled the exact published digests in the candidate table directly
from GHCR after `read:packages` access became available. Docker image metadata
reported version `1.0.0-rc.2` and source revision
`c5af7cb0f2ad943774b42466c8c3617dfc64f9c0`. No application image was built
locally or substituted. The BFF and both console images were run by digest.

The isolated development-host harness used one Tenant console, one Operator
console, two Tenant BFF replicas and two Operator BFF replicas, with separate
durable state paths per surface. A private test network connected them to an
HTTPS reverse proxy and a dedicated HTTPS OpenStack proxy to the host's
certified OpenStack 2026.1 Gazpacho services. Keycloak 26.3 (`araf-p28`) was
the OIDC provider. Each surface used its documented OpenStack credentials and
server-side session configuration; secret values are intentionally omitted.
Only loopback ports were published. Test TLS used an ephemeral local CA and a
browser SPKI pin, so this proves HTTPS application behavior, not public-CA
trust or production certificate lifecycle. This run used direct containers,
not Helm; it does not add Helm-install evidence.

The Tenant and Operator BFF `/version` endpoints reported the expected RC2
version and source SHA; both BFF surfaces' `/healthz`, `/readyz`, `/version`,
and `/metrics` endpoints were reachable. FixtureBackend was not selected. Both
OIDC callback flows completed over HTTPS; session cookies were distinct,
`Secure`, and `HttpOnly`. Keystone scope discovery and project selection
succeeded. A Tenant session presented to an Operator API returned 401. Request
and correlation IDs were present. The browser made zero direct OpenStack API
requests and held zero localStorage/sessionStorage keys in the observed
journeys.

### Exact artifact behavior and failures

Both published console bundles were blocked by the CSP served with the exact
RC2 frontend images. The response included
`script-src 'self' 'strict-dynamic'` without a nonce or hash; Chromium reported
the external `/assets/index-*.js` module blocked because `strict-dynamic`
disables the host allow-list. The application shell HTML and CSS loaded, but
the JavaScript application did not. The delivered policy also blocked its
embedded `data:` font with `font-src 'self'`. OIDC/API requests could be
exercised through the same authenticated browser origin, but the consoles are
not usable as shipped.

One real Neutron network create was accepted at `/v2.0/networks` (HTTP 201),
and the BFF returned a durable `openstack-compat-*` CompatibilityOperation.
RC2 then polled the operation and attempted the authoritative resource read at
`/networks/{id}` (omitting `/v2.0`); Neutron returned 404. The operation
remained `unknownOutcome`. Araf's delete path had the same missing version
prefix and returned 404. A direct authoritative GET at `/v2.0/networks/{id}`
confirmed the disposable network existed; it was removed through that
authoritative Neutron endpoint (HTTP 204). The RC2 journal retained the
unknown create and failed delete records. No further create was submitted, to
avoid leaving provider resources behind. This is a release-blocking mutation
and reconciliation defect in the exact published BFF digest, not a harness
failure.

### Supplementary soak, outage, restart and resource observations

After the mutation failure, a 360-second supplementary read-only soak ran ten
cycles of authenticated session, context/scope, bounded Neutron list and
metrics checks. It is not the required mutation-inclusive representative
soak: successful mutations were zero, and Operation polling only established
that the failed create remained `unknownOutcome` (the smoke poller timed out
after 24 reads). Each cycle succeeded outside the controlled outage/restart
boundaries. Stopping only the OpenStack proxy made a Neutron list return 502;
restarting the proxy restored the request to 200 while HTTPS ingress and the
IdP remained available. This was sequential functional coverage on a
development host, not a concurrent load or throughput test; no RC2 load
threshold is claimed.

After SIGKILL of one Tenant BFF, the first session read returned 504 and the
next read through the surviving replica returned 200. The killed replica
restarted and became ready; its shared session and unknown CompatibilityOperation
were still readable. SIGKILL/restart of one Operator BFF also retained its
authenticated session. These results demonstrate state continuity but also
show a transient request failure at abrupt replica loss; they are Araf with
OpenStack evidence only and are not O3K HA evidence.

Across the ten read cycles, Tenant session-store size stayed 10,178 bytes,
Operator session-store size stayed 7,635 bytes, and the Tenant journal stayed
968 bytes/two records (the failed create and failed delete). Tenant BFF RSS
varied from approximately 16.68–20.26 MiB per process, Operator BFF RSS from
13.92–18.61 MiB; final file-descriptor counts were 11–12 per Tenant BFF and
10 per Operator BFF. RSS varied around the replica restart; no monotonic
unbounded resource or durable-state growth was observed in this short,
development-host run. Metrics exposed `araf_bff_requests_total`; request and
correlation IDs remained available. Generated test credentials and sessions
remained in the temporary test environment and process memory; no credentials,
tokens, cookies, keys, or passwords were included in the evidence or emitted in
the test report.

No RC2 O3K journey was run in this harness. Existing O3K convergence evidence
is not substituted for RC2 OpenStack results or #106.

## Install and authentication

The RC1 live deployment was started from digest-pinned local OCI artifacts,
without building source in the target runtime. Four non-root, read-only BFF
containers (two Tenant and two Operator) shared encrypted durable session and
CompatibilityOperation paths. The deployment used an HTTPS reverse proxy and
Keycloak 26.3 (`araf-p28`) as the real IdP.

`/healthz`, `/readyz`, `/version` and `/metrics` returned successfully on both
surfaces. The live deployment reported `1.0.0-rc.1`. Tenant and Operator OIDC
login, callback, logout/session cookies and re-login were exercised over
HTTPS. No provider token was present in browser-visible responses. The exact
RC2 HTTPS/OIDC login and API-origin session tests are recorded in
the dedicated RC2 section above; the RC1 statements in this subsection are
not RC2 evidence.

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

The following bullets describe the RC1 live run unless an RC2 scope is stated
explicitly.

- Tenant-to-Operator access was rejected server-side (HTTP 401); this was not
  UI-only hiding.
- Missing Object Storage capability was distinguished from outage (HTTP 501).
- Invalid mutation returned a bounded problem response with correlation data;
  no automatic replay was attempted.
- HTTPS, OIDC state/PKCE, CSRF and cookie separation were exercised in the
  RC1 live path. Exact RC2 HTTPS/OIDC/cookie and Tenant-to-Operator negative
  results are recorded above. Existing release security negatives are in
  [`security-release-evidence.md`](security-release-evidence.md); the full #64
  negative suite was not repeated on the exact RC2 artifacts here.
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

The following is RC1-only rollout evidence; it was not repeated against the
exact published RC2 digests.

The previous BFF digest (`sha256:c1aabc13…`, source revision
`24a8b691…`) was started against the candidate durable state and returned
ready; it was then replaced by the exact candidate digest, which also returned
ready. A deliberately invalid session key caused a new replica to exit before
readiness while the healthy candidate continued serving (`/readyz` 200).
This validates the documented tested-state rollback and failed-rollout path;
it is not a claim for untested state formats or external Helm orchestration.

## Findings and decision

The following are release-gate findings, not hidden limitations:

1. **HIGH —** the CSP on both exact RC2 frontend images blocks the application
   JavaScript bundle (and the embedded font), so neither console is usable as
   shipped.
2. **BLOCKER —** the exact RC2 OpenStack adapter omits Neutron's `/v2.0` prefix
   in resource detail and delete requests. A real created network therefore
   cannot be reconciled or deleted through Araf; its CompatibilityOperation
   remains `unknownOutcome` while authoritative Neutron state contains the
   resource. The disposable test network required direct Neutron cleanup.
3. **MEDIUM —** abrupt Tenant replica loss caused one session GET to return
   504 before a retry succeeded on the surviving replica. Session and journal
   continuity passed after restart, but transient request availability is
   visible.
4. **HIGH —** the required mutation-inclusive RC2 soak did not pass. One create
   was attempted and exposed the blocker above; no successful mutations were
   counted, so the supplementary 360-second read-only soak cannot satisfy
   mutation/polling acceptance.
5. **HIGH —** additional mandatory P4.7 candidate acceptance remains
   unverified: no RC2 native O3K functional journey; the full release-critical
   #64 negative suite and the OpenStack invalid/quota/401/403/404/409/5xx
   matrix were not repeated on RC2; and RC2 was not exercised through Helm
   install, upgrade/rollback/failed rollout, or a cold-operator support
   rehearsal. RC1, P3.9 and prior convergence evidence do not substitute for
   these exact-candidate checks.
6. **MEDIUM —** no RC2 concurrent load/scale run or candidate-level
   performance thresholds were recorded; the short sequential soak is not
   load evidence.
7. **BOUNDED —** stable-release multi-node O3K HA/resilience certification is
   intentionally excluded and remains gated by #106.

### Post-RC2 source remediation status (not RC2 artifact evidence)

After the exact-digest run, this PR's source was updated to remove the
`strict-dynamic` CSP incompatibility and to preserve complete OpenStack
collection paths for detail/delete URLs, including Neutron's `/v2.0` prefix;
resource and action IDs are also validated and appended as escaped URL path
segments. Regression tests cover CSP compatibility and Neutron item URLs.
These edits are not present in the published RC2 digests above. No corrected
candidate image was built or published for this run, and the source changes
do not close any RC2 release finding or receive artifact-acceptance credit.

The next candidate must use newly published digests from a new source SHA;
the HTTPS/OIDC harness, UI bootstrap, Neutron mutation/reconciliation, soak,
failure/recovery and security gates must then be repeated against those exact
digests. RC2 remains the only live-artifact evidence recorded here.

## Review convergence

Earlier reviews fixed the release-gate portability issue, frontend runtime
security source, read-only nginx mounts, candidate identity, and stale
packaging/evidence statements. The exact RC2 run on this head identified the
remaining BLOCKER/HIGH/MEDIUM findings above. The source and evidence changes
reset review convergence to zero. The exact RC2 acceptance assessment
remains `B1/H3/M2/L0`; clean pass #1 and clean pass #2 are both not achieved.
Code-level tests of the unbuilt source edits cannot clear findings on the
immutable RC2 artifacts. Release review can only converge after a corrected
candidate is published and the exact replacement digests are re-tested.
Neither the release findings nor the review count may be waived to promote
this candidate.

### Verdict

**NO-GO — NOT PRODUCTION READY**

Exact published RC2 artifacts were exercised, but their frontend CSP blocks
both consoles, and the OpenStack Neutron mutation path leaves provider-created
resources unreconciled and undeletable through Araf. The mutation-inclusive
soak therefore failed; the supplementary read-only soak does not offset these
findings. No production-ready claim is justified.
Do not publish a v1.0 tag. Issue #67 is not ready for final approval until the
findings are closed and the affected evidence is rerun at one exact candidate
HEAD.

`#106 remains OPEN — stable-release multi-node HA/O3K certification intentionally deferred.`
