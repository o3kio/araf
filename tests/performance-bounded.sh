#!/usr/bin/env bash
set -euo pipefail

# Small, repeatable bounded-load gate for the fixture adapter. The fixture
# reports a 100,000-resource universe but every response remains one page.
# Production O3K/OpenStack runs use the same request shape and are recorded in
# the release evidence with their real endpoints.
root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
port=${ARAF_PERF_PORT:-18080}
requests=${ARAF_PERF_REQUESTS:-200}
concurrency=${ARAF_PERF_CONCURRENCY:-16}
log_file=$(mktemp)
latency_file=$(mktemp)
sorted_file=$(mktemp)
cleanup() {
  if [[ -n "${pid:-}" ]]; then kill "$pid" 2>/dev/null || true; fi
  rm -f "$log_file" "$latency_file" "$sorted_file"
}
trap cleanup EXIT

(
  cd "$root_dir/backend"
  ARAF_RUNTIME_PROFILE=development ARAF_UPSTREAM_ADAPTER=fixture \
    ARAF_TENANT_BFF_PORT="$port" RUST_LOG=warn \
    cargo run --quiet --bin tenant-bff
) >"$log_file" 2>&1 &
pid=$!
for _ in $(seq 1 300); do
  if curl -fsS "http://127.0.0.1:${port}/readyz" >/dev/null 2>&1; then break; fi
  sleep 0.2
done
curl -fsS "http://127.0.0.1:${port}/readyz" >/dev/null

seq "$requests" | xargs -P "$concurrency" -I{} \
  curl -fsS -o /dev/null -w '%{time_total}\n' \
  "http://127.0.0.1:${port}/api/v1/resources/compute.server?page=0&pageSize=100" \
  >"$latency_file"

sort -n "$latency_file" >"$sorted_file"
awk '
  { values[NR] = $1 * 1000 }
  END {
    if (NR == 0) exit 1
    p50 = int(NR * .50 + .5); p95 = int(NR * .95 + .5); p99 = int(NR * .99 + .5)
    printf "requests=%d p50_ms=%.2f p95_ms=%.2f p99_ms=%.2f\n", NR, values[p50], values[p95], values[p99]
  }
' "$sorted_file"
