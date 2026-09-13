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
| 0.0.x RC | native O3K P2 contract at the pinned deployment SHA | Keystone/Nova/Glance/Neutron/Cinder profile certified by P3.9 |

An upgrade/rollback test must deploy release N, create a session and pending
compatibility operation, replace images with N+1, verify `/version`, session
and journal recovery, then roll back to N and repeat health checks. The
release compose and Helm manifests permit this image-only transition without
rebuilding source; the final candidate records exact image digests and the
results in P4.7 evidence.
