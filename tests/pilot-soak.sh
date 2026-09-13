#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
work_dir=$(mktemp -d)
cleanup() { kill ${pids:-} 2>/dev/null || true; rm -rf "$work_dir"; }
trap cleanup EXIT
tenant_ports=(18081 18082)
operator_ports=(18083 18084)
pids=""
start_replica() {
  local surface=$1 binary=$2 port=$3
  local port_var=ARAF_TENANT_BFF_PORT
  [[ "$surface" == "operator" ]] && port_var=ARAF_OPERATOR_BFF_PORT
  (
    cd "$root_dir/backend"
    env ARAF_RUNTIME_PROFILE=development ARAF_UPSTREAM_ADAPTER=fixture \
      "$port_var=$port" RUST_LOG=warn cargo run --quiet --bin "$binary"
  ) >"$work_dir/$surface-$port.log" 2>&1 &
  [[ "$surface" == "tenant" && "$port" == "18081" ]] && tenant_pid=$!
  [[ "$surface" == "operator" && "$port" == "18083" ]] && operator_pid=$!
  pids+=" $!"
}
tenant_pid=""
operator_pid=""
for port in "${tenant_ports[@]}"; do start_replica tenant tenant-bff "$port"; done
for port in "${operator_ports[@]}"; do start_replica operator operator-bff "$port"; done
for port in "${tenant_ports[@]}" "${operator_ports[@]}"; do
  for _ in $(seq 1 300); do curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null 2>&1 && break; sleep .2; done
  curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null
done
requests=0
started_at=$(date +%s)
for cycle in $(seq 1 100); do
  for port in "${tenant_ports[@]}" "${operator_ports[@]}"; do
    curl -fsS "http://127.0.0.1:$port/api/v1/resources/compute.server?page=0&pageSize=25" >/dev/null
    curl -fsS "http://127.0.0.1:$port/metrics" >/dev/null
    requests=$((requests + 2))
  done
done
# Abruptly lose one replica of each surface, restart both, and verify recovery.
kill "$tenant_pid" "$operator_pid" 2>/dev/null || true
sleep .2
(
  cd "$root_dir/backend"
  ARAF_RUNTIME_PROFILE=development ARAF_UPSTREAM_ADAPTER=fixture \
    ARAF_TENANT_BFF_PORT=18081 RUST_LOG=warn cargo run --quiet --bin tenant-bff
) >"$work_dir/restarted.log" 2>&1 &
restarted_tenant=$!
(
  cd "$root_dir/backend"
  ARAF_RUNTIME_PROFILE=development ARAF_UPSTREAM_ADAPTER=fixture \
    ARAF_OPERATOR_BFF_PORT=18083 RUST_LOG=warn cargo run --quiet --bin operator-bff
) >"$work_dir/restarted-operator.log" 2>&1 &
restarted_operator=$!
pids+=" $restarted_tenant $restarted_operator"
for _ in $(seq 1 300); do curl -fsS http://127.0.0.1:18081/readyz >/dev/null 2>&1 && break; sleep .2; done
curl -fsS http://127.0.0.1:18081/api/v1/resources/compute.server?page=0\&pageSize=25 >/dev/null
for _ in $(seq 1 300); do curl -fsS http://127.0.0.1:18083/readyz >/dev/null 2>&1 && break; sleep .2; done
curl -fsS http://127.0.0.1:18083/api/v1/resources/compute.server?page=0\&pageSize=25 >/dev/null
finished_at=$(date +%s)
printf 'duration_seconds=%s request_count=%s tenant_replicas=2 operator_replicas=2 restart_recovery=pass\n' "$((finished_at - started_at))" "$requests"
