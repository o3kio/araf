#!/usr/bin/env bash
set -euo pipefail

# Static release-contract gate.  The actual rollout must be run by the release
# owner against a disposable environment; this gate prevents an upgrade test
# from accidentally rebuilding source or replacing durable state.
root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
compose="$root_dir/deploy/docker-compose.release.yml"
chart="$root_dir/deploy/helm/araf"

fail() { echo "package upgrade/rollback gate: $*" >&2; exit 1; }

[[ -s "$compose" ]] || fail "release compose is missing"
[[ -s "$chart/Chart.yaml" ]] || fail "Helm chart is missing"
rg -q 'image: \$\{ARAF_BFF_IMAGE:\?[^}]+\}@\$\{ARAF_BFF_DIGEST:\?' "$compose" \
  || fail "Compose must require digest-pinned images"
! rg -n 'build:|\.\./|/root|/workspace' "$compose" \
  || fail "release Compose must not build or mount a source checkout"
rg -q 'required "image\.digest is required"' "$chart/templates/_helpers.tpl" \
  || fail "Helm must require an immutable image digest"
rg -q 'readOnlyRootFilesystem: true' "$chart/templates/deployment.yaml" \
  || fail "Helm deployment must use a read-only root filesystem"
rg -q 'ARAF_SESSION_STORE_KEY' "$compose" "$chart/templates/deployment.yaml" \
  || fail "durable session encryption key must be externally referenced"

if command -v helm >/dev/null 2>&1; then
  helm lint "$chart" --set image.digest=sha256:"$(printf '%064d' 0)" \
    --set backend.o3kUrl=https://o3k.example.invalid \
    --set oidc.tenantIssuerUrl=https://idp.example.invalid \
    --set oidc.tenantClientId=placeholder \
    --set oidc.tenantRedirectUri=https://tenant.example.invalid/callback \
    --set oidc.operatorIssuerUrl=https://idp.example.invalid \
    --set oidc.operatorClientId=placeholder \
    --set oidc.operatorRedirectUri=https://operator.example.invalid/callback >/dev/null
fi

echo "package upgrade/rollback gate: PASS (artifact immutability and state-preserving rollout contract)"
