# Cold-operator validation record

This record captures a documentation-only cold-operator rehearsal. The tester
started from a clean shell with no maintainer conversation and was given the
repository plus [`operator/README.md`](README.md) only. No production
credentials or cloud endpoint were used.

| Task | Result | Evidence |
| --- | --- | --- |
| Identify release artifacts | pass | `installation.md` points to OCI digests, Helm chart and Compose release files. |
| Select and configure a deployment | pass | `configuration.md` and `secrets.md` enumerate required values and custody. |
| Determine health/readiness | pass | `observability.md` gives `/healthz`, `/readyz`, `/version` and `/metrics` checks. |
| Intentionally cause an upstream failure | pass | `troubleshooting.md` uses an unreachable endpoint and explains readiness/mutation ambiguity. |
| Collect a safe diagnostic bundle | pass | `support-bundle.md`; `tests/support-bundle-security.sh` removes synthetic markers. |
| Diagnose a failed VM create | pass | The decision tree follows capability → BFF/request ID → backend response → Operation. |
| Plan rollback/recovery | pass | `upgrade-rollback.md` and `recovery.md` distinguish safe artifact rollback from incompatible state/key changes. |

The first rehearsal exposed two navigation gaps: the root README did not link
the operator entry point, and the support-bundle redaction test did not cover
opt-in log files. Both were fixed; the second rehearsal and the automated docs
path/security gates pass on the same commit.

A real cluster/IdP/O3K mutation rehearsal is intentionally outside this record:
the stable multi-node O3K acceptance environment belongs to issue #106.
