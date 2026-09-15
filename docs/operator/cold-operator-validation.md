# Cold-operator validation record

This record separates a documentation rehearsal from the live acceptance that
P4.6 requires. The tester started from a clean shell with no maintainer
conversation and was given the repository plus [`operator/README.md`](README.md)
only. No production credentials were copied into the repository or bundle.

| Task | Result | Evidence |
| --- | --- | --- |
| Identify release artifacts | pass | `installation.md` points to OCI digests, Helm chart and Compose release files. |
| Select and configure a deployment | pass | `configuration.md` and `secrets.md` enumerate required values and custody. |
| Determine health/readiness | pass | `observability.md` gives private BFF probe commands and `/metrics` checks. |
| Diagnose a failed VM create | pass | The decision tree follows capability → BFF/request ID → backend response → Operation. |
| Collect a safe diagnostic bundle | pass | `support-bundle.md`; `tests/support-bundle-security.sh` removes synthetic markers. |
| Install published artifacts and perform login/backend smoke | pass | The exact release BFF digest `sha256:c1aabc13a2edc9fc2bc1e86780a2d57eef5a6e1503f3cf82e38a2bdbe74a7f61` was run from the local OCI registry. Both BFFs were ready; the published tenant/operator console artifacts served successfully and proxied unauthenticated API requests to their matching BFFs. |
| Login and select a real backend scope | pass | A real Keycloak realm user completed authorization-code + PKCE login; Keystone returned the configured project and Araf returned a 26-capability OpenStack context. No token or secret was recorded. |
| Induce a live failure and diagnose it | pass | Requesting the explicitly deferred `object.storage.bucket` capability returned a correlated `501` with `OpenStack capability is unavailable`; the diagnosis followed the capability/deferred-feature branch, not an outage assumption. |
| Execute rollback/recovery | pass | The operator BFF was stopped, restarted from the previous local release digest `sha256:3c06cb7c3458a53f4154bee31a24785d3f64c5dc136c890c70b3574814fb0574`, passed `/readyz`, and was restored to the approved digest with the same durable session/journal paths and key. |

The first documentation rehearsal exposed two navigation gaps: the root README
did not link the operator entry point, and the support-bundle redaction test did
not cover the unsafe arbitrary-log path. Both were fixed. The second rehearsal
and the automated docs/path/security gates pass on the same commit.

The live cold-operator acceptance passed against the local deployed Keycloak and
OpenStack services using only the operator documentation and published local
OCI artifacts. The temporary validation proxy used a fresh short-lived
certificate because the pre-existing local proxy certificate was expired; no
application or cloud resource was mutated.

The O3K native service was not running in this environment. O3K stable-release
and multi-node HA acceptance therefore remains intentionally deferred to
[#106](https://github.com/o3kio/araf/issues/106); this record is not O3K
certification evidence.
