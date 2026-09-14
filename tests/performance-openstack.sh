#!/usr/bin/env bash
set -Eeuo pipefail

# Bounded load gate for a real OpenStack-backed Tenant BFF. Credentials and
# the process identifier are supplied by the disposable deployment; neither
# is committed or printed.
: "${ARAF_PERF_BASE:?set ARAF_PERF_BASE to the HTTPS Tenant BFF URL}"
: "${ARAF_PERF_CA:?set ARAF_PERF_CA to the deployment CA bundle}"
: "${ARAF_PERF_USERNAME:?set ARAF_PERF_USERNAME to a test IdP user}"
: "${ARAF_PERF_PASSWORD_FILE:?set ARAF_PERF_PASSWORD_FILE to a protected password file}"
: "${ARAF_PERF_BFF_PID:?set ARAF_PERF_BFF_PID to the Tenant BFF PID}"

requests=${ARAF_PERF_REQUESTS:-200}
concurrency=${ARAF_PERF_CONCURRENCY:-16}
resource_type=${ARAF_PERF_RESOURCE_TYPE:-network.network}
page_size=${ARAF_PERF_PAGE_SIZE:-100}
(( requests > 0 && concurrency > 0 && page_size > 0 && page_size <= 100 ))
case "$ARAF_PERF_BASE" in
  https://*) ;;
  *) echo 'ARAF_PERF_BASE must use HTTPS' >&2; exit 2 ;;
esac
work=$(mktemp -d)
cookie="$work/cookies"
headers="$work/headers"
html="$work/login.html"
latencies="$work/latencies"
sorted="$work/sorted"
samples="$work/samples"
trap 'rm -rf "$work"' EXIT

curl_json() {
  curl --silent --show-error --fail --connect-timeout 5 --max-time 60 \
    --cacert "$ARAF_PERF_CA" "$@"
}

curl_json -c "$cookie" -D "$headers" \
  "$ARAF_PERF_BASE/api/v1/auth/login" -o /dev/null
location=$(sed -n 's/^location: //Ip' "$headers" | tr -d '\r')
test -n "$location"
curl_json -b "$cookie" -c "$cookie" "$location" -o "$html"
action=$(rg -o '<form id="kc-form-login"[^>]+action="[^"]+' "$html" \
  | sed 's/.*action="//' | sed 's/&amp;/\&/g')
test -n "$action"
password=$(<"$ARAF_PERF_PASSWORD_FILE")
curl --silent --show-error --connect-timeout 5 --max-time 60 \
  --cacert "$ARAF_PERF_CA" -L -b "$cookie" -c "$cookie" \
  --data-urlencode "username=$ARAF_PERF_USERNAME" \
  --data-urlencode "password=$password" --data-urlencode credentialId= \
  --data-urlencode login=Login "$action" -o /dev/null

context=$(curl_json -b "$cookie" "$ARAF_PERF_BASE/api/v1/context")
test "$(jq '.capabilities | length' <<<"$context")" -gt 0
csrf=$(awk '$6=="araf_csrf" {print $7}' "$cookie")
project=$(jq -r '.projectId' <<<"$context")
curl_json -X POST -H 'Content-Type: application/json' \
  -H "X-CSRF-Token: $csrf" -b "$cookie" \
  --data "{\"project_id\":\"$project\"}" \
  "$ARAF_PERF_BASE/api/v1/auth/scope" >/dev/null
sample=$(curl_json -b "$cookie" \
  "$ARAF_PERF_BASE/api/v1/resources/$resource_type?page=0&pageSize=$page_size")
jq -e --argjson limit "$page_size" \
  '(.items | type == "array") and (.items | length <= $limit)' <<<"$sample" >/dev/null

while kill -0 "$ARAF_PERF_BFF_PID" 2>/dev/null; do
  rss_kib=$(awk '/^VmRSS:/ {print $2}' \
    "/proc/$ARAF_PERF_BFF_PID/status" 2>/dev/null || echo 0)
  cpu_seconds=$(awk '{print ($14 + $15) / 100}' \
    "/proc/$ARAF_PERF_BFF_PID/stat" 2>/dev/null || echo 0)
  printf '%s %s\n' "${rss_kib:-0}" "${cpu_seconds:-0}" >>"$samples"
  sleep 0.05
done &
sampler=$!
kill -0 "$ARAF_PERF_BFF_PID" 2>/dev/null

start_ns=$(date +%s%N)
seq "$requests" | xargs -P "$concurrency" -I{} \
  curl --silent --show-error --fail --connect-timeout 5 --max-time 60 \
    --cacert "$ARAF_PERF_CA" -b "$cookie" -o /dev/null \
    -w '%{http_code} %{time_total}\n' \
    "$ARAF_PERF_BASE/api/v1/resources/$resource_type?page=0&pageSize=$page_size" \
  >"$latencies"
end_ns=$(date +%s%N)
kill "$sampler" 2>/dev/null || true
wait "$sampler" 2>/dev/null || true
test -s "$samples"

awk '$1==200 {print $2}' "$latencies" | sort -n >"$sorted"
test "$(wc -l <"$sorted")" -eq "$requests"
awk '{ values[NR] = $1 * 1000 }
  END {
    p50=int(NR*.50+.5); p95=int(NR*.95+.5); p99=int(NR*.99+.5)
    printf "requests=%d p50_ms=%.2f p95_ms=%.2f p99_ms=%.2f\n", NR, values[p50], values[p95], values[p99]
  }' "$sorted"
elapsed_ns=$((end_ns - start_ns))
awk -v requests="$requests" -v elapsed_ns="$elapsed_ns" '
  BEGIN { max_rss=0; min_cpu=-1; max_cpu=0 }
  { if ($1>max_rss) max_rss=$1; if (min_cpu<0 || $2<min_cpu) min_cpu=$2; if ($2>max_cpu) max_cpu=$2 }
  END {
    printf "throughput_rps=%.2f peak_rss_mib=%.2f cpu_seconds=%.2f\n",
      requests/(elapsed_ns/1000000000), max_rss/1024, max_cpu-min_cpu
  }' "$samples"
printf 'profile=openstack resource_type=%s page_size=%s concurrency=%s\n' \
  "$resource_type" "$page_size" "$concurrency"
