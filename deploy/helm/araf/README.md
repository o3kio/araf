# Araf reference Helm deployment

This chart deploys the Tenant and Operator BFFs as separate Deployments and
Services. They use separate commands, OIDC client settings, and session PVCs;
the Operator service is never routed through the Tenant service. Enable
`ingress` only with an ingress controller configured for TLS and provide two
explicit host/path rules (each path must set `surface` to `tenant` or
`operator`).

The image is digest-pinned: set `image.repository` and the immutable
`image.digest` produced by the release workflow. The chart intentionally has
no production defaults for adapter, endpoints, OIDC, or secrets. Supply these
through a reviewed values file and a Kubernetes Secret named by
`secrets.name`; do not put secret values in values or ConfigMaps.

Each BFF listens on its documented container port (Tenant 8080, Operator
8081), exposes `/healthz` for liveness/startup and `/readyz` for readiness,
and mounts its encrypted durable session authority at `/var/lib/araf`. The
PVC must provide a supported shared RWX authority when replicas exceed one;
local-only storage is not a multi-replica deployment.

For a container-only deployment, use `deploy/docker-compose.release.yml` with
pre-built digest-pinned images. It is a reference contract, not a substitute
for external TLS, secret management, or shared durable storage.
