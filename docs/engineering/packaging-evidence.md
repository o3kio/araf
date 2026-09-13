# P4.5 packaging, deployment and upgrade evidence

Versioned release artifacts are defined by OCI labels in `backend/Dockerfile`
and `Dockerfile.frontend` (`org.opencontainers.image.version` and `revision`).
`deploy/docker-compose.release.yml` consumes digest-pinned images and external
environment/secret references; it never embeds provider or OIDC credentials.
`deploy/helm/araf` is the reference Kubernetes deployment with two replicas per
console surface, read-only roots, dropped Linux capabilities, persistent
session storage and `/healthz`/`/readyz` probes.

The BFF exposes `/version` for operator support and release inventory. Runtime
configuration remains fail-closed: production requires an explicit adapter,
HTTPS public/trusted origins, upstream credentials/endpoints and
`ARAF_SESSION_STORE_PATH`.

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
