# P4.5 packaging, deployment and upgrade evidence

Versioned release artifacts are defined by OCI labels in `backend/Dockerfile`
and `Dockerfile.frontend` (`org.opencontainers.image.version` and `revision`).
`deploy/docker-compose.release.yml` consumes digest-pinned images and external
environment/secret references; it never embeds provider or OIDC credentials.
`deploy/helm/araf` is the reference Kubernetes deployment with two replicas per
console surface, read-only roots, dropped Linux capabilities, persistent
session storage and `/healthz`/`/readyz` probes.

Both references pass backend endpoints and per-surface OIDC configuration
explicitly. Compose values are supplied from an external environment file; the
Helm chart reads O3K/OpenStack credentials and OIDC client secrets from an
operator-created Kubernetes Secret (`secrets.name`), including the base64
AES-256 `ARAF_SESSION_STORE_KEY` used to encrypt the shared durable session
file. No credential or encryption key is stored in the chart or image. Missing
values remain fail-closed at BFF startup.

The BFF exposes `/version` for operator support and release inventory. Runtime
configuration remains fail-closed: production requires an explicit adapter,
HTTPS public/trusted origins, upstream credentials/endpoints and
`ARAF_SESSION_STORE_PATH` and `ARAF_SESSION_STORE_KEY`.

Development-host OCI evidence for RC `0.1.0-rc` is available in the local
registry. The current images were rebuilt from Araf
`24a8b691a7c447ce001271519713d5b322757eb8` with bounded contexts and exact
labels:

- BFF: `sha256:c1aabc13a2edc9fc2bc1e86780a2d57eef5a6e1503f3cf82e38a2bdbe74a7f61`
- Tenant console: `sha256:5fbdf4c6bf33c58cfff5597fc7f29af0c2c4a5bb03c10f59b5f04d0fc42132cf`
- Operator console: `sha256:9475659bb370a15d355d2310a325e513f6ce6ac69ec6921afd7515315eec78ee`

These are development-host artifacts, not the publishable release: trusted
CI keyless provenance and external install/upgrade/rollback evidence remain
required.

The images build with bounded contexts and include OCI labels. Local cosign
signatures and CycloneDX SBOM attestations verify against the development
public key; publication remains blocked until CI attaches the trusted keyless
provenance identity and external install/upgrade evidence is attached.

Local clean-install smoke (2026-09-13) also exercised the BFF digest in a
non-root, read-only container behind HTTPS with a durable encrypted session
volume. A fresh O3K/Keycloak journey completed login, scope selection and a
real network create/detail/delete; the same volume survived a start on the
previous BFF digest and a return to the candidate digest. This validates the
packaging mechanics on one host only; it does not replace the external
environment, trusted provenance, multi-host rollout or pilot gates.

Compatibility matrix:

| Araf release | O3K | OpenStack |
| --- | --- | --- |
| 0.0.x RC | native O3K P2 API profile at the tested development commit recorded in `docs/engineering/o3k-production-evidence.md`; no stable O3K release is claimed and multi-node certification remains deferred to #106 | OpenStack 2026.1 Gazpacho (SLURP), Keystone/Nova/Glance/Neutron/Cinder profile certified by P3.9; Swift, floating IP and volume attachment are deferred |

The operator-facing release claim and limitation boundary is canonical in
[`docs/operator/limitations.md`](../operator/limitations.md). In particular,
`#106 remains OPEN — stable-release multi-node HA/O3K certification is intentionally deferred.`

The release artifact topology is three OCI images: one shared Rust BFF image
with separate `tenant-bff` and `operator-bff` commands, plus independently
built Tenant and Operator console images. The console images contain only
static assets and receive their BFF upstream at runtime; no customer endpoint,
OIDC value or credential is baked into an image. Helm deploys each surface as
an independent frontend/BFF Deployment and Service. Tenant and Operator
ingress rules must be configured separately.

Container contract: BFFs listen on 8080 (Tenant) or 8081 (Operator), expose
`/healthz`, `/readyz` and `/version`, and write only to the mounted durable
session/journal path plus temporary storage. Console images listen on 8080 and
require `BFF_UPSTREAM`; an unset upstream makes nginx reject its configuration.
Because the nginx entrypoint renders that value at startup, both release
references mount `/etc/nginx/conf.d` as an ephemeral uid-101 writable volume
while keeping the remaining image root read-only. `/var/cache/nginx`,
`/var/run` and `/tmp` are likewise ephemeral. The release Compose file supplies
all four digest-pinned images and external runtime configuration, while Helm
requires both BFF and per-surface frontend digests and preserves the same
read-only-root contract.

An upgrade/rollback test must deploy release N, create a session and pending
compatibility operation, replace images with N+1, verify `/version`, session
and journal recovery, then roll back to N and repeat health checks. The
release compose and Helm manifests permit this image-only transition without
rebuilding source; the final candidate records exact image digests and the
results in the release evidence. Run `tests/package-upgrade-rollback.sh` before
that deployment. It is a fail-closed contract gate: it rejects source mounts
and build directives, requires immutable image references, checks external
session-key wiring, and lints the chart when Helm is installed. It does not
pretend to be the deployment test. The release owner must record both exact
image digests, `/version` values, health/readiness, session continuity and
journal continuity for N → N+1 → N in a disposable environment. A rollback is
valid only when the same durable state is reused; rebuilding source or
regenerating keys is not a rollback.

Reproducibility policy: CI pins Node, pnpm and Rust toolchains and uses both
lockfiles, while BuildKit records the source revision, SBOM and provenance on
each published digest. The release workflow intentionally does not claim
byte-identical images: base-image refreshes and toolchain timestamps can alter
bytes. The digest, source SHA, lockfile hashes and workflow run are the
authoritative identity tuple for an artifact.
