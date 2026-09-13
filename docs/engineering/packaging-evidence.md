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

Development-host OCI evidence for RC `0.1.0-rc` (revision
`309b827c6108a08fd24601e903b3071956526771`) is available in the local registry:

- BFF: `sha256:0ecef2122b00a5e1a5cd03831a755a55b70651ac841db190c8803d3d04379336`
- Tenant console: `sha256:ab8bc01ad0cf0266dfc463cfc72c4795059ea82988f578409791d9fc0d63435`
- Operator console: `sha256:11538ef1fe6c7f3a8f6eb05582d4a9cef1f5087cfbe839f3a103c9aa2d182c2c`

The images build with bounded contexts and include OCI labels, but publication
remains blocked pending signed provenance and remediation/acceptance of the
Trivy HIGH findings recorded in the security evidence.

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
