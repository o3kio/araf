#!/usr/bin/env bash
# Assemble the versioned Araf release asset bundle for a published OCI image
# set. Resolves each release image to its immutable index digest, extracts the
# SPDX SBOM and SLSA provenance attestation layers from GHCR, saves each image
# as a docker-save OCI tarball for unauthenticated demo installs (the ghcr.io
# packages require authentication to pull), and writes the digests, checksum
# and deploy-bundle files under dist/release/.
#
# Inputs (environment):
#   RELEASE_VERSION  release tag, e.g. v1.0.0-rc.12 (required)
#   GITHUB_TOKEN     token with package:read for GHCR attestation fetch
#                    (GH_TOKEN is accepted as a fallback)
#   GHCR_USER        user for the GHCR token exchange (default: GITHUB_ACTOR)
#   GHCR_REGISTRY    registry host (default: ghcr.io)
#   IMAGE_NAMESPACE  image namespace (default: o3kio)
#   OUTPUT_DIR       asset output directory (default: <repo>/dist/release)
#
# The docker CLI must be logged in to GHCR (docker login) so `docker pull`
# can fetch the images for the OCI tarballs. The script only reads from the
# registry and never mutates it, never creates or modifies a GitHub Release,
# and never prints tokens.
set -Eeuo pipefail

usage() {
  cat <<'EOF'
Usage: RELEASE_VERSION=vX.Y.Z[-suffix] [GITHUB_TOKEN=... GHCR_USER=...] publish-release.sh

Resolves the release image set to index digests, extracts SBOM and SLSA
provenance attestation layers from GHCR, saves docker-save OCI tarballs for
unauthenticated installs, and writes under dist/release/:
  sbom/<component>-<version>.spdx.json
  provenance/<component>-<version>.provenance.json
  araf-<component>-<version>.oci.tar
  araf-<version>-digests.txt
  araf-<version>-sbom.sha256
  araf-<version>-provenance.sha256
  araf-<version>-oci-tarballs.sha256
  araf-<version>-deploy.tar.gz   (docker-compose.release.yml, ENVIRONMENT.md,
                                  digests.txt, VERIFY.md)

digests.txt pins three lines per component: the index digest
(`<component> <image>:<version>@sha256:<index>`), the linux/amd64 platform
manifest digest (`<component>-platform <image>:<version>@sha256:<platform>`)
and the image config digest (`<component>-config sha256:<config>`).

Fails closed when the version is unset/malformed, any image or attestation
layer is missing, a saved tarball does not match the registry config digest,
or any checksum does not verify. Never prints tokens.
EOF
}

fail() { echo "publish-release: ERROR: $*" >&2; exit 1; }

require_tools() {
  local tool
  for tool in docker curl jq tar sha256sum; do
    command -v "$tool" >/dev/null 2>&1 || fail "required tool not found: $tool"
  done
}

# Resolve <image-ref> to its registry index digest. Fail closed: a missing or
# unpullable image is a release-blocking error, never a warning.
resolve_digest() {
  local image_ref=$1
  local digest
  if ! digest=$(docker buildx imagetools inspect "$image_ref" \
      --format '{{.Manifest.Digest}}' 2>/dev/null); then
    fail "cannot resolve image ref (fail-closed): $image_ref"
  fi
  [[ "$digest" =~ ^sha256:[0-9a-f]{64}$ ]] \
    || fail "unexpected digest format for $image_ref: '$digest'"
  printf '%s' "$digest"
}

# Exchange the caller's GitHub token for a scoped GHCR pull token. The token
# is returned on stdout; callers must never print it.
ghcr_pull_token() {
  local repo=$1
  local token_response token
  token_response=$(curl -fsSL -u "$GHCR_USER:$GH_TOKEN_RESOLVED" \
    "https://${GHCR_REGISTRY}/token?service=${GHCR_REGISTRY}&scope=repository:${repo}:pull") \
    || fail "GHCR token exchange failed for ${repo} (check GITHUB_TOKEN/GHCR_USER)"
  token=$(printf '%s' "$token_response" | jq -r '.token // .access_token // empty')
  [[ -n "$token" && "$token" != "null" ]] || fail "GHCR token exchange returned no token for ${repo}"
  printf '%s' "$token"
}

# Fetch an OCI manifest. $3 selects the media type(s) to accept.
fetch_manifest() {
  local repo=$1 ref=$2 accept=$3 auth=$4
  curl -fsSL -H "Authorization: Bearer $auth" -H "Accept: $accept" \
    "https://${GHCR_REGISTRY}/v2/${repo}/manifests/${ref}" \
    || fail "cannot fetch manifest ${repo}@${ref}"
}

# Download a blob (attestation layer) to $3. The Accept header covering the
# layer media type avoids registry-side content negotiation surprises.
fetch_blob() {
  local repo=$1 digest=$2 out=$3 auth=$4
  curl -fsSL -H "Authorization: Bearer $auth" \
    -H "Accept: application/vnd.in-toto+json, application/octet-stream" \
    -o "$out" "https://${GHCR_REGISTRY}/v2/${repo}/blobs/${digest}" \
    || fail "cannot fetch blob ${digest} from ${repo}"
}

# Print the digest of the attestation manifest inside image index json $1.
attestation_manifest_digest() {
  local index_json=$1
  jq -r '.manifests[] | select(
      (.annotations."vnd.docker.reference.type" // "") == "attestation-manifest"
    ) | .digest' <<<"$index_json" | head -n1
}

# Print the blob digest of the layer annotated with predicate type $2 inside
# attestation manifest json $1.
attestation_layer_digest() {
  local att_json=$1 predicate=$2
  jq -r --arg predicate "$predicate" '.layers[] | select(
      (.annotations."in-toto.io/predicate-type" // "") == $predicate
    ) | .digest' <<<"$att_json" | head -n1
}

# Print the digest of the linux/amd64 platform manifest inside image index
# json $1. Empty when the index has no such platform.
platform_manifest_digest() {
  local index_json=$1
  jq -r '.manifests[] | select(
      .platform.os == "linux" and .platform.architecture == "amd64"
    ) | .digest' <<<"$index_json" | head -n1
}

# Print the config blob digest (hex, no algorithm prefix) recorded in the
# manifest.json of a docker-save tarball at $1. docker save writes the Config
# either as a legacy "<hex>.json" filename or, on containerd-backed engines,
# as an OCI-layout "blobs/sha256/<hex>" reference; both name the config blob.
saved_tar_config_hex() {
  local tar_file=$1 saved_config
  if ! saved_config=$(tar -xOf "$tar_file" manifest.json | jq -r '.[0].Config' 2>/dev/null); then
    fail "cannot read manifest.json from saved OCI tar: $tar_file"
  fi
  saved_config=${saved_config##*/}
  if [[ "$saved_config" =~ ^([0-9a-f]{64})\.json$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
  elif [[ "$saved_config" =~ ^([0-9a-f]{64})$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
  else
    fail "cannot parse config blob reference '$saved_config' in saved OCI tar: $tar_file"
  fi
}

# Save <image-ref>@<digest> as a docker-save OCI tarball at $3 and verify the
# config blob recorded in the tarball matches the registry config digest $2.
# The ghcr.io packages require authentication to pull, so the tarball is the
# unauthenticated install path; a config mismatch means the local image is
# not the attested one and the release must fail closed.
save_and_verify_oci_tar() {
  local image_ref=$1 config_digest=$2 tar_file=$3
  [[ "$config_digest" =~ ^sha256:[0-9a-f]{64}$ ]] \
    || fail "unexpected config digest format for $image_ref: '$config_digest'"
  docker pull "$image_ref" >/dev/null \
    || fail "docker pull failed for $image_ref (is the GHCR login valid?)"
  docker save -o "$tar_file" "$image_ref" \
    || fail "docker save failed for $image_ref"
  local saved_hex
  saved_hex=$(saved_tar_config_hex "$tar_file")
  [[ "$saved_hex" == "${config_digest#sha256:}" ]] \
    || fail "saved tar config mismatch for $image_ref: tar has '$saved_hex', registry config is '${config_digest#sha256:}'"
}

# Validate a digests.txt file. Every line must be one of:
#   <component> <registry>/<namespace>/<name>:<version>@sha256:<64 hex>
#   <component>-platform <registry>/<namespace>/<name>:<version>@sha256:<64 hex>
#   <component>-config sha256:<64 hex>
# Returns non-zero (and prints the offending line) when malformed.
validate_digests_file() {
  local file=$1 line component ref
  [[ -s "$file" ]] || { echo "digests file missing or empty: $file" >&2; return 1; }
  while IFS= read -r line; do
    [[ "$line" =~ ^([a-z0-9-]+)\ (.+)$ ]] || { echo "bad digests line: $line" >&2; return 1; }
    component=${BASH_REMATCH[1]}
    ref=${BASH_REMATCH[2]}
    [[ "$component" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]] || { echo "bad component in line: $line" >&2; return 1; }
    if [[ "$component" == *-config ]]; then
      [[ "$ref" =~ ^sha256:[0-9a-f]{64}$ ]] \
        || { echo "bad config digest in line: $line" >&2; return 1; }
    else
      [[ "$ref" =~ ^[a-z0-9./-]+:v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?@sha256:[0-9a-f]{64}$ ]] \
        || { echo "bad image ref in line: $line" >&2; return 1; }
    fi
  done < "$file"
  return 0
}

write_environment_md() {
  local out=$1 version=$2
  cat >"$out" <<EOF
# Araf ${version} — environment schema

Distilled from \`deploy/docker-compose.release.yml\`, \`docs/operator/secrets.md\`
and \`docs/operator/installation.md\`. The release Compose file pins every
image by digest (\`ARAF_*_IMAGE\` + \`ARAF_*_DIGEST\`) and never carries OIDC or
provider credentials.

## Services

| Service | Image | Notes |
| --- | --- | --- |
| tenant-bff | \${ARAF_BFF_IMAGE}@\${ARAF_BFF_DIGEST} | \`command: tenant-bff\`, port 8080, durable session volume |
| operator-bff | \${ARAF_BFF_IMAGE}@\${ARAF_BFF_DIGEST} | \`command: operator-bff\`, port 8081, separate session volume |
| tenant-console | \${ARAF_TENANT_CONSOLE_IMAGE}@\${ARAF_TENANT_CONSOLE_DIGEST} | nginx, proxies \`/api/\` to tenant-bff |
| operator-console | \${ARAF_OPERATOR_CONSOLE_IMAGE}@\${ARAF_OPERATOR_CONSOLE_DIGEST} | nginx, proxies \`/api/\` to operator-bff |

## Required variables (Compose fails at startup when unset)

| Variable | Applies | Notes |
| --- | --- | --- |
| \`ARAF_BFF_IMAGE\`, \`ARAF_BFF_DIGEST\` | both BFFs | digest-pinned BFF image; digest is the only immutable authority |
| \`ARAF_TENANT_CONSOLE_IMAGE\`, \`ARAF_TENANT_CONSOLE_DIGEST\` | tenant-console | digest-pinned console image |
| \`ARAF_OPERATOR_CONSOLE_IMAGE\`, \`ARAF_OPERATOR_CONSOLE_DIGEST\` | operator-console | digest-pinned console image |
| \`ARAF_UPSTREAM_ADAPTER\` | both BFFs | \`o3k\` or \`openstack\`; no implicit fallback |
| \`ARAF_TENANT_PUBLIC_URL\` | tenant-bff | HTTPS origin; sets \`ARAF_PUBLIC_URL\` and \`ARAF_TRUSTED_ORIGINS\` |
| \`ARAF_OPERATOR_PUBLIC_URL\` | operator-bff | HTTPS origin; sets \`ARAF_PUBLIC_URL\` and \`ARAF_TRUSTED_ORIGINS\` |
| \`ARAF_SESSION_STORE_KEY\` | matching BFF | base64-encoded exactly 32-byte AES key; secret-manager only |
| \`ARAF_TENANT_OIDC_CLIENT_ID\` / \`_CLIENT_SECRET\` / \`_ISSUER_URL\` / \`_REDIRECT_URI\` | tenant-bff | confidential OIDC client; secret never leaves the secret manager |
| \`ARAF_OPERATOR_OIDC_CLIENT_ID\` / \`_CLIENT_SECRET\` / \`_ISSUER_URL\` / \`_REDIRECT_URI\` | operator-bff | separate client/origin/cookie namespace from Tenant |

## Adapter-selected variables (set exactly one auth mode)

| Variable | Required when | Notes |
| --- | --- | --- |
| \`O3K_URL\` | adapter=\`o3k\` | HTTPS host-qualified O3K gateway URL |
| \`O3K_TOKEN\` | optional | server-side bootstrap token; never browser-visible |
| \`OPENSTACK_AUTH_URL\` | adapter=\`openstack\` | HTTPS Keystone v3 URL |
| \`OPENSTACK_TOKEN\` | one OpenStack mode | server-side Keystone token |
| \`OPENSTACK_USERNAME\` + \`OPENSTACK_PASSWORD\` | one OpenStack mode | service identity pair |
| \`OPENSTACK_USER_DOMAIN_NAME\` / \`OPENSTACK_PROJECT_NAME\` / \`OPENSTACK_PROJECT_ID\` / \`OPENSTACK_REGION\` | optional | Keystone scope hints |
| \`OPENSTACK_COMPUTE_URL\` / \`OPENSTACK_IMAGE_URL\` / \`OPENSTACK_NETWORK_URL\` / \`OPENSTACK_VOLUME_URL\` / \`OPENSTACK_OBJECT_STORAGE_URL\` | optional | pin only when catalog discovery is unsuitable |

## Fixed or optional operational variables

- \`ARAF_RUNTIME_PROFILE=production\` is fixed by the release Compose file.
  \`development\`/\`test\` are fixture-only profiles; production fixture mode is
  rejected at startup, and fixture adapters must never be enabled implicitly.
- \`ARAF_OPENSTACK_COMPATIBILITY_JOURNAL\` defaults to
  \`/var/lib/araf/compatibility-operations.json\` on the durable session volume.
- \`ARAF_VERSION\` / \`ARAF_GIT_SHA\` are release metadata baked into the image.
- \`RUST_LOG\` controls BFF logging; structured logs never include secrets.

## Secret custody

\`ARAF_SESSION_STORE_KEY\`, OIDC client secrets, \`O3K_TOKEN\`,
\`OPENSTACK_TOKEN\` and \`OPENSTACK_PASSWORD\` are secret-manager values. Keep
them in Kubernetes Secrets / external-secrets / Compose env injection only —
never in Git, values files, ConfigMaps, images, logs, issues or support
archives. Rotate per \`docs/operator/secrets.md\`.
EOF
}

write_verify_md() {
  local out=$1 version=$2 digests_file=$3
  cat >"$out" <<EOF
# Verifying the Araf ${version} release bundle

## What is in this bundle

- \`docker-compose.release.yml\` — the reference deployment, copied unmodified.
- \`ENVIRONMENT.md\` — the complete environment variable schema.
- \`digests.txt\` — the immutable image digest pins for this release.
- \`release-manifest.json\` — source-bound component and compatibility metadata.
- \`manifest.schema.json\` — schema for the machine-readable release manifest.
- \`VERIFY.md\` — this file.

The matching SBOM (\`*.spdx.json\`) and SLSA provenance
(\`*.provenance.json\`) files, plus their \`*.sha256\` checksum manifests, are
published alongside this tarball as assets on the GitHub Release.

## Verify image digests

GHCR tags are mutable; the \`@sha256:...\` digest is the only authority for
what runs. Re-resolve each image and compare against \`digests.txt\`:

\`\`\`bash
docker buildx imagetools inspect ghcr.io/o3kio/araf-bff:${version} --format '{{.Manifest.Digest}}'
\`\`\`

Then pull and deploy by digest, never by tag (see \`docs/operator/installation.md\`).

## Verify SBOM and provenance checksums

Download the SBOM/provenance assets from the GitHub Release into one
directory and run:

\`\`\`bash
sha256sum -c araf-${version}-sbom.sha256
sha256sum -c araf-${version}-provenance.sha256
\`\`\`

## Immutability caveat

The published digests are cryptographically and policy immutable (they are
the attestation subjects and the deployment contract), but GHCR does not
platform-enforce tag immutability: re-pushing a tag can change what the tag
resolves to. Always pin by digest and re-verify with the commands above;
never trust a tag alone.
EOF
}

main() {
  if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    usage
    exit 0
  fi

  require_tools

  local version=${RELEASE_VERSION:-}
  [[ -n "$version" ]] || fail "RELEASE_VERSION is unset (e.g. RELEASE_VERSION=v1.0.0-rc.12)"
  [[ "$version" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] \
    || fail "RELEASE_VERSION '$version' is not a release tag (expected vX.Y.Z[-suffix])"

  GH_TOKEN_RESOLVED=${GITHUB_TOKEN:-${GH_TOKEN:-}}
  [[ -n "$GH_TOKEN_RESOLVED" ]] || fail "GITHUB_TOKEN (or GH_TOKEN) is unset; needed for GHCR attestation fetch"
  export GH_TOKEN_RESOLVED
  GHCR_USER=${GHCR_USER:-${GITHUB_ACTOR:-}}
  [[ -n "$GHCR_USER" ]] || fail "GHCR_USER is unset and GITHUB_ACTOR is unavailable"
  GHCR_REGISTRY=${GHCR_REGISTRY:-ghcr.io}
  IMAGE_NAMESPACE=${IMAGE_NAMESPACE:-o3kio}

  local repo_root output_dir
  repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
  output_dir=${OUTPUT_DIR:-"$repo_root/dist/release"}
  local staging="$output_dir/.staging-$version"
  rm -rf "$staging"
  mkdir -p "$staging" "$output_dir/sbom" "$output_dir/provenance"

  # component short-name -> image repository (namespace is fixed per release).
  local -a components=(bff tenant-console operator-console)
  local -A image_repo=(
    [bff]="$IMAGE_NAMESPACE/araf-bff"
    [tenant-console]="$IMAGE_NAMESPACE/araf-tenant-console"
    [operator-console]="$IMAGE_NAMESPACE/araf-operator-console"
  )

  local digests_file="$output_dir/araf-${version}-digests.txt"
  : >"$digests_file"
  trap 'rm -rf "$staging"' EXIT

  local component repo image_ref digest index_json auth att_digest att_json sbom_layer prov_layer
  local platform_digest platform_json config_digest oci_tar
  for component in "${components[@]}"; do
    repo=${image_repo[$component]}
    image_ref="${GHCR_REGISTRY}/${repo}:${version}"
    echo "publish-release: resolving $image_ref"
    digest=$(resolve_digest "$image_ref")
    printf '%s %s@%s\n' "$component" "$image_ref" "$digest" >>"$digests_file"

    auth=$(ghcr_pull_token "$repo")

    index_json=$(fetch_manifest "$repo" "$digest" \
      "application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json" \
      "$auth")

    # linux/amd64 platform manifest + image config digests: published in
    # digests.txt and used to verify the saved OCI tarball.
    platform_digest=$(platform_manifest_digest "$index_json")
    [[ "$platform_digest" =~ ^sha256:[0-9a-f]{64}$ ]] \
      || fail "no linux/amd64 platform manifest in index for $image_ref"
    platform_json=$(fetch_manifest "$repo" "$platform_digest" \
      "application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json" \
      "$auth")
    config_digest=$(jq -r '.config.digest' <<<"$platform_json")
    [[ "$config_digest" =~ ^sha256:[0-9a-f]{64}$ ]] \
      || fail "no config digest in platform manifest for $image_ref"
    printf '%s-platform %s@%s\n' "$component" "$image_ref" "$platform_digest" >>"$digests_file"
    printf '%s-config %s\n' "$component" "$config_digest" >>"$digests_file"

    # docker-save tarball: the ghcr.io packages require authentication to
    # pull, so demo hosts install from this tarball instead.
    oci_tar="$output_dir/araf-${component}-${version}.oci.tar"
    save_and_verify_oci_tar "${image_ref}@${digest}" "$config_digest" "$oci_tar"
    echo "publish-release: saved + config-verified OCI tar for $component"

    att_digest=$(attestation_manifest_digest "$index_json")
    [[ -n "$att_digest" && "$att_digest" != "null" ]] \
      || fail "no attestation manifest found for $image_ref (image was not attested at build)"

    att_json=$(fetch_manifest "$repo" "$att_digest" \
      "application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json" \
      "$auth")
    sbom_layer=$(attestation_layer_digest "$att_json" "https://spdx.dev/Document")
    [[ -n "$sbom_layer" && "$sbom_layer" != "null" ]] \
      || fail "no SPDX SBOM layer in attestation manifest for $image_ref"
    prov_layer=$(attestation_layer_digest "$att_json" "https://slsa.dev/provenance/v1")
    [[ -n "$prov_layer" && "$prov_layer" != "null" ]] \
      || fail "no SLSA provenance layer in attestation manifest for $image_ref"

    fetch_blob "$repo" "$sbom_layer" \
      "$output_dir/sbom/${component}-${version}.spdx.json" "$auth"
    fetch_blob "$repo" "$prov_layer" \
      "$output_dir/provenance/${component}-${version}.provenance.json" "$auth"
    echo "publish-release: extracted SBOM + provenance for $component ($digest)"
  done

  validate_digests_file "$digests_file" || fail "generated digests file failed validation: $digests_file"

  # Bind the four runtime components to the exact source and the immutable
  # image indexes. The two BFF entries intentionally point at the one
  # canonical multi-binary araf-bff image.
  local source_sha="${SOURCE_SHA:-}"
  if [[ -z "$source_sha" ]]; then
    source_sha=$(git -C "$repo_root" rev-parse HEAD)
  fi
  SOURCE_SHA="$source_sha" RELEASE_VERSION="$version" \
    ARAF_DIGESTS_PATH="$digests_file" \
    ARAF_MANIFEST_PATH="$output_dir/araf-${version}-release-manifest.json" \
    node "$repo_root/scripts/generate-release-manifest.mjs"
  node "$repo_root/scripts/validate-release-manifest.mjs" \
    "$output_dir/araf-${version}-release-manifest.json"

  (
    cd "$output_dir"
    sha256sum "sbom/"*"-${version}.spdx.json" >"araf-${version}-sbom.sha256"
    sha256sum "provenance/"*"-${version}.provenance.json" >"araf-${version}-provenance.sha256"
    sha256sum "araf-bff-${version}.oci.tar" \
      "araf-tenant-console-${version}.oci.tar" \
      "araf-operator-console-${version}.oci.tar" >"araf-${version}-oci-tarballs.sha256"
  )

  cp "$repo_root/deploy/docker-compose.release.yml" "$staging/docker-compose.release.yml"
  cp "$digests_file" "$staging/digests.txt"
  cp "$output_dir/araf-${version}-release-manifest.json" "$staging/release-manifest.json"
  cp "$repo_root/release/manifest.schema.json" "$staging/manifest.schema.json"
  write_environment_md "$staging/ENVIRONMENT.md" "$version"
  write_verify_md "$staging/VERIFY.md" "$version" "$digests_file"

  local tarball="$output_dir/araf-${version}-deploy.tar.gz"
  tar -C "$staging" -czf "$tarball" \
    docker-compose.release.yml ENVIRONMENT.md digests.txt release-manifest.json manifest.schema.json VERIFY.md
  rm -rf "$staging"
  trap - EXIT

  echo "publish-release: wrote $tarball"
  echo "publish-release: wrote $digests_file"
  echo "publish-release: wrote $output_dir/araf-${version}-release-manifest.json"
  echo "publish-release: wrote $output_dir/araf-${version}-sbom.sha256"
  echo "publish-release: wrote $output_dir/araf-${version}-provenance.sha256"
  echo "publish-release: wrote $output_dir/araf-${version}-oci-tarballs.sha256"
  echo "publish-release: done (assets under $output_dir; release creation is a separate step)"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
