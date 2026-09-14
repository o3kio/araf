#!/usr/bin/env bash
set -euo pipefail

umask 077
out=${1:-support-bundle-$(date -u +%Y%m%dT%H%M%SZ)}
mkdir -p "$out"

# Keep collector output useful without turning it into a credential copier.
# Explicit log paths are opt-in and are filtered line-by-line; the collector
# never reads the process environment, session store or compatibility journal.
redact_stream() {
  sed -E \
    -e 's/(Bearer[[:space:]]+)[A-Za-z0-9._~+\/-]+=*/\1<redacted>/Ig' \
    -e 's/((Authorization|Cookie|X-CSRF-Token|token|password|passwd|secret|private[_-]?key|session[_-]?key)[=:][[:space:]]*)[^[:space:]]+/\1<redacted>/Ig' \
    -e "s/(client_secret|access_token|refresh_token|oidc_token)[\\\"'=:\\t ]+[^, }]+/\\1=<redacted>/Ig"
}

{
  date -u +%FT%TZ
  uname -a
  printf 'version='; curl -fsS "${ARAF_TENANT_BFF_URL:-http://127.0.0.1:8080}/version" || true
  printf '\nreadiness='; curl -fsS "${ARAF_TENANT_BFF_URL:-http://127.0.0.1:8080}/readyz" || true
  printf '\nmetrics='; curl -fsS "${ARAF_TENANT_BFF_URL:-http://127.0.0.1:8080}/metrics" || true
} | redact_stream >"$out/diagnostics.txt"
# Deliberately collect names/status only. Never copy the process environment,
# cookies, authorization headers, session files, journals or request bodies.
env | sed -E 's/=.*/=<redacted>/' | sort >"$out/environment-names.txt"
if [[ -n "${ARAF_SUPPORT_BUNDLE_LOG_FILES:-}" ]]; then
  : >"$out/sanitized-logs.txt"
  IFS=',' read -r -a log_files <<<"$ARAF_SUPPORT_BUNDLE_LOG_FILES"
  for log_file in "${log_files[@]}"; do
    [[ -f "$log_file" ]] || continue
    printf '\n--- %s ---\n' "$(basename "$log_file")" >>"$out/sanitized-logs.txt"
    redact_stream <"$log_file" >>"$out/sanitized-logs.txt"
  done
fi
printf '%s\n' 'Support bundle is intentionally redacted; attach logs separately after secret review.' >"$out/README.txt"
parent_dir=$(dirname -- "$out")
bundle_name=$(basename -- "$out")
tar -czf "$out.tar.gz" -C "$parent_dir" "$bundle_name"
rm -rf -- "$out"
echo "$out.tar.gz"
