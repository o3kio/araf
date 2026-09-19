#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
version=${ARAF_VERSION:-v1.0.0-rc.14}
source_sha=$(git -C "$root_dir" rev-parse HEAD)
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

ARAF_VERSION="$version" ARAF_GIT_SHA="$source_sha" \
ARAF_BFF_DIGEST=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
ARAF_TENANT_CONSOLE_DIGEST=sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
ARAF_OPERATOR_CONSOLE_DIGEST=sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc \
ARAF_MANIFEST_PATH="$tmp_dir/manifest.json" ARAF_MANIFEST_SHA_PATH="$tmp_dir/manifest.sha256" \
node "$root_dir/scripts/generate-release-manifest.mjs"
node "$root_dir/scripts/validate-release-manifest.mjs" "$tmp_dir/manifest.json"

node -e 'const c=require(process.argv[1]); if(c.schema_version!==1 || c.fixture_mode_allowed!==false || c.o3k.api_contract!=="o3k.io/v1") process.exit(1)' \
  "$root_dir/backend/console-bff-core/contracts/release-contract.json"

if rg -n '(^|[[:space:]])build:|context:|latest|main|edge|stable|nightly' "$root_dir/deploy/docker-compose.release.yml"; then
  echo "release contract gate: moving tags or source builds found" >&2
  exit 1
fi
if ! rg -q 'source_build_required: false|source_build_required' "$tmp_dir/manifest.json"; then
  echo "release contract gate: source-build requirement missing" >&2
  exit 1
fi
echo "release contract gate: PASS"
