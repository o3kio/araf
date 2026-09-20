#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
compose="$root_dir/deploy/docker-compose.release.yml"
helm_chart="$root_dir/deploy/helm/araf"

command -v docker >/dev/null 2>&1 || {
  echo "release deployment validation: docker is required" >&2
  exit 1
}
docker compose version >/dev/null
command -v helm >/dev/null 2>&1 || {
  echo "release deployment validation: helm is required" >&2
  exit 1
}

if grep -Eq '^[[:space:]]*(build|context):' "$compose"; then
  echo "release deployment validation: release Compose must never build from source" >&2
  exit 1
fi
image_count=$(grep -Ec '^[[:space:]]+image:' "$compose")
digest_image_count=$(grep -Ec '^[[:space:]]+image:.*@\$\{[^}]+:\?[^}]+\}' "$compose")
if [[ "$image_count" -ne 4 || "$digest_image_count" -ne 4 ]]; then
  echo "release deployment validation: all four runtime images must be digest-pinned" >&2
  exit 1
fi

env_file=$(mktemp)
trap 'rm -f "$env_file"' EXIT
cat >"$env_file" <<'EOF'
ARAF_BFF_IMAGE=ghcr.io/o3kio/araf-bff:v1.0.0-rc.validation
ARAF_BFF_DIGEST=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
ARAF_TENANT_CONSOLE_IMAGE=ghcr.io/o3kio/araf-tenant-console:v1.0.0-rc.validation
ARAF_TENANT_CONSOLE_DIGEST=sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
ARAF_OPERATOR_CONSOLE_IMAGE=ghcr.io/o3kio/araf-operator-console:v1.0.0-rc.validation
ARAF_OPERATOR_CONSOLE_DIGEST=sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
ARAF_UPSTREAM_ADAPTER=o3k
ARAF_TENANT_PUBLIC_URL=https://tenant.example
ARAF_OPERATOR_PUBLIC_URL=https://operator.example
ARAF_SESSION_STORE_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
ARAF_TENANT_OIDC_CLIENT_ID=tenant
ARAF_TENANT_OIDC_CLIENT_SECRET=secret
ARAF_TENANT_OIDC_ISSUER_URL=https://issuer.example
ARAF_TENANT_OIDC_REDIRECT_URI=https://tenant.example/callback
ARAF_OPERATOR_OIDC_CLIENT_ID=operator
ARAF_OPERATOR_OIDC_CLIENT_SECRET=secret
ARAF_OPERATOR_OIDC_ISSUER_URL=https://issuer.example
ARAF_OPERATOR_OIDC_REDIRECT_URI=https://operator.example/callback
EOF

docker compose --env-file "$env_file" -f "$compose" config >/dev/null
helm lint "$helm_chart" \
  --set image.digest=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  --set frontendImage.tenant.digest=sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
  --set frontendImage.operator.digest=sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc \
  >/dev/null

echo "release deployment validation: PASS"
