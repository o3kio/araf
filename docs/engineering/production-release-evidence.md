# Araf P4.7 production-release evidence

This record is the source of truth for the P4.7 decision. RC1 live functional
results below were collected on 2026-09-15. Exact published RC2 OCI digests
were pulled and exercised in an isolated HTTPS/IdP/OpenStack harness on
2026-09-16; that evidence is recorded separately below. Later RC4/RC5
artifacts have build-integrity and narrowly scoped process-smoke evidence only;
they have not received the RC2 live acceptance workload. The corrected RC12
candidate below is the first exact published candidate with a live
mutation-inclusive DevStack run; RC1 and partial RC5 checks do not close
candidate acceptance criteria.

## RC2 full-run candidate identity

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
and `/metrics` endpoints were reachable. FixtureBackend was not selected. The
BFF OIDC callback/session paths completed over HTTPS; session cookies were
distinct, `Secure`, and `HttpOnly`. The frontend JavaScript was blocked by
CSP, so this does not establish a usable interactive console login. Keystone
scope discovery and project selection succeeded. A Tenant session presented
to an Operator API returned 401. Request and correlation IDs were present.
The browser made zero direct OpenStack API requests and held zero
localStorage/sessionStorage keys in the observed journeys.

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

## Corrected RC12 exact DevStack acceptance — 2026-09-16

RC12 is the candidate published from the source correction that repairs
status-less Neutron create reconciliation and initializes the non-root BFF
state directory in the image. The exact source is `de64cc9193085116fa30ad51c04ccab24a013dd0`;
the release workflow was run successfully and all images were pulled by their
published digests (no local rebuild or substitution):

| Artifact | Exact digest |
| --- | --- |
| BFF | `ghcr.io/o3kio/araf-bff@sha256:bc717ecdbbbf3ea673efe168c90419936677d644aa0ae25af4eb84906cd744ba` |
| Tenant console | `ghcr.io/o3kio/araf-tenant-console@sha256:25f5fe41927f68db3dafd49597c6b8cb4520bca2dd45ec131d1372474ef3e5e5` |
| Operator console | `ghcr.io/o3kio/araf-operator-console@sha256:cbbad76033eced4d4290c9848e0665a23c30bd4a7c647077ba0cf03150666b18` |

The isolated acceptance cloud was the disposable libvirt VM
`p14-openstack-source`: Ubuntu 24.04.4, 8 vCPU, 20 GiB RAM, 130 GiB qcow2,
nested KVM/libvirt, and DevStack stable/2026.1 at commit
`da2f4d73f5ad74fc8ecfbe15bd7e20f6b0982dbb`. Keystone, Nova, Placement,
Glance, Neutron/OVN and Cinder were enabled; Swift was not enabled or
advertised. This DevStack run is development/CI evidence for the exact Araf
artifacts, not certification of the matched Kolla production deployment.
The latter remains covered by the Kolla 2026.1 evidence referenced above.

An isolated project `araf-rc5-acceptance` used an ordinary member user with
quotas of 4 instances, 4 vCPU, 8,192 MiB RAM, 4 networks, 8 subnets, 32 ports,
4 volumes and 20 GiB. Passwords and tokens were held only in mode-0600 local
secret files and are not part of this record.

The exact RC12 HTTPS harness used Keycloak 26.3.5 and a short-lived local TLS
CA. Tenant login, session/context discovery and the separate Operator surface
completed over HTTPS; cookies were Secure, HttpOnly where appropriate and
surface-specific. Browser storage held no provider credentials. The BFFs
reported the candidate version/revision and `/healthz`, `/readyz`, `/version`
and `/metrics` were reachable. A Tenant request to an Operator route was
denied server-side (404), CSRF without its token was denied (403), and a
foreign resource read was denied (404).

The mandatory Neutron regression passed with the exact RC12 images: Network create
returned a durable CompatibilityOperation and reconciled to `succeeded` from
authoritative `ACTIVE`; Subnet create reconciled to `succeeded` with the
explicit event `Observed authoritative resource: PRESENT`; both resources
were then deleted and reconciled to `succeeded`. Keystone catalog discovery
advertised 30 capabilities and all certified list paths returned 200. Cinder
create/show/delete and Nova create/stop/start/delete passed with authoritative
provider states. A Nova reboot also eventually reconciled to `succeeded` after
the DevStack guest completed its long QEMU reboot; the operation was polled
for up to three minutes, and no mutation was replayed.

A 64.0-second exact-candidate soak executed 300 meaningful requests: repeated
session/context/service and bounded resource reads, 25 real Network
create/read/delete cycles and CompatibilityOperation polling. All 300
requests completed successfully (p50 140.9 ms, p95 689.1 ms, p99 887.4 ms).
An injected Neutron API stop produced bounded HTTP 502 responses, and
restarting the service restored the same list request to HTTP 200. Restarting
the Tenant BFF after durable state existed retained the session and journal;
the post-restart HTTPS login and Network/Subnet mutation smoke passed again.
Tenant BFF RSS was approximately 15 MiB before and after the soak, with 13
file descriptors; the journal and session files grew only with the expected
operation/session records and showed no unexplained monotonic growth.

The initial RC12 run exposed a harness configuration issue: Keystone’s public
catalog used HTTP while the HTTPS-only Araf client correctly rejected mixed
schemes. Explicit HTTPS service URLs were then supplied for the isolated
proxy (compute, image, networking and volume); no application source or image
was changed. This is recorded as harness setup, not a product defect.

RC12 has not been promoted. Live upgrade/rollback from a previously accepted
candidate was not completed because the prior RC5 candidate had known
release-blocking defects; generic #65 package/upgrade checks are not being
represented as this live gate. Stable O3K release and multi-node HA evidence
was not attempted. `#106 remains OPEN — stable-release multi-node HA/O3K
certification intentionally deferred.`

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
7. **HIGH —** the RC2 backend accepted insecure/mismatched service endpoint
   schemes and followed redirects while an auth header was attached. No
   credential leak was observed, but RC2 could forward credentials to an
   untrusted endpoint. The fix is present in RC5 source; exact RC2 remains
   affected, and live RC5 backend requests have not verified the mitigation.
8. **BOUNDED —** stable-release multi-node O3K HA/resilience certification is
   intentionally excluded and remains gated by #106.

### Post-RC2 source remediation status (not RC2 artifact evidence)

The source after RC2 was updated to remove the `strict-dynamic` CSP
incompatibility and to preserve complete OpenStack
collection paths for detail/delete URLs, including Neutron's `/v2.0` prefix;
resource and action IDs are also validated and appended as escaped URL path
segments. Regression tests cover CSP compatibility and Neutron item URLs.
A further source audit found a credential-forwarding risk: configured or
catalog-discovered OpenStack service URLs could use HTTP even when Keystone
used HTTPS, and Reqwest followed redirects while carrying `X-Auth-Token`.
This was not observed as a leak in the RC2 harness, but the insecure endpoint
path was a HIGH finding.

Source SHA `83c27b67c20777cd10317836c004b23802d83fa9` validates credential-free
HTTP(S) Keystone URLs, requires service endpoints to match Keystone's scheme,
ignores insecure/credentialed catalog endpoints, and disables redirects on
the OpenStack client. Regression tests cover scheme/credential validation and
non-followed redirects. These fixes are not present in RC2 and cannot
retroactively clear its findings.

#### Later published artifacts — not full runtime acceptance

The official RC4 workflow (run
[`35082217433`](https://github.com/o3kio/araf/actions/runs/35082217433))
published exact digests from source
`d82ccc7d519da28dbed737e47ec4cb8477509267`:

| RC4 artifact | Exact digest |
| --- | --- |
| BFF | `ghcr.io/o3kio/araf-bff@sha256:8ec771e25db80cafa5294407727372fa1654b3adfe08fa2122e377c1ba1c884a` |
| Tenant console | `ghcr.io/o3kio/araf-tenant-console@sha256:94a5c812b409b582184a4230143aaac32abeb831ef2a8e20c2d41c2d7bf9b918` |
| Operator console | `ghcr.io/o3kio/araf-operator-console@sha256:2ce5b5b119a970a1d743d463aee2a6b97dc38623e02560a98af142529c66eea9` |

The official RC5 workflow (version `1.0.0-rc.5`, run
[`35085223179`](https://github.com/o3kio/araf/actions/runs/35085223179))
published source SHA `83c27b67c20777cd10317836c004b23802d83fa9`. Its exact
artifacts were pulled by digest and their OCI version/revision metadata
verified:

| RC5 artifact | Exact digest |
| --- | --- |
| BFF | `ghcr.io/o3kio/araf-bff@sha256:a150796f1d7f58af632d0bd4d480655df7bf8f3b7d2f5805cff50448feb4d9f6` |
| Tenant console | `ghcr.io/o3kio/araf-tenant-console@sha256:2aeb4bda3f35c36b117989b4932f99a553e603d01888322d479025bb08c756f9` |
| Operator console | `ghcr.io/o3kio/araf-operator-console@sha256:0691d1402563b66b7d4f1a66651fbac546408edd0791747127222ae884524a37` |

RC4/RC5 official gates completed successfully, including the configured
security/package checks, image HIGH/CRITICAL scans, SBOM generation and
provenance attachment. RC5 exact-digest process smokes verified that both
console roots and JS bundles return HTTP 200 with the expected CSP, that BFF
health/readiness/version return HTTP 200 with the RC5 identity, and that an
insecure HTTP Neutron URL is rejected at startup. These are artifact and narrow
process/config checks—not production HTTPS/IdP/OpenStack deployment,
successful mutation, failure-matrix, or mutation-inclusive soak evidence.
RC4 was not run through live acceptance either.

The RC2-only issues and measurements above remain attributed to RC2. They
must not be projected as confirmed defects in RC5, but source fixes and process
smokes do not prove the fixed UI or OpenStack mutation/reconciliation flows
work in RC5. A read-only local check found no usable stored Kolla cloud
credential. No shared Keystone credential, database, or password was changed
to work around this. A disposable least-privilege project credential is
required to finish live candidate acceptance.

## Current RC5 acceptance status

### RC5 DevStack acceptance harness (2026-09-16)

RC5 exact published images were pulled by digest from GHCR and run without a
local rebuild. The isolated OpenStack environment was the disposable libvirt
VM `p14-openstack-source`: Ubuntu 24.04 Noble, 8 vCPU, 20 GiB RAM, 130 GiB
disk, nested KVM enabled, and DevStack `stable/2026.1` at commit
`da2f4d73f5ad74fc8ecfbe15bd7e20f6b0982dbb`. It exposed the certified Keystone,
Nova/Placement, Glance, Neutron and Cinder services; Swift and public
floating-IP workflows were not enabled. This is development/CI evidence and
does not replace the matched Kolla-Ansible 2026.1 production-profile record.

The least-privilege project `araf-rc5-acceptance` and member-only user
`araf-rc5` were provisioned locally with bounded quotas (3 instances, 4
cores, 4 GiB RAM, 3 networks, 5 subnets, 20 ports, 3 volumes/10 GiB). No
credential values are recorded. Direct CLI checks passed Keystone token and
catalog discovery, CirrOS image availability, Nova create/ACTIVE/stop/start/
reboot/delete, Neutron network/subnet/port create/delete and Cinder volume
create/show/delete.

The exact RC5 HTTPS/Keycloak 26.3 harness authenticated a Tenant session over
TLS, returned only the configured project scope, kept provider tokens out of
browser storage, and used Secure/HttpOnly surface-specific cookies. Exact RC5
Network create reconciled to Neutron `ACTIVE`. Exact RC5 Subnet create exposed
a release blocker: Neutron returned an authoritative project-scoped subnet
without a lifecycle `status`, while the CompatibilityOperation remained
`running` and recorded `UNKNOWN`. The resource was then removed through the
authoritative Neutron path. This is ordinary supported functionality, not a
#106 scenario; RC5 therefore remains NO-GO and its mutation-inclusive soak is
invalidated. A correction was made in source after this run and requires a
newly published candidate before any acceptance result can be reused.

The fresh Compose state-volume probe also found that an uninitialized named
volume is root-owned while the exact RC5 non-root BFF (UID 65532) cannot write
it. RC5 required a manual ownership repair before login. This is a clean-
install packaging HIGH. The follow-up source change seeds `/var/lib/araf` in
the image as UID/GID 65532 and sets the Helm pod `fsGroup`; no RC5 artifact was
modified or substituted. No RC5 soak, outage/recovery, restart, upgrade or
rollback claim is made beyond these recorded observations.

RC2 remains the only full live-artifact acceptance run. No RC5 HTTPS/OIDC
login, OpenStack create/read/update/delete journey, representative mutation
and Operation/CompatibilityOperation polling soak, controlled backend
outage/recovery run, or production-topology deployment has been completed.
The prior RC2 outage/restart and short resource observations are historical
RC2-only evidence, not RC5 proof. RC1/P3.9 and convergence records do not
substitute for RC5 candidate acceptance.

1. **BLOCKER (RC2 artifact) —** the exact RC2 Neutron item-path defect left a
   real provider resource unreconciled and undeletable through Araf. The
   source fix is in RC5, but the real mutation journey has not verified it.
2. **HIGH (RC2 artifact) —** the exact RC2 frontend CSP blocked both console
   applications. RC5 root/JS process smoke passed, but authenticated browser
   journeys on exact RC5 images remain unverified.
3. **HIGH (RC2 artifact/source audit) —** RC2 allowed a service endpoint
   scheme mismatch and redirect-following with an auth header. This was not an
   observed leak; it is fixed in RC5 source and covered by tests and an
   insecure-config startup smoke, but live RC5 backend requests have not
   exercised the protections.
4. **HIGH (candidate acceptance) —** no successful RC5 mutation-inclusive
   soak exists. The RC2 attempt had zero successful mutations and failed on
   the Neutron path; its read-only supplementary soak is insufficient.
5. **HIGH (candidate acceptance) —** required exact-candidate evidence is
   absent for the full #64 negative suite, OpenStack invalid/quota/401/403/
   404/409/5xx matrix, HTTPS/OIDC end-to-end flows, Helm install and rollout
   recovery, and independent cold-operator support rehearsal.
6. **MEDIUM (candidate acceptance) —** there is no RC5 concurrent load/scale
   run or candidate-level performance threshold result.
7. **MEDIUM (RC2 observation) —** abrupt RC2 Tenant replica loss caused one
   504 before retry success; RC5 restart continuity and request behavior have
   not been measured.

Source/workspace tests and exact-head CI are regression evidence, but do not
discharge missing published-artifact runtime tests. Do not promote, mark the
PR ready, or merge until production-support claims have candidate-specific
results and no release-critical BLOCKER/HIGH remains.

## Review convergence

The source remediation commit and this evidence update reset review
convergence to zero. Exact RC2 artifact-specific findings are
`B1/H4/M2/L0`; the current-candidate release gate remains blocked by
unverified RC5 functionality and acceptance coverage above. Clean pass #1 and
clean pass #2 are **not achieved**. A reviewer cannot truthfully record
`B0/H0/M0/L0` while release-critical candidate checks remain unproven. No
review count or finding has been waived to promote the candidate.

### Verdict

**NO-GO — NOT PRODUCTION READY**

The exact RC2 artifacts exposed release-blocking frontend and Neutron defects.
Source fixes have since been published in RC5, but RC5 only has artifact
integrity and limited process/config smoke evidence—not an HTTPS/IdP/OpenStack
journey or the required successful mutation-inclusive soak. No production-
ready claim is justified until exact RC5 acceptance is completed and reviewed.
Do not publish a v1.0 tag. Issue #67 is not ready for final approval until the
findings are closed and the affected evidence is rerun at one exact candidate
HEAD. Review convergence is currently `0/2`.

`#106 remains OPEN — stable-release multi-node HA/O3K certification intentionally deferred.`
