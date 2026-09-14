# Installation from release artifacts

Normal installation consumes the release OCI images and the versioned Helm
chart or Compose reference. Do not build from source for an operational
install.

## Prerequisites

- Kubernetes with an ingress controller and a TLS certificate, or Docker
  Compose plus an equivalent TLS reverse proxy.
- A secret manager (Kubernetes Secret, external-secrets controller, or an
  equivalent) and durable storage for each console surface.
- DNS names and HTTPS origins for the Tenant and Operator consoles.
- A supported external OIDC provider with one confidential client per surface.
- Either a reachable O3K HTTPS gateway or a certified OpenStack Keystone v3
  deployment. Production fixture mode is rejected at startup.
- For more than one replica, a shared, encrypted, lock-safe RWX/session
  authority. A local filesystem is not multi-replica HA evidence.

## Verify and stage artifacts

Record the release version, source SHA, image digests, SBOM and CI provenance
attestation. Pull by digest, not by a mutable tag:

```bash
docker pull ghcr.io/o3kio/araf-bff@sha256:<bff-digest>
docker pull ghcr.io/o3kio/araf-tenant-console@sha256:<tenant-digest>
docker pull ghcr.io/o3kio/araf-operator-console@sha256:<operator-digest>
```

The placeholders are illustrative and intentionally not runnable until
replaced with the release values. Verify the same digests in the release
attestation before rollout.

## Helm path

1. Create the reviewed values file from `deploy/helm/araf/values.yaml`.
   Set both frontend digests, the BFF digest, `version`, `gitSha`, backend
   endpoint, HTTPS origins, OIDC issuer/client IDs/redirect URIs, and the
   ingress TLS hosts. Keep secret values out of values and ConfigMaps.
2. Create the referenced Kubernetes Secret with the keys listed in
   `values.yaml` ([secret guide](secrets.md)).
3. Confirm the chart renders and fails on missing digests:

   ```bash
   helm lint deploy/helm/araf \
     --set image.digest=sha256:<bff-digest> \
     --set frontendImage.tenant.digest=sha256:<tenant-digest> \
     --set frontendImage.operator.digest=sha256:<operator-digest>
   helm upgrade --install araf deploy/helm/araf \
     --namespace araf --create-namespace --values values.production.yaml
   ```

   The first command is a template check; replace placeholders before use.
4. Wait for both BFF readiness probes and frontend probes. Keep Tenant and
   Operator ingress hosts separate; never route operator paths to the Tenant
   BFF.

## Compose path

Copy `deploy/docker-compose.release.yml` and provide an external `.env` or
secret-injection mechanism containing all required non-secret metadata and
secret references. Then run:

```bash
docker compose --env-file .env -f deploy/docker-compose.release.yml up -d
docker compose --env-file .env -f deploy/docker-compose.release.yml ps
```

Compose is a reference container contract. It does not provide TLS,
secret-manager protection, a shared multi-host filesystem or a load balancer;
add those at the deployment boundary.

## First verification

From the private BFF network and both browser-facing HTTPS origins, collect:

```bash
curl --fail https://tenant.example/healthz
curl --fail https://tenant.example/readyz
curl --fail https://tenant.example/version
curl --fail https://operator.example/healthz
curl --fail https://operator.example/readyz
curl --fail https://operator.example/version
```

`healthz` is process liveness. `readyz` means validated configuration and the
selected adapter were constructed; it does not prove provider capacity or a
successful mutation. Complete one login, scope selection, service discovery,
bounded list, and read-only operation smoke before enabling users.
