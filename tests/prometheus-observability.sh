#!/usr/bin/env bash
set -euo pipefail

# Real Prometheus scrape smoke for the bounded BFF telemetry contract. The
# fixture adapter is used only to make the BFF self-contained; Prometheus is a
# pinned OCI image and validates the wire-format/label contract.
root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
work_dir=$(mktemp -d)
container_name="araf-prometheus-$$"
bff_pid=""
cleanup() {
  [[ -n "$bff_pid" ]] && kill "$bff_pid" 2>/dev/null || true
  docker rm -f "$container_name" >/dev/null 2>&1 || true
  rm -rf "$work_dir"
}
trap cleanup EXIT

(
  cd "$root_dir/backend"
  ARAF_RUNTIME_PROFILE=development ARAF_UPSTREAM_ADAPTER=fixture \
    ARAF_TENANT_BFF_PORT=18080 RUST_LOG=warn cargo run --quiet --bin tenant-bff
) >"$work_dir/bff.log" 2>&1 &
bff_pid=$!
for _ in $(seq 1 300); do
  curl -fsS http://127.0.0.1:18080/readyz >/dev/null 2>&1 && break
  sleep .2
done
curl -fsS http://127.0.0.1:18080/readyz >/dev/null
curl -fsS http://127.0.0.1:18080/api/v1/resources/compute.server?page=0\&pageSize=25 >/dev/null

docker run -d --name "$container_name" --network host \
  -v "$root_dir/tests/prometheus-observability.yml:/etc/prometheus/prometheus.yml:ro" \
  prom/prometheus@sha256:2659f4c2ebb718e7695cb9b25ffa7d6be64db013daba13e05c875451cf51b0d3 \
  --config.file=/etc/prometheus/prometheus.yml --web.listen-address=:19099 >/dev/null
for _ in $(seq 1 120); do
  curl -fsS http://127.0.0.1:19099/-/ready >/dev/null 2>&1 && break
  sleep .5
done
curl -fsS http://127.0.0.1:19099/-/ready >/dev/null
query=""
for _ in $(seq 1 60); do
  query=$(curl -fsS --get --data-urlencode 'query=araf_bff_requests_total{surface="tenant-bff"}' \
    http://127.0.0.1:19099/api/v1/query)
  grep -q 'tenant-bff' <<<"$query" && break
  sleep .5
done
grep -q '"status":"success"' <<<"$query"
grep -q 'tenant-bff' <<<"$query"
printf 'prometheus_scrape=pass target=tenant-bff requests_series=present\n'
