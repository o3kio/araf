#!/usr/bin/env bash
set -euo pipefail

# This gate is intentionally token-injected only from the operator's accepted
# authentication harness. It does not mint credentials, invoke handlers, or
# enable the fixture adapter. O3K_URL must point at the real converged o3kd
# process and O3K_TOKEN must be its server-issued system/operator token.
: "${O3K_URL:?O3K_URL must point at the real converged O3K process}"
: "${O3K_TOKEN:?O3K_TOKEN must be a server-issued system/operator token}"

cd "${ARAF_ROOT:-/root/araf}/backend"
O3K_URL="${O3K_URL}" O3K_TOKEN="${O3K_TOKEN}" \
  cargo test -p console-bff-core --test p2_6_real_o3k -- --ignored --nocapture
echo 'Araf #49 real Operator BFF -> O3K diagnostics smoke: PASS'
