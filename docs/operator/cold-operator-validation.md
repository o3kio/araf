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
| Install a published artifact and perform login/backend smoke | **blocked** | The exact published GHCR digest pull was attempted, but this environment has no registry authorization (`denied`); no source build was substituted. |
| Induce a live backend failure and execute rollback/recovery | **blocked** | Requires the external O3K/OpenStack and IdP deployment; no fixture or invented endpoint is acceptable evidence. |

The first documentation rehearsal exposed two navigation gaps: the root README
did not link the operator entry point, and the support-bundle redaction test did
not cover the unsafe arbitrary-log path. Both were fixed. The second rehearsal
and the automated docs/path/security gates pass on the same commit.

The live cold-operator acceptance remains **blocked**, rather than being marked
pass, until an independent operator can use published artifacts against a real
deployment for install, login/read-only smoke, one safe failure, bundle
collection and rollback/recovery.

A real cluster/IdP/O3K mutation rehearsal is intentionally outside this record:
the stable multi-node O3K acceptance environment belongs to issue #106.
