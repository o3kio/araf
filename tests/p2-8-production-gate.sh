#!/usr/bin/env bash
set -Eeuo pipefail

# P2.8 is an integration gate, not a fixture smoke test.  The harness is
# supplied by the deployment environment because it must own the real O3K,
# provider, IdP, TLS ingress, and both BFF processes.  This wrapper performs
# fail-closed preflight and records a redacted result; it deliberately refuses
# to treat the fake provider or a fixture adapter as production evidence.

araf_root="${ARAF_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
evidence_dir="${ARAF_P2_8_EVIDENCE_DIR:-${araf_root}/target/p2-8-production-gate}"
result_file="${evidence_dir}/result.env"
harness="${ARAF_P2_8_HARNESS:-}"
provider="${ARAF_P2_8_PROVIDER:-}"
o3k_url="${ARAF_P2_8_O3K_URL:-}"
tenant_url="${ARAF_P2_8_TENANT_URL:-}"
operator_url="${ARAF_P2_8_OPERATOR_URL:-}"
idp_discovery="${ARAF_P2_8_OIDC_DISCOVERY_URL:-}"

mkdir -p "${evidence_dir}"
chmod 700 "${evidence_dir}"

write_result() {
  local verdict="$1" reason="$2"
  {
    printf 'verdict=%q\n' "${verdict}"
    printf 'reason=%q\n' "${reason}"
    printf 'araf_commit=%q\n' "$(git -C "${araf_root}" rev-parse HEAD 2>/dev/null || echo unknown)"
    printf 'o3k_convergence_commit=%q\n' '21fe687c387a04f107b6e87fac04060b1c28e449'
    printf 'provider=%q\n' "${provider:-unset}"
  } >"${result_file}"
}

fail_gate() {
  local reason="$1"
  write_result 'NO-GO' "${reason}"
  echo "P2.8 production gate: NO-GO — ${reason}" >&2
  exit 2
}

[[ -x "${harness}" ]] || fail_gate 'ARAF_P2_8_HARNESS must name an executable real-environment harness'
[[ "${provider}" != '' && "${provider}" != 'fake' && "${provider}" != 'fixture' ]] \
  || fail_gate 'a real supported provider must be named; fake/fixture profiles are not production evidence'

command -v python3 >/dev/null 2>&1 \
  || fail_gate 'python3 is required for strict endpoint validation'

for endpoint in "${o3k_url}" "${tenant_url}" "${operator_url}" "${idp_discovery}"; do
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
  ARAF_P2_8_PROVIDER="${provider}" \
  ARAF_P2_8_O3K_URL="${o3k_url}" \
  ARAF_P2_8_TENANT_URL="${tenant_url}" \
  ARAF_P2_8_OPERATOR_URL="${operator_url}" \
  ARAF_P2_8_OIDC_DISCOVERY_URL="${idp_discovery}" \
  ARAF_P2_8_EVIDENCE_DIR="${evidence_dir}" \
  "${harness}"; then
  write_result 'NO-GO' 'real-environment harness failed; inspect its redacted artifacts'
  echo 'P2.8 production gate: NO-GO — real-environment harness failed' >&2
  exit 1
fi

write_result 'GO' 'all real-environment assertions passed'
echo "P2.8 production gate: GO — evidence in ${result_file}"
