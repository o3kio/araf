#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cat >"$work/digests.txt" <<'EOF'
bff ghcr.io/o3kio/araf-bff:v1.0.0-rc.99@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
tenant-console ghcr.io/o3kio/araf-tenant-console:v1.0.0-rc.99@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
operator-console ghcr.io/o3kio/araf-operator-console:v1.0.0-rc.99@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
EOF
RELEASE_VERSION=v1.0.0-rc.99 \
SOURCE_SHA=0123456789abcdef0123456789abcdef01234567 \
ARAF_DIGESTS_PATH="$work/digests.txt" \
ARAF_MANIFEST_PATH="$work/manifest.json" \
node "$root_dir/scripts/generate-release-manifest.mjs"
node "$root_dir/scripts/validate-release-manifest.mjs" "$work/manifest.json"
node -e 'const m=require(process.argv[1]); if(m.compatibility.required_o3k_api_contract!=="o3k-native-iam-v1") process.exit(1)' "$work/manifest.json"
node -e 'const s=require(process.argv[1]); if(s.properties.schema_version.const!==1) process.exit(1)' "$root_dir/release/manifest.schema.json"
node -e 'const fs=require("fs"); const p=process.argv[1]; const m=JSON.parse(fs.readFileSync(p)); m.artifacts.tenant_bff.tag="v1.0.0-rc.98"; fs.writeFileSync(p, JSON.stringify(m));' "$work/manifest.json"
if node "$root_dir/scripts/validate-release-manifest.mjs" "$work/manifest.json" >/dev/null 2>&1; then
  echo "release manifest gate: version mismatch was accepted" >&2
  exit 1
fi
echo "release manifest gate: PASS"
