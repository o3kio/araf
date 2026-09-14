#!/usr/bin/env bash
set -euo pipefail

# Synthetic-secret regression for the operator collector. Values are placed in
# the environment and an opt-in log; the resulting archive must contain none
# of them. This test never uses a real credential or endpoint.
root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
work_dir=$(mktemp -d)
trap 'rm -rf -- "$work_dir"' EXIT
log_file="$work_dir/input.log"
bundle_base="$work_dir/support-bundle"
oidc_marker='oidc-client-secret-SYNTHETIC-DO-NOT-USE'
bearer_marker='eyJsynthetic.bearer.marker'
session_marker='c2Vzc2lvbi1rZXktU1lOVEhFVElD'
password_marker='password-SYNTHETIC-DO-NOT-USE'
printf 'Authorization: Bearer %s password=%s client_secret=%s session_key=%s\n' \
  "$bearer_marker" "$password_marker" "$oidc_marker" "$session_marker" >"$log_file"

archive=$(
  ARAF_TENANT_BFF_URL='http://127.0.0.1:9' \
  OIDC_CLIENT_SECRET="$oidc_marker" \
  SYNTHETIC_BEARER="$bearer_marker" \
  ARAF_SESSION_STORE_KEY="$session_marker" \
  DATABASE_PASSWORD="$password_marker" \
  ARAF_SUPPORT_BUNDLE_LOG_FILES="$log_file" \
    "$root_dir/tests/support-bundle.sh" "$bundle_base" 2>/dev/null
)
[[ -s "$archive" ]]
for marker in "$oidc_marker" "$bearer_marker" "$session_marker" "$password_marker"; do
  if tar -xOzf "$archive" | grep -Fq -- "$marker"; then
    echo "support bundle leaked synthetic secret marker" >&2
    exit 1
  fi
done
tar -tzf "$archive" | grep -q 'environment-names.txt'
echo 'support-bundle security gate: PASS (synthetic secret markers absent)'
