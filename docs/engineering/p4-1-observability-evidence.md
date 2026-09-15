# P4.1 observability and correlation evidence

The BFFs expose a bounded telemetry surface for both supported production
profiles. `GET /metrics` is scrapeable without a browser session and emits
Prometheus text with fixed labels only:

- request counters and latency buckets by Tenant/Operator surface and HTTP
  status class;
- upstream call counters by `o3k`/`openstack` backend and status class;
- authentication failures and rate-limit outcomes by surface; and
- OpenStack compatibility reconciliation outcomes.

Resource IDs, project IDs, operation IDs, request IDs, raw URLs and arbitrary
error text are not metric labels. They remain available through the bounded
request correlation chain and structured Problem Details response.

## Correlation chain

Every request receives or preserves `X-Request-ID` and `X-Correlation-ID`.
The values are echoed in the response headers and Problem Details. Structured
completion events include only method, status, elapsed time, surface and those
safe identifiers. The O3K adapter forwards the operation/request correlation
where the upstream contract provides it; the OpenStack adapter records the
same correlation on its derived CompatibilityOperation journal entry.

The intended diagnostic path is therefore:

```text
browser Problem Details / response headers
  -> BFF request-completed event
  -> backend status counter + adapter request log
  -> O3K Operation or OpenStack CompatibilityOperation
  -> authoritative upstream resource/error response
```

## Health semantics

`GET /healthz` is liveness. `GET /readyz` reports that the selected, validated
upstream adapter boundary is configured; adapter construction fails closed at
startup. Provider-specific health and dependency degradation remain visible
through the authenticated operator diagnostics endpoints and are not inferred
as successful cloud state by readiness.

## Secret handling

The request span and completion event never include request headers or bodies.
Authorization headers, cookies, CSRF tokens, OIDC/O3K/OpenStack credentials,
private keys and upstream payloads therefore cannot enter these telemetry
fields. Regression coverage includes a metrics cardinality test and the
existing security log-redaction contract tests.

## Validation

```text
./tests/prometheus-observability.sh          PASS (Prometheus 2.55.1 pinned digest; scrape series present)
cargo fmt --all -- --check       PASS
cargo check --workspace --all-targets --all-features  PASS
cargo test --workspace --all-features  PASS (56 unit, 62 contract, 15 O3K, 5 OpenStack)
```

This evidence covers fixture, O3K and OpenStack adapter code paths. The exact
candidate's live Prometheus attachment is recorded below; broader release
readiness remains governed by the P4.7 evidence record.

## P4.7 exact-candidate attachment

On 2026-09-15, the digest-pinned `1.0.0-rc.1` Tenant BFF from functional-test
SHA `a8d0d0e` was scraped by the repository's pinned Prometheus image
(`sha256:2659f4c2ebb718e7695cb9b25ffa7d6be64db013daba13e05c875451cf51b0d3`).
Prometheus reached the candidate readiness endpoint and
returned a successful query for
`araf_bff_requests_total{surface="tenant-bff"}` with bounded status-class
labels. The candidate also emitted request and correlation IDs on the live
HTTPS/OIDC/OpenStack path. This closes the production attachment evidence for
the observability contract for that functional candidate; no secret-bearing
labels or payloads were observed. The reviewed head's rebuilt images are
recorded in the P4.7 release evidence; a live Prometheus attachment against
those new frontend digests is still required before exact-head approval.
