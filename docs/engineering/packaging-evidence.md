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
`110ab0ae86bf57bee3897b35ca62d68dc0f45c57` with bounded contexts and exact
labels:

- BFF: `sha256:3e4ebd068cf55720f9b89d55ee9371464357a55696148f2bed8bbbad11b578ff`
- Tenant console: `sha256:d7528b28809bb6bc817d35fed96b139636c54ab992971e02c774d76373c23fc6`
- Operator console: `sha256:b29b38fc3036f57ab44d733a946f52eabf008af8410f4da7bb6532c959d76789`

These are development-host artifacts, not the publishable release: trusted
CI keyless provenance and external install/upgrade/rollback evidence remain
required.

The images build with bounded contexts and include OCI labels. Local cosign
signatures and CycloneDX SBOM attestations verify against the development
public key; publication remains blocked until CI attaches the trusted keyless
provenance identity and external install/upgrade evidence is attached.

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
