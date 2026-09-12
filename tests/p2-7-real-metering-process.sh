#!/usr/bin/env bash
set -euo pipefail
araf_root="${ARAF_ROOT:-/root/araf}"
o3k_root="${O3K_ROOT:-/root/o3k-rust}"
o3k_port="${P2_7_O3K_PORT:-19093}"
workdir="${P2_7_WORKDIR:-$(mktemp -d)}"
mkdir -p "${workdir}"
o3k_pid=""
cleanup() { if [[ -n "${o3k_pid}" ]]; then kill "${o3k_pid}" 2>/dev/null || true; wait "${o3k_pid}" 2>/dev/null || true; fi; }
trap cleanup EXIT
(
  cd "${o3k_root}"
  O3K_LISTEN_ADDR="127.0.0.1:${o3k_port}" O3K_DATA_DIR="${workdir}" O3K_PROVIDER=fake \
  O3K_BOOTSTRAP_PASSWORD=p2-7-bootstrap-password \
  O3K_TOKEN_SIGNING_KEY=p2-7-real-process-signing-key-at-least-32-bytes \
  O3K_NATIVE_CURSOR_HMAC_KEY=p2-7-real-native-cursor-key-at-least-32-bytes cargo run --quiet -p o3kd
) >"${workdir}/o3kd.log" 2>&1 &
o3k_pid=$!
for _ in $(seq 1 60); do curl -fsS "http://127.0.0.1:${o3k_port}/healthz" >/dev/null 2>&1 && break; sleep 1; done
headers="${workdir}/token.headers"
curl -fsS -D "${headers}" -H 'content-type: application/json' -X POST "http://127.0.0.1:${o3k_port}/v3/auth/tokens" \
  -d '{"auth":{"identity":{"methods":["password"],"password":{"user":{"name":"admin","password":"p2-7-bootstrap-password"}}},"scope":{"project":{"name":"admin"}}}}' -o /dev/null
token="$(awk 'tolower($1)=="x-subject-token:"{print $2}' "${headers}" | tr -d '\r')"
test -n "${token}"
cd "${araf_root}/backend"
O3K_URL="http://127.0.0.1:${o3k_port}" O3K_TOKEN="${token}" cargo test -p console-bff-core --test p2_7_real_o3k -- --ignored --nocapture
echo 'Araf #50 real metering process smoke: PASS'
