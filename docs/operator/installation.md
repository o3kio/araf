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

The release Compose and Helm references keep console image roots read-only.
They provide only ephemeral writable mounts for nginx-generated configuration
(`/etc/nginx/conf.d`) and nginx runtime paths. Preserve those mounts when
translating the manifests to another orchestrator; without them the console
entrypoint fails closed before readiness.

## First verification

The browser-facing ingress routes to the static console. Its `/api/` location
proxies API calls, but `/healthz`, `/readyz`, `/version` and `/metrics` are not
browser-origin probe paths. Query the private BFF Services directly instead.

For Helm, replace `<release>` with the actual Helm release name (the angle
brackets are a placeholder, not shell syntax) and keep each port-forward
running in its own terminal:

```bash
kubectl -n araf port-forward svc/<release>-tenant-bff 18080:80
kubectl -n araf port-forward svc/<release>-operator-bff 18081:80
curl --fail http://127.0.0.1:18080/healthz
curl --fail http://127.0.0.1:18080/readyz
curl --fail http://127.0.0.1:18080/version
curl --fail http://127.0.0.1:18081/healthz
curl --fail http://127.0.0.1:18081/readyz
curl --fail http://127.0.0.1:18081/version
```

For Compose, the release reference publishes the BFF listeners on localhost:

```bash
curl --fail http://127.0.0.1:8080/healthz
curl --fail http://127.0.0.1:8080/readyz
curl --fail http://127.0.0.1:8081/healthz
curl --fail http://127.0.0.1:8081/readyz
```

`healthz` is process liveness. `readyz` means validated configuration and the
selected adapter were constructed; it does not prove provider capacity or a
successful mutation. Complete one login, scope selection, service discovery,
bounded list, and read-only operation smoke before enabling users.
