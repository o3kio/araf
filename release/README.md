# Release Candidate contract

Prerelease / Release Candidate. Not production-ready certification.

`backend/RELEASE_VERSION` is the release version authority (including `v`).
Cargo/npm package versions are internal package versions, not release identity.
The version must equal the immutable Git tag, OCI version labels, compiled
BFF version, frontend `version.json`, and release manifest version. Helm's
packaged chart version/appVersion are derived from it, without the `v`.
The full tag source SHA is embedded in all images. Runtime overrides that
conflict with BFF build identity fail startup. Local non-release builds report
`0.0.0-dev`/`unknown` unless explicitly supplied build metadata.

There are four components and three image repositories. Both BFFs use the
same `araf-bff` digest and different commands. They remain separate processes,
stores, routing and authentication surfaces. No additional packaging system
is introduced. Compose is the primary single-host release path; Helm remains
maintained but is not certified by Compose evidence.

A release is **release-policy immutable + digest/provenance bound**. Platform
immutability is not claimed. Never move a tag, rerun its image publication,
replace an image or overwrite an asset. A partial failed publication consumes
the version. Record the failure and publish a successor after fixing source.
Only final reviewed source on protected main can be tagged. The release
workflow rejects non-tag dispatch, repeat attempts, version mismatch, existing
version images and existing GitHub Releases. The GitHub prerelease flag is
derived from the version policy (`^v[0-9]+\.[0-9]+\.[0-9]+-(rc|alpha|beta)\.[0-9]+$`,
case-insensitive on the suffix word) by the release publication workflow, so
policy-matching versions publish as GitHub prereleases and stable versions do
not. Keep all GHCR packages public;
anonymous verification is a release requirement, not optional documentation.

Published BuildKit SPDX SBOM and SLSA provenance are attached to the OCI index.
GitHub's OIDC attestation binds each index to this repository, release workflow,
source SHA and tag. The release manifest lists four components, exact digests,
platform, per-image evidence and hashed deployment/compatibility assets.
It is a client compatibility record, not authority over O3K readiness/state.

## Compatibility

The embedded contract is
`backend/console-bff-core/contracts/release-contract.json`, served inside BFF
`/version` and distributed in the release bundle. It identifies native
`o3k.io/v1` (`/o3k/v1`) and the historical P2 test baseline. No stable O3K
release range is asserted. `ARAF_O3K_API_CONTRACT` declares the deployment's
upstream contract and mismatches fail startup. This is a configuration guard,
not an invented O3K version-negotiation endpoint; actual resource schemas and
versioned diagnostic responses remain validated by the native adapter.
Candidate-specific real backend testing belongs in release evidence; the
embedded historical baseline must not be represented as a new live test.

Existing OpenStack support is bounded to 2026.1 core Keystone v3/Nova v2.1/
Glance v2/Neutron v2/Cinder v3. Swift, floating IP and volume attachments remain
excluded. #106 remains the O3K multi-node HA/pre-production gate; #113 owns
live cross-candidate rollback. Araf owns no cloud resource truth.

## Primary runtime and configuration

Initial validation target: Linux amd64, Docker Engine 29.8.0 and Compose 5.5.1.
These are the minimum versions for this tuple until older versions are tested.
Required features: digest pulls, read-only rootfs, tmpfs ownership, health-based
dependencies, CPU/memory/PID limits, bind `create_host_path: false`, named
volumes and `up --wait --no-build`. No Podman or HA support is claimed.

The bundle contains the existing release Compose file with fixed image
identities. Deployment uses `docker compose -p <owned-project> --env-file
<external-config> -f compose.yml up -d --wait --no-build`. Do not run a build on
the target. No build context, source mount or image build exists in the bundle.
Use the same explicit project and configuration on every run.

Provide distinct HTTPS Tenant/Operator origins, OIDC confidential clients,
redirect URIs and session keys, and `ARAF_TENANT_O3K_URL` /
`ARAF_OPERATOR_O3K_URL` (the URLs may match if the upstream serves both).
Set the Helm equivalent `backend.o3kApiContract` or the Compose
`ARAF_O3K_API_CONTRACT` to the contract named by the manifest. No O3K release
version or endpoint is baked into images. Native production
requests use per-session server-held O3K credentials. Supply the contract
identifier explicitly. OpenStack credentials remain external per the operator
secret guide; no profile is broadened by release packaging.

`ARAF_CA_BUNDLE` is an absolute readable PEM CA bundle path mounted read-only
into both BFFs. Use the system bundle for public CAs or a deployment-managed
bundle including the required private CA. `SSL_CERT_FILE` selects that trust
for native TLS (OIDC, O3K and OpenStack); unreadable/invalid PEM fails startup.
No TLS verification bypass exists. External TLS ingress and its private key
remain deployment-owned. All host ports bind loopback by default.

Release binaries only accept production mode. Missing/invalid config fails
closed. Fixture selection is forbidden even when someone changes the runtime
profile; development fixtures require a separate non-release build.

## State, health and cleanup

Config: external reviewed configuration and paths (mode 0600 if it transports
secrets). Secrets: deployment secret manager; never tracked or bundled. State:
separate named Tenant/Operator session volumes, seeded UID/GID 65532 by the
image, holding encrypted sessions/revocations/OIDC state and, for OpenStack,
the derived CompatibilityOperation journal. Preserve keys with backups. No
normal root repair is needed. Cache: nginx configuration/cache/run/tmp tmpfs,
UID 101 where needed, discarded on recreation. Logs: stdout/stderr owned and
rotated by the runtime; never a cloud-state authority.

Frontend `/` proves static serving; `/version.json` reports build identity.
BFF `/healthz` means process alive; `/readyz` means startup configuration and
adapter constructed, **not backend health**. Backend availability is established
by authenticated authoritative requests. A cloud outage must not create
synthetic resources or a false O3K-ready verdict.

Same-version `up` preserves project containers/networks/volumes and keys.
Restart/recreation/stop-start retain named-volume state; tmpfs and process
metrics are ephemeral. `unless-stopped` supports daemon restart, but host reboot
must be recorded as tested or not tested, never inferred from restart policy.

Normal uninstall: `docker compose -p <owned-project> --env-file <external-config>
-f compose.yml down` (retains state). Explicit state deletion uses the same
command with `--volumes`; this deletes only this project's named volumes.
Never prune global Docker resources. Retain configuration, CA and secrets
unless their owner explicitly requests deletion. Check project ownership
before cleanup. Foreign canaries must survive the cleanup test.

Upgrade only by a separately verified manifest with compatible declared
backend/auth/session/deployment/state contracts. Unknown compatibility requires
review and fails automatic admission. No automated rollback promise is made.
