#!/usr/bin/env bash
set -Eeuo pipefail

# P3.9 is a real-environment gate. The deployment harness owns the supported
# OpenStack cloud, Keystone/IdP, TLS ingress, and production Araf processes.
# This wrapper refuses fixture/demo endpoints and records only redacted state.

araf_root="${ARAF_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
evidence_dir="${ARAF_P3_9_EVIDENCE_DIR:-${araf_root}/target/p3-9-openstack-gate}"
result_file="${evidence_dir}/result.env"
harness_result="${evidence_dir}/harness.success"
harness="${ARAF_P3_9_HARNESS:-}"
profile="${ARAF_P3_9_PROFILE:-}"
tenant_url="${ARAF_P3_9_TENANT_URL:-}"
operator_url="${ARAF_P3_9_OPERATOR_URL:-}"
openstack_auth_url="${ARAF_P3_9_OPENSTACK_AUTH_URL:-}"
idp_discovery="${ARAF_P3_9_OIDC_DISCOVERY_URL:-}"

mkdir -p "${evidence_dir}"
chmod 700 "${evidence_dir}"

write_result() {
  local verdict="$1" reason="$2"
  {
    printf 'verdict=%q\n' "${verdict}"
    printf 'reason=%q\n' "${reason}"
    printf 'araf_commit=%q\n' "$(git -C "${araf_root}" rev-parse HEAD 2>/dev/null || echo unknown)"
    printf 'profile=%q\n' "${profile:-unset}"
  } >"${result_file}"
}

fail_gate() {
  local reason="$1"
  write_result 'NO-GO' "${reason}"
  echo 'NO-GO'
  echo "P3.9 OpenStack production gate: ${reason}" >&2
  exit 2
}

[[ -x "${harness}" ]] || fail_gate 'ARAF_P3_9_HARNESS must name an executable real-environment harness'
[[ -n "${profile}" && "${profile}" != 'fixture' && "${profile}" != 'fake' ]] \
  || fail_gate 'a real supported OpenStack profile must be named'

command -v python3 >/dev/null 2>&1 \
  || fail_gate 'python3 is required for strict endpoint validation'

for endpoint in "${tenant_url}" "${operator_url}" "${openstack_auth_url}" "${idp_discovery}"; do
  python3 - "${endpoint}" <<'PY' || fail_gate 'endpoints must be absolute HTTPS URLs without credentials, query, or fragment'
import sys
from urllib.parse import urlparse

value = urlparse(sys.argv[1])
if value.scheme != "https" or not value.hostname or value.username or value.password:
    raise SystemExit(1)
if value.query or value.fragment:
    raise SystemExit(1)
PY
done

if ! env -i PATH="${PATH}" \
  ARAF_P3_9_PROFILE="${profile}" \
  ARAF_P3_9_TENANT_URL="${tenant_url}" \
  ARAF_P3_9_OPERATOR_URL="${operator_url}" \
  ARAF_P3_9_OPENSTACK_AUTH_URL="${openstack_auth_url}" \
  ARAF_P3_9_OIDC_DISCOVERY_URL="${idp_discovery}" \
  ARAF_P3_9_EVIDENCE_DIR="${evidence_dir}" \
  "${harness}"; then
  write_result 'NO-GO' 'real-environment harness failed; inspect its redacted artifacts'
  echo 'NO-GO'
  echo 'P3.9 OpenStack production gate: real-environment harness failed' >&2
  exit 1
fi

[[ -f "${harness_result}" && ! -L "${harness_result}" ]] \
  || fail_gate 'real-environment harness must create the redacted harness.success marker'
grep -Fqx 'ARAF_P3_9_HARNESS_PASS=1' "${harness_result}" \
  || fail_gate 'real-environment harness success marker is malformed'

write_result 'GO' 'all real OpenStack profile assertions passed'
echo 'GO — OPENSTACK PROFILE SUPPORTED'
