# Observability and correlation

## Signals

- `GET /healthz`: process liveness only.
- `GET /readyz`: startup configuration and selected adapter boundary are
  constructed. It does not certify provider capacity or mutation success.
- `GET /version`: version, git SHA (when set), surface and backend metadata.
- `GET /metrics`: Prometheus text with bounded labels: surface, backend,
  status class and fixed outcome dimensions.
- Structured BFF logs: method, status, elapsed time, surface,
  `request_id` and `correlation_id`; no request bodies, cookies, Authorization
  headers, tokens or provider payloads.

Tracing is not a separate required runtime dependency. The correlation IDs are
the portable trace key and are echoed as `X-Request-ID` and
`X-Correlation-ID` response headers and in Problem Details.

## Diagnostic chain

```text
browser response header / Problem Details
  -> BFF request-completed log and status/latency metric
  -> adapter upstream-call log/counter
  -> O3K canonical Operation OR OpenStack CompatibilityOperation
  -> authoritative provider resource/error read
```

Collect the surface, release version/SHA, UTC timestamp, request and
correlation IDs, project/region (if policy permits), resource ID, operation ID,
HTTP status and structured error code. Do not collect cookies, Authorization
headers, request bodies or secret-bearing environment dumps.

## Scrape example

```bash
kubectl -n araf port-forward svc/<release>-tenant-bff 18080:80
curl --fail http://127.0.0.1:18080/metrics > tenant.metrics.txt
grep 'araf_bff_requests_total' tenant.metrics.txt
```

Do not scrape `/metrics` from the browser-facing console URL: the static
frontend does not proxy that path. For Compose, use
`http://127.0.0.1:8080/metrics` or `:8081/metrics` on the private BFF listener.

Use the repository's pinned smoke check when validating a deployment:
`./tests/prometheus-observability.sh` (requires Docker and Prometheus access).
The script is a test command, not a production-side secret collector.
