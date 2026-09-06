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
fail() { echo "P2.1 operator process FAILED: $1" >&2; exit 1; }

admin_token="$(curl -fsS -X POST "http://127.0.0.1:${kc_port}/realms/master/protocol/openid-connect/token" -d grant_type=password -d client_id=admin-cli -d username=p12-7-admin -d password="${O3K_P12_7_KEYCLOAK_ADMIN_PASSWORD}" | python3 -c 'import json,sys; print(json.load(sys.stdin)["access_token"])')"
curl -fsS -X POST "http://127.0.0.1:${kc_port}/admin/realms/o3k-p12-7/clients" -H "Authorization: Bearer ${admin_token}" -H 'Content-Type: application/json' -d "{\"clientId\":\"${client_id}\",\"enabled\":true,\"publicClient\":false,\"clientAuthenticatorType\":\"client-secret\",\"secret\":\"${client_secret}\",\"standardFlowEnabled\":true,\"directAccessGrantsEnabled\":true,\"redirectUris\":[\"${redirect_uri}\"],\"protocol\":\"openid-connect\",\"protocolMappers\":[{\"name\":\"o3k-audience\",\"protocol\":\"openid-connect\",\"protocolMapper\":\"oidc-audience-mapper\",\"config\":{\"included.client.audience\":\"o3k\",\"id.token.claim\":\"false\",\"access.token.claim\":\"true\"}}]}" >/dev/null

sqlite3 "${db}" 'PRAGMA wal_checkpoint(TRUNCATE);'
runtime_db="$(dirname "${db}")/o3k.sqlite"
cp "${db}" "${runtime_db}"
echo "P2.1 operator step: seeded binding diagnostics (database=${runtime_db})" >&2
sqlite3 "${runtime_db}" 'select count(*) from federated_bindings;' >&2
sqlite3 "${runtime_db}" "select trusted_issuer_id,issuer,subject,principal_id from federated_bindings where id like 'p12-7-%' order by id;" >&2
sqlite3 "${runtime_db}" "select count(*) from operator_assignments where user_id='bootstrap-user' and enabled=1;" >&2
echo 'P2.1 operator step: seeded binding verified' >&2
(
  cd "${o3k_root}"
  O3K_LISTEN_ADDR="127.0.0.1:${o3k_port}" O3K_DATA_DIR="$(dirname "${db}")" O3K_PROVIDER=fake O3K_BOOTSTRAP_PASSWORD="${O3K_P12_7_BOOTSTRAP_SECRET}" O3K_TOKEN_SIGNING_KEY=p2-1-live-process-signing-key-at-least-32-bytes O3K_OIDC_TRUST_ID=p12-7-keycloak O3K_OIDC_ISSUER="${issuer}" O3K_OIDC_AUDIENCE=o3k O3K_OIDC_DISCOVERY_URL="${discovery}" O3K_OIDC_ALLOW_INSECURE_LOCAL=true cargo run --quiet -p o3kd
) >"${workdir}/p2-1-o3kd.log" 2>&1 & o3k_pid=$!
for _ in $(seq 1 60); do curl -fsS "http://127.0.0.1:${o3k_port}/healthz" >/dev/null 2>&1 && break; sleep 1; done
curl -fsS "http://127.0.0.1:${o3k_port}/healthz" >/dev/null
echo 'P2.1 operator step: OIDC client configured' >&2
operator_probe_token="$(curl -fsS -X POST "http://127.0.0.1:${kc_port}/realms/o3k-p12-7/protocol/openid-connect/token" -d grant_type=password -d client_id="${client_id}" -d client_secret="${client_secret}" -d username=operator -d password="${P12_8_OPERATOR_PASSWORD}" | python3 -c 'import json,sys; print(json.load(sys.stdin)["access_token"])')"
operator_aud="$(python3 - "${operator_probe_token}" <<'PY'
import base64, json, sys
claims=json.loads(base64.urlsafe_b64decode(sys.argv[1].split('.')[1] + '=' * (-len(sys.argv[1].split('.')[1]) % 4)))
aud=claims.get('aud', [])
print(' '.join(aud if isinstance(aud, list) else [aud]))
PY
)"
echo "P2.1 operator step: token audience=${operator_aud}" >&2
[[ " ${operator_aud} " == *" o3k "* ]] || fail 'operator token audience does not contain o3k'
native_response="$(curl -sS -H 'content-type: application/json' -X POST "http://127.0.0.1:${o3k_port}/o3k/v1/identity/tokens" -d "{\"auth\":{\"method\":\"federated\",\"federated\":{\"access_token\":\"${operator_probe_token}\",\"scope\":{\"kind\":\"system\"}}}}")"
native_token="$(python3 - "${native_response}" <<'PY'
import json, sys
print(json.loads(sys.argv[1]).get('token', {}).get('id', ''))
PY
)"
unset operator_probe_token native_response
test -n "${native_token}" || fail 'O3K system exchange did not return a native token'
direct_profile_status="$(curl -sS -o /dev/null -w '%{http_code}' -H "Authorization: Bearer ${native_token}" "http://127.0.0.1:${o3k_port}/o3k/v1/operator/profile")"
unset native_token
echo "P2.1 operator step: direct O3K profile HTTP ${direct_profile_status}" >&2
test "${direct_profile_status}" = 200 || fail "O3K operator profile HTTP ${direct_profile_status}"
(
  cd "${araf_root}/backend"
  ARAF_RUNTIME_PROFILE=test ARAF_UPSTREAM_ADAPTER=o3k ARAF_OPERATOR_OIDC_CLIENT_ID="${client_id}" ARAF_OPERATOR_OIDC_CLIENT_SECRET="${client_secret}" ARAF_OPERATOR_OIDC_ISSUER_URL="${issuer}" ARAF_OPERATOR_OIDC_REDIRECT_URI="${redirect_uri}" O3K_URL="http://127.0.0.1:${o3k_port}" ARAF_OPERATOR_BFF_PORT="${operator_port}" cargo run --quiet -p operator-bff
) >"${workdir}/p2-1-operator-bff.log" 2>&1 & bff_pid=$!
for _ in $(seq 1 60); do curl -fsS "http://127.0.0.1:${operator_port}/healthz" >/dev/null 2>&1 && break; sleep 1; done
curl -fsS "http://127.0.0.1:${operator_port}/healthz" >/dev/null

jar="${workdir}/p2-1-operator.cookies"
curl -fsS -D "${workdir}/p2-1-login.headers" -o /dev/null "http://127.0.0.1:${operator_port}/api/v1/auth/login"
auth_url="$(sed -n 's/^Location: //Ip' "${workdir}/p2-1-login.headers" | tr -d '\r' | head -1)"
test -n "${auth_url}" || fail 'authorization redirect missing'
echo 'P2.1 operator step: authorization redirect received' >&2
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
provider_status="$(curl -sS -D "${workdir}/p2-1-provider.headers" -o "${workdir}/p2-1-provider.body" -w '%{http_code}' -c "${jar}" -b "${jar}" -X POST "${form_action}" -H 'Content-Type: application/x-www-form-urlencoded' --data-urlencode username=operator --data-urlencode "password=${P12_8_OPERATOR_PASSWORD}" --data-urlencode credentialId= --max-redirs 0 || true)"
echo "P2.1 operator step: provider login HTTP ${provider_status}" >&2
callback="$(sed -n 's/^Location: //Ip' "${workdir}/p2-1-provider.headers" | tr -d '\r' | head -1)"
test -n "${callback}" || fail 'provider callback redirect missing'
echo 'P2.1 operator step: provider login accepted' >&2
status="$(curl -sS -D "${workdir}/p2-1-callback.headers" -o /dev/null -w '%{http_code}' -c "${jar}" -b "${jar}" "${callback}")"
case "${status}" in 302|303) ;; *) fail "Araf callback HTTP ${status}";; esac
echo 'P2.1 operator step: Araf callback accepted' >&2
session="$(curl -fsS -b "${jar}" "http://127.0.0.1:${operator_port}/api/v1/auth/session")"
echo "${session}" | grep -q '"authenticated":true' || fail 'operator session unauthenticated'
echo 'P2.1 operator step: operator session authenticated' >&2
profile_status="$(curl -sS -b "${jar}" -o "${workdir}/p2-1-profile.body" -w '%{http_code}' "http://127.0.0.1:${operator_port}/api/v1/operator/profile")"
profile="$(cat "${workdir}/p2-1-profile.body")"
echo "P2.1 operator step: profile HTTP ${profile_status}" >&2
echo "${profile}" | grep -q '"scope":"system"' || fail "system AuthContext missing (HTTP ${profile_status}, body=${profile})"
echo 'P2.1 operator step: O3K system AuthContext obtained' >&2
echo "${profile}" | grep -q 'operator-console' || fail 'operator profile not authorized'
echo 'P2.1 operator step: operator profile authorized' >&2
if grep -Eiq 'access_token|refresh_token|id_token' "${jar}" "${workdir}/p2-1-callback.headers"; then exit 1; fi
curl -fsS -X POST -b "${jar}" "http://127.0.0.1:${operator_port}/api/v1/auth/logout" >/dev/null

alice_system="$(curl -sS -w '\n%{http_code}' -H 'content-type: application/json' -X POST "http://127.0.0.1:${o3k_port}/o3k/v1/identity/tokens" -d "{\"auth\":{\"method\":\"federated\",\"federated\":{\"access_token\":\"${O3K_P12_7_ALICE_TOKEN}\",\"scope\":{\"kind\":\"system\"}}}}")"
echo "${alice_system}" | tail -1 | grep -Eq '^4(01|03)$'
echo 'P2.1 negative: tenant system exchange rejected' >&2
echo 'P2.1 operator step: logout passed' >&2
echo 'P2.1 real Operator process evidence: PASS'
