#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out_dir=${ARAF_SECURITY_OUT:-$root_dir/target/security}
mkdir -p "$out_dir"

# Fail closed on the two highest-risk classes that can be checked without
# credentials: browser token persistence and committed private keys.
if rg -n --glob '!docs/**' --glob '!*.map' \
  '(localStorage|sessionStorage)\.(setItem|getItem).*([Tt]oken|[Ss]ession)|Bearer[[:space:]]+[A-Za-z0-9._-]{24,}' \
  "$root_dir/apps" "$root_dir/packages"; then
  echo "security gate: browser token persistence or embedded bearer token found" >&2
  exit 1
fi
if rg -n --glob '!docs/**' --glob '!*.md' \
  -- '-----BEGIN (RSA|EC|OPENSSH|PRIVATE) KEY-----|AKIA[0-9A-Z]{16}' \
  "$root_dir"; then
  echo "security gate: private key or AWS access-key pattern found" >&2
  exit 1
fi

(cd "$root_dir/backend" && cargo metadata --locked --format-version 1 >"$out_dir/cargo-sbom.json")
(cd "$root_dir/backend" && cargo audit)
# cargo-deny performs the deterministic license/source/ban policy checks. The
# advisory database is checked by cargo-audit above; keeping these checks
# separate also lets CI report database-format incompatibilities explicitly.
(cd "$root_dir/backend" && cargo deny check licenses bans sources --disable-fetch)
(cd "$root_dir" && pnpm list --json --depth Infinity >"$out_dir/frontend-dependencies.json")
cat >"$out_dir/policy.txt" <<'EOF'
ARAF release security policy v1
- Rust and frontend dependency advisories: no unresolved HIGH/BLOCKER.
- Licenses: Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC and Unicode-DFS-2016 allowlist.
- Artifacts require a digest, SBOM and CI provenance attestation before publication.
- Secret scanning and the browser-token gate are mandatory on every release candidate.
EOF
echo "security release gate: static checks and dependency inventories PASS"
