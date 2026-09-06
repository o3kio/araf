#!/usr/bin/env bash
set -euo pipefail
command -v curl >/dev/null
command -v python3 >/dev/null
command -v sqlite3 >/dev/null

araf_root="${ARAF_ROOT:-/root/araf}"
o3k_root="${O3K_ROOT:-/root/o3k-rust}"
kc_port="${O3K_P12_7_KEYCLOAK_PORT:?}"
issuer="${O3K_P12_7_ISSUER:?}"
discovery="${O3K_P12_7_DISCOVERY_URL:?}"
db="${O3K_P12_7_SQLITE_PATH:?}"
workdir="${O3K_P12_7_WORKDIR:?}"
o3k_port="${P2_1_O3K_PORT:-18182}"
operator_port="${P2_1_OPERATOR_PORT:-18183}"
redirect_uri="http://127.0.0.1:${operator_port}/api/v1/auth/callback"
client_id=o3k-operator-p2-1
client_secret=p2-1-operator-client-secret
o3k_pid=""; bff_pid=""
cleanup() { [[ -n "${bff_pid}" ]] && kill "${bff_pid}" 2>/dev/null || true; [[ -n "${o3k_pid}" ]] && kill "${o3k_pid}" 2>/dev/null || true; }
trap cleanup EXIT

admin_token="$(curl -fsS -X POST "http://127.0.0.1:${kc_port}/realms/master/protocol/openid-connect/token" -d grant_type=password -d client_id=admin-cli -d username=p12-7-admin -d password="${O3K_P12_7_KEYCLOAK_ADMIN_PASSWORD}" | python3 -c 'import json,sys; print(json.load(sys.stdin)["access_token"])')"
curl -fsS -X POST "http://127.0.0.1:${kc_port}/admin/realms/o3k-p12-7/clients" -H "Authorization: Bearer ${admin_token}" -H 'Content-Type: application/json' -d "{\"clientId\":\"${client_id}\",\"enabled\":true,\"publicClient\":false,\"clientAuthenticatorType\":\"client-secret\",\"secret\":\"${client_secret}\",\"standardFlowEnabled\":true,\"directAccessGrantsEnabled\":true,\"redirectUris\":[\"${redirect_uri}\"],\"protocol\":\"openid-connect\"}" >/dev/null

sqlite3 "${db}" 'PRAGMA wal_checkpoint(TRUNCATE);'
(
  cd "${o3k_root}"
  O3K_LISTEN_ADDR="127.0.0.1:${o3k_port}" O3K_DATA_DIR="$(dirname "${db}")" O3K_PROVIDER=fake O3K_BOOTSTRAP_PASSWORD="${O3K_P12_7_BOOTSTRAP_SECRET}" O3K_TOKEN_SIGNING_KEY=p2-1-live-process-signing-key-at-least-32-bytes O3K_OIDC_TRUST_ID=p12-7-keycloak O3K_OIDC_ISSUER="${issuer}" O3K_OIDC_AUDIENCE=o3k O3K_OIDC_DISCOVERY_URL="${discovery}" O3K_OIDC_ALLOW_INSECURE_LOCAL=true cargo run --quiet -p o3kd
) >"${workdir}/p2-1-o3kd.log" 2>&1 & o3k_pid=$!
for _ in $(seq 1 60); do curl -fsS "http://127.0.0.1:${o3k_port}/healthz" >/dev/null 2>&1 && break; sleep 1; done
curl -fsS "http://127.0.0.1:${o3k_port}/healthz" >/dev/null
(
  cd "${araf_root}/backend"
  ARAF_RUNTIME_PROFILE=test ARAF_UPSTREAM_ADAPTER=o3k ARAF_OPERATOR_OIDC_CLIENT_ID="${client_id}" ARAF_OPERATOR_OIDC_CLIENT_SECRET="${client_secret}" ARAF_OPERATOR_OIDC_ISSUER_URL="${issuer}" ARAF_OPERATOR_OIDC_REDIRECT_URI="${redirect_uri}" O3K_URL="http://127.0.0.1:${o3k_port}" ARAF_OPERATOR_BFF_PORT="${operator_port}" cargo run --quiet -p operator-bff
) >"${workdir}/p2-1-operator-bff.log" 2>&1 & bff_pid=$!
for _ in $(seq 1 60); do curl -fsS "http://127.0.0.1:${operator_port}/healthz" >/dev/null 2>&1 && break; sleep 1; done
curl -fsS "http://127.0.0.1:${operator_port}/healthz" >/dev/null

jar="${workdir}/p2-1-operator.cookies"
curl -fsS -D "${workdir}/p2-1-login.headers" -o /dev/null "http://127.0.0.1:${operator_port}/api/v1/auth/login"
auth_url="$(sed -n 's/^Location: //Ip' "${workdir}/p2-1-login.headers" | tr -d '\r' | head -1)"
curl -fsS -c "${jar}" -b "${jar}" "${auth_url}" -o "${workdir}/p2-1-auth.html"
form_action="$(python3 - "${workdir}/p2-1-auth.html" "${auth_url}" <<'PY'
import re, sys
from urllib.parse import urljoin
html=open(sys.argv[1], encoding='utf-8').read()
m=re.search(r'<form[^>]+action=["\x27]([^"\x27]+)', html, re.I)
if not m: raise SystemExit('provider login form missing')
print(urljoin(sys.argv[2], m.group(1).replace('&amp;', '&')))
PY
)"
curl -sS -D "${workdir}/p2-1-provider.headers" -o /dev/null -w '%{http_code}' -c "${jar}" -b "${jar}" -X POST "${form_action}" -H 'Content-Type: application/x-www-form-urlencoded' --data-urlencode username=operator --data-urlencode "password=${P12_8_OPERATOR_PASSWORD}" --data-urlencode credentialId= --max-redirs 0 >/dev/null || true
callback="$(sed -n 's/^Location: //Ip' "${workdir}/p2-1-provider.headers" | tr -d '\r' | head -1)"
test -n "${callback}"
status="$(curl -sS -D "${workdir}/p2-1-callback.headers" -o /dev/null -w '%{http_code}' -c "${jar}" -b "${jar}" "${callback}")"
case "${status}" in 302|303) ;; *) exit 1;; esac
session="$(curl -fsS -b "${jar}" "http://127.0.0.1:${operator_port}/api/v1/auth/session")"
echo "${session}" | grep -q '"authenticated":true'
profile="$(curl -fsS -b "${jar}" "http://127.0.0.1:${operator_port}/api/v1/operator/profile")"
echo "${profile}" | grep -q '"scope":"system"'
echo "${profile}" | grep -q 'operator-console'
if grep -Eiq 'access_token|refresh_token|id_token' "${jar}" "${workdir}/p2-1-callback.headers"; then exit 1; fi
curl -fsS -X POST -b "${jar}" "http://127.0.0.1:${operator_port}/api/v1/auth/logout" >/dev/null

alice_system="$(curl -sS -w '\n%{http_code}' -H 'content-type: application/json' -X POST "http://127.0.0.1:${o3k_port}/o3k/v1/identity/tokens" -d "{\"auth\":{\"method\":\"federated\",\"federated\":{\"access_token\":\"${O3K_P12_7_ALICE_TOKEN}\",\"scope\":{\"kind\":\"system\"}}}}")"
echo "${alice_system}" | tail -1 | grep -Eq '^4(01|03)$'
echo 'P2.1 real Operator process evidence: PASS'
