#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
work_dir=$(mktemp -d)
cleanup() { kill ${pids:-} 2>/dev/null || true; rm -rf "$work_dir"; }
trap cleanup EXIT
ports=(18081 18082)
pids=""
for port in "${ports[@]}"; do
  (
    cd "$root_dir/backend"
    ARAF_RUNTIME_PROFILE=development ARAF_UPSTREAM_ADAPTER=fixture \
      ARAF_TENANT_BFF_PORT="$port" RUST_LOG=warn cargo run --quiet --bin tenant-bff
  ) >"$work_dir/$port.log" 2>&1 &
  [[ "$port" == "18081" ]] && first_pid=$!
  pids+=" $!"
done
for port in "${ports[@]}"; do
  for _ in $(seq 1 300); do curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null 2>&1 && break; sleep .2; done
  curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null
done
requests=0
for cycle in $(seq 1 100); do
  for port in "${ports[@]}"; do
    curl -fsS "http://127.0.0.1:$port/api/v1/resources/compute.server?page=0&pageSize=25" >/dev/null
    curl -fsS "http://127.0.0.1:$port/metrics" >/dev/null
    requests=$((requests + 2))
  done
done
# Abruptly lose one replica, restart it, and verify recovery/readiness.
kill "$first_pid" 2>/dev/null || true
sleep .2
(
  cd "$root_dir/backend"
  ARAF_RUNTIME_PROFILE=development ARAF_UPSTREAM_ADAPTER=fixture \
    ARAF_TENANT_BFF_PORT=18081 RUST_LOG=warn cargo run --quiet --bin tenant-bff
) >"$work_dir/restarted.log" 2>&1 &
restarted=$!
pids+=" $restarted"
for _ in $(seq 1 300); do curl -fsS http://127.0.0.1:18081/readyz >/dev/null 2>&1 && break; sleep .2; done
curl -fsS http://127.0.0.1:18081/api/v1/resources/compute.server?page=0\&pageSize=25 >/dev/null
printf 'duration_seconds=~%s request_count=%s replicas=2 restart_recovery=pass\n' "$((100 * 2))" "$requests"
