# Installation from release artifacts

Normal installation consumes the release OCI images and the versioned Helm
chart or Compose reference. Do not build from source for an operational
install.

## Prerequisites

- Kubernetes with an ingress controller and a TLS certificate, or Docker
  Compose plus an equivalent TLS reverse proxy.
- The certified Compose preparation tuple is Docker Engine `29.8.0` with
  Docker Compose `v5.5.1`. Older Docker versions and Podman are unsupported
  until separately tested; this release path does not claim runtime breadth.
- A secret manager (Kubernetes Secret, external-secrets controller, or an
  equivalent) and durable storage for each console surface.
- DNS names and HTTPS origins for the Tenant and Operator consoles.
- A supported external OIDC provider with one confidential client per surface.
- Either a reachable O3K HTTPS gateway or a certified OpenStack Keystone v3
  deployment. Production fixture mode is rejected at startup.
- For more than one replica, a shared, encrypted, lock-safe RWX/session
  authority. A local filesystem is not multi-replica HA evidence.

The BFF image seeds `/var/lib/araf` for its non-root UID (65532), and the Helm
chart sets the pod `fsGroup` to the same UID. This is required for a fresh
session PVC or Compose named volume to initialize without a privileged manual
`chown`. If an existing volume was provisioned with incompatible ownership,
stop the rollout and correct the storage provisioner or restore a reviewed
volume snapshot; do not make the BFF run as root.

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

## PP demo release bundle

Versioned releases publish a signed-off bundle as GitHub Release assets:

- `araf-<version>-deploy.tar.gz` — `docker-compose.release.yml` (unmodified),
  `ENVIRONMENT.md` (complete environment schema), `digests.txt` and
  `VERIFY.md`.
- `araf-<version>-digests.txt` — the immutable image digest pins.
- `araf-<version>-release-manifest.json` — schema-validated version, source
  SHA, four runtime component identities, backend compatibility and security
  contract. The Tenant and Operator BFF entries intentionally share the
  canonical `araf-bff` image digest and select separate binaries at runtime.
- `<component>-<version>.spdx.json` + `araf-<version>-sbom.sha256` — SPDX
  SBOMs extracted from the GHCR image attestation manifests.
- `<component>-<version>.provenance.json` + `araf-<version>-provenance.sha256`
  — SLSA v1 provenance extracted from the same attestations.
- `araf-<component>-<version>.oci.tar` + `araf-<version>-oci-tarballs.sha256`
  — docker-save OCI tarballs. The `ghcr.io/o3kio` packages currently require
  authentication to pull, so hosts without GHCR credentials install from
  these tarballs instead of the registry.

Releases are created by `.github/workflows/release-publish.yml` only for an
existing tag whose images were built and attested by `release-images`.
Publish fails closed: a missing tag, a missing image, a missing
SBOM/provenance attestation, or an OCI tarball whose config blob does not
match the registry platform manifest stops the release instead of shipping a
partial bundle.

Verify before staging:

```bash
tar -xzf araf-v1.0.0-rc.12-deploy.tar.gz
cat digests.txt   # <component> <image>:<version>@sha256:<index> (+ -platform / -config lines)
sha256sum -c araf-v1.0.0-rc.12-sbom.sha256
sha256sum -c araf-v1.0.0-rc.12-provenance.sha256
sha256sum -c araf-v1.0.0-rc.12-oci-tarballs.sha256
# Re-resolve each image and compare against digests.txt:
docker buildx imagetools inspect ghcr.io/o3kio/araf-bff:v1.0.0-rc.12 --format '{{.Manifest.Digest}}'
```

### Hosts without GHCR credentials

Use the `araf-<component>-<version>.oci.tar` assets. Verify the tarball
sha256, verify the config blob named in the tarball's `manifest.json`
against the `<component>-config` line of `digests.txt`, `docker load`, then
tag the loaded image ID as `ghcr.io/o3kio/<component>:<version>` so the
digest-pinned Compose references resolve locally (the loaded image is
restored under its index digest, so the release `.env` digest pins keep
working). The full step-by-step sequence with the v1.0.0-rc.12 values is
documented in
[releases/v1.0.0-rc.12.md](../releases/v1.0.0-rc.12.md); config-digest
verification before tagging is mandatory.

GHCR tags are mutable; the `@sha256:` digest is the only authority for what
runs (cryptographically/policy immutable, not platform-enforced). Pull and
pin by digest as described above — never by tag. See
[releases/v1.0.0-rc.12.md](../releases/v1.0.0-rc.12.md) for the v1.0.0-rc.12
notes and known limitations.

## Helm path

1. Create the reviewed values file from `deploy/helm/araf/values.yaml`.
   Set both frontend digests, the BFF digest, backend
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
