#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out_dir=${ARAF_SECURITY_OUT:-$root_dir/target/security}
mkdir -p "$out_dir"

require_command() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "security gate: required command '$1' is unavailable" >&2
    exit 1
  }
}

require_command cargo
require_command cargo-audit
require_command cargo-deny
require_command pnpm
require_command node

# Workflow actions are immutable inputs. Every external action must resolve to
# a full commit SHA; rejecting only well-known moving tags would leave custom
# branches/tags as an avoidable supply-chain bypass.
action_ref_pattern='^[[:space:]]*uses:[[:space:]]+[^@[:space:]]+@'
unpinned_action_pattern='@[0-9a-fA-F]{40}([[:space:]]+#.*)?$'
if command -v rg >/dev/null 2>&1; then
  action_ref_scan=(rg -n --glob '*.yml' --glob '*.yaml' "$action_ref_pattern" "$root_dir/.github/workflows")
  unpinned_action_scan=(rg -n -v "$unpinned_action_pattern")
else
  action_ref_scan=(grep -RInE --include='*.yml' --include='*.yaml' "$action_ref_pattern" "$root_dir/.github/workflows")
  unpinned_action_scan=(grep -vE "$unpinned_action_pattern")
fi
if "${action_ref_scan[@]}" | "${unpinned_action_scan[@]}"; then
  echo "security gate: workflow action must be pinned to a full commit SHA" >&2
  exit 1
fi

release_workflow="$root_dir/.github/workflows/release-images.yml"
if command -v rg >/dev/null 2>&1; then
  release_has_provenance=(rg -q 'provenance:[[:space:]]+mode=max' "$release_workflow")
  release_has_sbom=(rg -q 'sbom:[[:space:]]+true' "$release_workflow")
  release_has_digest=(rg -q 'subject-digest:' "$release_workflow")
else
  release_has_provenance=(grep -Eq 'provenance:[[:space:]]+mode=max' "$release_workflow")
  release_has_sbom=(grep -Eq 'sbom:[[:space:]]+true' "$release_workflow")
  release_has_digest=(grep -Eq 'subject-digest:' "$release_workflow")
fi
if ! "${release_has_provenance[@]}" || ! "${release_has_sbom[@]}" || ! "${release_has_digest[@]}"; then
  echo "security gate: release workflow is missing SBOM or provenance enforcement" >&2
  exit 1
fi

base_image_scan_pattern='^FROM[[:space:]]+(rust|node|nginx|nginxinc|gcr\.io/)'
if command -v rg >/dev/null 2>&1; then
  base_image_scan=(rg -n "$base_image_scan_pattern" "$root_dir/backend/Dockerfile" "$root_dir/Dockerfile.frontend")
  unpinned_scan=(rg -v '@sha256:')
else
  base_image_scan=(grep -nE "$base_image_scan_pattern" "$root_dir/backend/Dockerfile" "$root_dir/Dockerfile.frontend")
  unpinned_scan=(grep -v '@sha256:')
fi
if "${base_image_scan[@]}" | "${unpinned_scan[@]}"; then
  echo "security gate: release Dockerfile base must be digest pinned" >&2
  exit 1
fi

# Fail closed on the two highest-risk classes that can be checked without
# credentials: browser token persistence and committed private keys.
token_pattern='(localStorage|sessionStorage)\.(setItem|getItem).*([Tt]oken|[Ss]ession)|Bearer[[:space:]]+[A-Za-z0-9._-]{24,}'
if command -v rg >/dev/null 2>&1; then
  token_scan=(rg -n --glob '!docs/**' --glob '!*.map')
else
  token_scan=(grep -RInE --exclude='*.map' --exclude-dir=docs --exclude-dir=node_modules --exclude-dir=target)
fi
if "${token_scan[@]}" "$token_pattern" "$root_dir/apps" "$root_dir/packages"; then
  echo "security gate: browser token persistence or embedded bearer token found" >&2
  exit 1
fi
key_pattern='-----BEGIN (RSA|EC|OPENSSH|PRIVATE) KEY-----|AKIA[0-9A-Z]{16}'
if command -v rg >/dev/null 2>&1; then
  key_scan=(rg -n --glob '!docs/**' --glob '!*.md' --)
else
  key_scan=(grep -RInE --exclude='*.md' --exclude-dir=docs --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git --exclude-dir=.o3k-rust --)
fi
if "${key_scan[@]}" "$key_pattern" "$root_dir"; then
  echo "security gate: private key or AWS access-key pattern found" >&2
  exit 1
fi

(cd "$root_dir/backend" && cargo metadata --locked --format-version 1 >"$out_dir/cargo-sbom.json")
(cd "$root_dir/backend" && cargo audit)
# cargo-deny performs the deterministic license/source/ban policy checks. The
# advisory database is checked by cargo-audit above; keeping these checks
# separate also lets CI report database-format incompatibilities explicitly.
(cd "$root_dir/backend" && cargo deny check licenses bans sources)
(cd "$root_dir" && pnpm list --json --depth Infinity >"$out_dir/frontend-dependencies.json")
(cd "$root_dir" && pnpm audit --prod --audit-level=high)
(cd "$root_dir" && node tests/check-frontend-licenses.mjs)

# These checks are deliberately source-only and complement (rather than
# replace) runtime security tests. Test fixtures are excluded from the
# executable-content check, but remain covered by the secret scan below.
dynamic_code_pattern='dangerouslySetInnerHTML|new[[:space:]]+Function[[:space:]]*\(|(^|[^[:alnum:]_])eval[[:space:]]*\('
if command -v rg >/dev/null 2>&1; then
  dynamic_code_scan=(rg -n --glob '!*.test.*' --glob '!**/target/**')
else
  dynamic_code_scan=(grep -RInE --exclude='*.test.*' --exclude-dir=target --exclude-dir=node_modules --exclude-dir=dist)
fi
if "${dynamic_code_scan[@]}" "$dynamic_code_pattern" "$root_dir/apps" "$root_dir/packages"; then
  echo "security gate: executable HTML or dynamic code construction found" >&2
  exit 1
fi

if git -C "$root_dir" grep -nI -E \
  -e '-----BEGIN (RSA|EC|OPENSSH|PRIVATE) KEY-----|AKIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9_]{20,}|xox[baprs]-[0-9A-Za-z-]{20,}' \
  -- ':!docs/**' ':!*.md' ':!tests/security-release-gate.sh'; then
    echo "security gate: tracked credential pattern found" >&2
    exit 1
fi

cat >"$out_dir/manifest.json" <<EOF
{
  "araf_sha": "$(git -C "$root_dir" rev-parse HEAD)",
  "cargo_audit": "$(cargo-audit --version | head -1)",
  "cargo_deny": "$(cargo-deny --version | head -1)",
  "node": "$(node --version)",
  "pnpm": "$(pnpm --version)",
  "workflow_actions": "commit-pinned",
  "artifact_requirements": ["digest", "sbom", "keyless-provenance"]
}
EOF
cat >"$out_dir/policy.txt" <<'EOF'
ARAF release security policy v1
- Rust and frontend dependency advisories: no unresolved HIGH/BLOCKER.
- Licenses: Rust uses deny.toml; frontend packages use policy/frontend-licenses.txt.
- Artifacts require a digest, SBOM and CI provenance attestation before publication.
- Secret scanning and the browser-token gate are mandatory on every release candidate.
- Workflow actions and release scanner images must be immutable (commit/digest pinned).
- SLSA level is not claimed; CI provenance is required and verified per release.
EOF
echo "security release gate: static checks and dependency inventories PASS"
