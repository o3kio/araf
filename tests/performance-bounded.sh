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
resource_file=$(mktemp)
cleanup() {
  if [[ -n "${pid:-}" ]]; then kill "$pid" 2>/dev/null || true; fi
  rm -f "$log_file" "$latency_file" "$sorted_file" "$resource_file"
}
trap cleanup EXIT

(cd "$root_dir/backend" && cargo build --quiet -p tenant-bff)
(
  cd "$root_dir/backend"
  exec env ARAF_RUNTIME_PROFILE=development ARAF_UPSTREAM_ADAPTER=fixture \
    ARAF_TENANT_BFF_PORT="$port" RUST_LOG=warn \
    target/debug/tenant-bff
) >"$log_file" 2>&1 &
pid=$!
for _ in $(seq 1 300); do
  if curl -fsS "http://127.0.0.1:${port}/readyz" >/dev/null 2>&1; then break; fi
  sleep 0.2
done
curl -fsS "http://127.0.0.1:${port}/readyz" >/dev/null

(
  while kill -0 "$pid" 2>/dev/null; do
    rss_kib=$(awk '/^VmRSS:/ {print $2}' "/proc/${pid}/status" 2>/dev/null || echo 0)
    cpu_seconds=$(awk '{print ($14 + $15) / 100}' "/proc/${pid}/stat" 2>/dev/null || echo 0)
    printf '%s %s\n' "${rss_kib:-0}" "${cpu_seconds:-0}" >>"$resource_file"
    sleep 0.05
  done
) &
sampler_pid=$!
start_ns=$(date +%s%N)
seq "$requests" | xargs -P "$concurrency" -I{} \
  curl -fsS -o /dev/null -w '%{time_total}\n' \
  "http://127.0.0.1:${port}/api/v1/resources/compute.server?page=0&pageSize=100" \
  >"$latency_file"
end_ns=$(date +%s%N)
kill "$sampler_pid" 2>/dev/null || true
wait "$sampler_pid" 2>/dev/null || true

sort -n "$latency_file" >"$sorted_file"
awk '
  { values[NR] = $1 * 1000 }
  END {
    if (NR == 0) exit 1
    p50 = int(NR * .50 + .5); p95 = int(NR * .95 + .5); p99 = int(NR * .99 + .5)
    printf "requests=%d p50_ms=%.2f p95_ms=%.2f p99_ms=%.2f\n", NR, values[p50], values[p95], values[p99]
  }
' "$sorted_file"
elapsed_ns=$((end_ns - start_ns))
awk -v requests="$requests" -v elapsed_ns="$elapsed_ns" '
  BEGIN { max_rss = 0; cpu = 0 }
  { if ($1 > max_rss) max_rss = $1; if ($2 > cpu) cpu = $2 }
  END {
    if (elapsed_ns <= 0) exit 1
    printf "throughput_rps=%.2f peak_rss_mib=%.2f cpu_seconds=%.2f\n", requests / (elapsed_ns / 1000000000), max_rss / 1024, cpu
  }
' "$resource_file"
