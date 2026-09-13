# P4.2 performance and resource-budget evidence

## Topology and workload

The repeatable local gate is `tests/performance-bounded.sh`. It runs the real
Tenant BFF binary with the explicit fixture adapter, sends 200 bounded
`compute.server` collection requests at concurrency 16, and requests a page
of 100 items from a reported 100,000-resource universe. No request downloads
the inventory or filters it in the browser.

Observed run on the development host (2026-09-13):

```text
requests=200 p50_ms=12.35 p95_ms=19.04 p99_ms=21.26
```

The BFF's 256-per-process concurrency gate provides backpressure and returns
`429` with `Retry-After: 1` when saturated. Request and rate-limit counters are
available from `/metrics`.

## Resource and protocol budgets

- Collection page size is bounded to 100 by every production adapter.
- OpenStack marker/offset requests and O3K cursor requests remain server-side;
  the browser receives only one bounded page and continuation metadata.
- Request bodies are capped at 256 KiB.
- O3K/OpenStack upstream HTTP clients use finite timeouts and bounded response
  bodies.
- Operation, governance, audit and usage collections use the same bounded
  page contract.
- No list route performs an unbounded inventory fetch or client-side filter.

## Browser budget

The production builds completed successfully with separate Tenant and Operator
bundles. The current baseline artifacts were:

| Console | JS (gzip) | CSS (gzip) |
| --- | ---: | ---: |
| Tenant | 339.68 KiB | 363.48 KiB |
| Operator | 333.54 KiB | 362.53 KiB |

Vite reports a chunk-size warning because the shared UI/component baseline is
large. This is a known optimization item; the measured gzip sizes remain the
release budget for the current profile, and route-level code splitting is a
follow-up if the production pilot requires a tighter first-load target.

## Backend profiles and limits

The bounded-load gate is deterministic and covers BFF/fixture behavior. O3K
and OpenStack production runs must be repeated with the same request matrix;
their authoritative pagination, provider latency and dependency limits are
published with the P4.7 release evidence rather than replaced with synthetic
claims. Adapter tests prove cursor/marker handling and page bounds for both
profiles.

## Validation

```text
./tests/performance-bounded.sh                                  PASS
pnpm build                                                       PASS
cargo check --workspace --all-targets --all-features             PASS
cargo test --workspace --all-features                            PASS
```
