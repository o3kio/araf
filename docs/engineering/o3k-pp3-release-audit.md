# O3K PP.3 dependency preparation: phase 0 audit

Audit date: 2026-09-19. Protected upstream `main`:
`871f2b5534cf4f6eb1cb2a04d74e0f0275ca7613` (fetched and checked against
GitHub's branch API). Working branch: `release/araf-o3k-pp3-prep`.
No O3K repository changes or installer integration are in scope.

## Inventory and authority

Read README, AGENTS, issues #42/#65/#67/#106 and their production prompts,
product MVP, architecture overview/integration/backend contracts, all ADRs,
threat model and quality gates; inspected production/security/packaging/O3K
evidence, operator installation/configuration/secrets/recovery/upgrade docs,
release gates and protected-main policy. Inspected both workflows, both
Dockerfiles, release Compose, Helm templates/values, both BFF entry points,
configuration validation, O3K/OpenStack adapters, OIDC/session/CSRF middleware,
health/version handlers, journals and contract/security/packaging tests.

Canonical runtime components are `tenant-console`, `operator-console`,
`tenant-bff`, `operator-bff`. The canonical three OCI repositories are
`ghcr.io/o3kio/araf-tenant-console`, `araf-operator-console`, and `araf-bff`.
Both BFF components use the same image with separate commands, processes,
stores, routes and OIDC clients. Four component identities do not require
four distinct image digests. Preserve this architecture.

GitHub Releases API/list reports no releases. Git tags are `v1.0.0-rc.11`
(`ca419280604f088ec1fd3752c695d7ff833d8e41`) and `v1.0.0-rc.12`
(`de64cc9193085116fa30ad51c04ccab24a013dd0`). GHCR also contains
`v1.0.0-rc.13`, built by run 35135014348 from
`41b9f260cb8ec9d94b6ada9e42e246d4dad78332`. All three packages are private.
Latest inspected OCI index identities:

| Image | v1.0.0-rc.13 digest (historical; not this task's candidate) |
| --- | --- |
| araf-bff | `sha256:b4cca0cb1ae4892236a35e1ad9084e3d57ef55e0f3dc1d6ede7c92ecadb5c381` |
| araf-tenant-console | `sha256:a021663031a871e6709aa484c7f3f45506eb58091e1b4b2408081bdf406afcb8` |
| araf-operator-console | `sha256:14ac8d50dced7c9f674d68bbe8ed626017ab3d328ad1f19f3271e8a446aab1a8` |

Root npm version is 0.0.0; Helm chart is 0.1.0/appVersion 0.0.0;
release workflow accepts arbitrary version input or ref name. No single
validated release authority exists. Reserve no tag yet; next unused candidate
is provisionally **v1.0.0-rc.14**, following existing prerelease history, with
no GA or production certification implied. Recheck registry before tagging.

BuildKit SBOM/max provenance and GitHub OIDC digest attestation already exist.
Actions and base images are pinned. Trivy gates fixable HIGH/CRITICAL findings.
No release manifest, published verification bundle or signed release assets
exist. No platform immutability is established; policy plus verified digests
and provenance is the honest claim. Historical successful builds do not prove
current public download or the proposed candidate's behavior.

## Gaps before implementation

| Severity | Gap | Required disposition |
| --- | --- | --- |
| BLOCKER | No public GitHub prerelease or anonymously pullable runtime packages | Publish and re-resolve public artifacts after gates; package visibility must permit anonymous access |
| BLOCKER | No single source/version/four-component digest manifest and compatibility schema | Validated release authority, immutable build identity, schema-tested manifest and bundled contracts |
| BLOCKER | Workflow can reuse candidate tags, accepts non-tag source/version input | Ref/version equality, no rerun/reuse, no overwrites; failed versions require successors |
| HIGH | `/version` trusts mutable runtime env; Helm supplies arbitrary version/SHA | Embed identity at build time and reject conflicting runtime metadata |
| HIGH | Compose BFF ports exposed on all interfaces contrary to docs | Bind loopback; preserve distinct surfaces; drop capabilities and no-new-privileges |
| HIGH | O3K HTTP client follows redirects by default | Disable redirects on the credential-bearing client; regression test |
| HIGH | No exact candidate clean Compose convergence/recreation/cleanup evidence | Source-free digest-only harness, durable-state checks, foreign-resource canaries |
| HIGH | Release publication does not verify SBOM subjects, provenance and labels after push | Download/verify digest graph and signed provenance before bundling |
| HIGH | No machine-readable O3K contract claim or known-incompatible profile guard | Publish actual native v1/P2 baseline without inventing O3K release compatibility |
| MEDIUM | Compose lacks resource bounds/restart policy; Helm digests merely nonempty | Explicit limits and restart policy; validate sha256 syntax |
| MEDIUM | Compose lacks distinct operator upstream and explicit TLS trust mapping | Per-surface upstream configuration; document system CA trust without TLS bypass |
| MEDIUM | Frontend has no independently queryable immutable version document | Embed static version metadata and verify against image labels |
| MEDIUM | CI packaging/Helm checks are incomplete/optional on PRs | Required manifest, Compose, Helm and release identity checks |
| MEDIUM | State/cleanup runtime contract has no certified minimum runtime | Document tested Docker/Compose only, ownership, retention and scoped cleanup |
| LOW | Historical README/evidence can be read as broad production readiness | Explicit RC versus implementation/testing/production/HA status |

No ADR conflict was identified. Araf remains a client and session boundary;
O3K/OpenStack remain the authorities. Existing production selection rejects
fixtures and missing/invalid config; test/development binaries may use
fixtures explicitly. Existing HTTPS/OIDC/PKCE, opaque Secure/HttpOnly cookies,
CSRF, encrypted durable state and cross-surface regression tests are retained.
Readiness reports configured adapter, not cloud readiness.

## Environment and release boundaries

Available: Docker Engine 29.8.0, Compose 5.5.1, linux/amd64 and Helm. Existing
containers and untracked `data/` are foreign state and must remain untouched.
The active GitHub account can inspect repo/packages and use workflows.
Protected main requires current frontend/Rust/browser checks and one independent
approving review, with stale approvals dismissed. No administrative bypass.

A compatible real O3K deployment/config reference has been requested. Historical
P2 evidence uses O3K `21fe687c387a04f107b6e87fac04060b1c28e449`; this does
not establish compatibility with any current PP release. #106 stays open for
multi-node HA/pre-production; #113 owns live cross-candidate rollback.
Controlled-backend packaging evidence must not be called real O3K acceptance.
