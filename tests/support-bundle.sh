#!/usr/bin/env bash
set -euo pipefail

out=${1:-support-bundle-$(date -u +%Y%m%dT%H%M%SZ)}
mkdir -p "$out"
umask 077
{
  date -u +%FT%TZ
  uname -a
  printf 'version='; curl -fsS "${ARAF_TENANT_BFF_URL:-http://127.0.0.1:8080}/version" || true
  printf '\nreadiness='; curl -fsS "${ARAF_TENANT_BFF_URL:-http://127.0.0.1:8080}/readyz" || true
  printf '\nmetrics='; curl -fsS "${ARAF_TENANT_BFF_URL:-http://127.0.0.1:8080}/metrics" || true
} >"$out/diagnostics.txt"
# Deliberately collect names/status only. Never copy the process environment,
# cookies, authorization headers, session files, journals or request bodies.
env | sed -E 's/=.*/=<redacted>/' | sort >"$out/environment-names.txt"
printf '%s\n' 'Support bundle is intentionally redacted; attach logs separately after secret review.' >"$out/README.txt"
tar -czf "$out.tar.gz" "$out"
rm -rf "$out"
echo "$out.tar.gz"
