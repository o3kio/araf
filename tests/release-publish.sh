#!/usr/bin/env bash
# Guard-rail tests for scripts/publish-release.sh. Everything here runs
# offline: argument/env validation, fail-closed image resolution against a
# mocked docker, and digests.txt format validation. The happy path (real
# GHCR attestation extraction) is exercised by release-publish.yml itself.
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
script="$root_dir/scripts/publish-release.sh"

fail() { echo "release-publish test: $*" >&2; exit 1; }

[[ -x "$script" ]] || fail "scripts/publish-release.sh is missing or not executable"

# ---------------------------------------------------------------- helpers
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

expect_fail() {
  # $1: description; runs remaining args, expects non-zero exit and a message
  # on stderr. Prints captured stderr for inspection by the caller's greps.
  local desc=$1
  shift
  if "$@" 2>"$work/stderr"; then
    fail "$desc: expected failure but command succeeded"
  fi
}

# ---------------------------------------------------------------- usage
"$script" --help >"$work/help" 2>&1 \
  || fail "--help must exit zero"
grep -q "RELEASE_VERSION" "$work/help" \
  || fail "--help output must document RELEASE_VERSION"
grep -q "dist/release" "$work/help" \
  || fail "--help output must document the output layout"

# ---------------------------------------------------------------- env validation
unset RELEASE_VERSION GITHUB_TOKEN GH_TOKEN GHCR_USER || true
expect_fail "unset RELEASE_VERSION" env -u RELEASE_VERSION -u GITHUB_TOKEN -u GH_TOKEN -u GHCR_USER bash "$script"
grep -q "RELEASE_VERSION is unset" "$work/stderr" \
  || fail "unset RELEASE_VERSION must name the missing variable"

expect_fail "malformed RELEASE_VERSION" \
  env RELEASE_VERSION=1.0.0 GITHUB_TOKEN=dummy GHCR_USER=tester bash "$script"
grep -q "not a release tag" "$work/stderr" \
  || fail "malformed RELEASE_VERSION must be rejected"

expect_fail "missing GITHUB_TOKEN" \
  env RELEASE_VERSION=v1.0.0-rc.12 GHCR_USER=tester bash "$script"
grep -q "GITHUB_TOKEN" "$work/stderr" \
  || fail "missing token must be rejected before any registry access"

# ---------------------------------------------------------------- fail-closed image resolution (mocked docker)
mockbin="$work/bin"
mkdir -p "$mockbin"
cat >"$mockbin/docker" <<'EOF'
#!/usr/bin/env bash
# Mock: every buildx imagetools inspect fails as if the image were missing.
exit 42
EOF
chmod +x "$mockbin/docker"
expect_fail "unresolvable image ref (mocked docker)" \
  env PATH="$mockbin:/usr/bin:/bin" OUTPUT_DIR="$work/out" \
      RELEASE_VERSION=v1.0.0-rc.12 GITHUB_TOKEN=dummy GHCR_USER=tester \
      bash "$script"
grep -q "cannot resolve image ref" "$work/stderr" \
  || fail "unresolvable image must fail closed with a clear error"
grep -q "araf-bff:v1.0.0-rc.12" "$work/stderr" \
  || fail "resolution failure must name the offending image ref"
[[ -e "$work/out/araf-v1.0.0-rc.12-digests.txt" ]] \
  || fail "script must honor OUTPUT_DIR (refusing to write outside the sandbox)"

# ---------------------------------------------------------------- digests.txt validation (sourced function)
# shellcheck disable=SC1090
source "$script"

good="$work/digests-good.txt"
cat >"$good" <<'EOF'
bff ghcr.io/o3kio/araf-bff:v1.0.0-rc.12@sha256:bc717ecdbbbf3ea673efe168c90419936677d644aa0ae25af4eb84906cd744ba
tenant-console ghcr.io/o3kio/araf-tenant-console:v1.0.0-rc.12@sha256:25f5fe41927f68db3dafd49597c6b8cb4520bca2dd45ec131d1372474ef3e5e5
operator-console ghcr.io/o3kio/araf-operator-console:v1.0.0-rc.12@sha256:cbbad76033eced4d4290c9848e0665a23c30bd4a7c647077ba0cf03150666b18
EOF
validate_digests_file "$good" || fail "valid digests file must pass validation"

bad_ref="$work/digests-bad-ref.txt"
sed 's/@sha256:[0-9a-f]\{64\}//' "$good" >"$bad_ref"
if validate_digests_file "$bad_ref" 2>"$work/stderr"; then
  fail "digest-less image ref must fail validation"
fi
grep -q "bad image ref" "$work/stderr" \
  || fail "bad ref validation must explain the failure"

bad_component="$work/digests-bad-component.txt"
sed 's/^bff /BFF_1 /' "$good" >"$bad_component"
if validate_digests_file "$bad_component" 2>"$work/stderr"; then
  fail "malformed component must fail validation"
fi

bad_digest="$work/digests-bad-digest.txt"
sed 's/sha256:bc717ecdbbbf3ea673efe168c90419936677d644aa0ae25af4eb84906cd744ba/sha256:zzz/' "$good" >"$bad_digest"
if validate_digests_file "$bad_digest" 2>"$work/stderr"; then
  fail "non-hex digest must fail validation"
fi

: >"$work/digests-empty.txt"
if validate_digests_file "$work/digests-empty.txt" 2>/dev/null; then
  fail "empty digests file must fail validation"
fi

# ---------------------------------------------------------------- workflow pinning style
workflow="$root_dir/.github/workflows/release-publish.yml"
[[ -s "$workflow" ]] || fail "release-publish workflow is missing"
# Every actions/ use must be pinned by a full-length commit SHA, matching
# release-images.yml (defense against tag-mutated action code).
unpinned=$(grep -E 'uses: actions/' "$workflow" | grep -vE '@[0-9a-f]{40}' || true)
[[ -z "$unpinned" ]] || fail "workflow actions must be pinned by commit SHA:\n$unpinned"
grep -q -- "--verify-tag" "$workflow" \
  || fail "release creation must verify the tag"
if grep -qE 'gh release (create|edit|delete).*(--force|--clobber|--latest=false)' "$workflow"; then
  fail "workflow must never force/recreate a release"
fi

echo "release-publish guard-rail tests: PASS"
