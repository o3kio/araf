# P4.2 performance and resource-budget evidence

## Topology and workload

The repeatable local gate is `tests/performance-bounded.sh`. It runs the real
Tenant BFF binary with the explicit fixture adapter, sends 200 bounded
`compute.server` collection requests at concurrency 16, and requests a page
of 100 items from a reported 100,000-resource universe. No request downloads
the inventory or filters it in the browser.

Observed run on the development host (2026-09-13):

```text
requests=200 p50_ms=12.41 p95_ms=19.16 p99_ms=20.93
throughput_rps=563.89 peak_rss_mib=28.44 cpu_seconds=1.67
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

## Current-head O3K diagnostic

The current-head production-profile Tenant BFF was exercised against a fresh
HTTPS O3K TestLab (`agent` provider) with 200 bounded network collection reads
at concurrency 16. All 200 responses were HTTP 200. The run measured p50
1816.75 ms, p95 2338.71 ms, p99 2390.50 ms, throughput 8.59 requests/s,
0.55 BFF CPU seconds, and 25.77 -> 26.97 MiB RSS. A serial control run was
approximately 54 ms p50; the concurrent tail is consistent with the disposable
agent lab's serialized upstream path and is not a production OpenStack budget
claim. It is retained as a real upstream-load observation; supported
OpenStack load and an agreed O3K production latency budget remain release gates.

## Current OpenStack 2025.1 diagnostic

The production-profile Tenant BFF was also exercised against the disposable
Kolla-Ansible **2025.1** all-in-one deployment described in
`openstack-production-evidence.md`. A protected IdP password file was supplied
to `tests/performance-openstack.sh`; no token, password, cookie or resource
payload was written to the evidence. The test performed 100 authenticated
`network.network` page reads (`pageSize=100`) at concurrency 8 through the
HTTPS ingress and sampled the real BFF process (PID 659990):

```text
requests=100 p50_ms=1130.17 p95_ms=1351.19 p99_ms=1432.28
throughput_rps=6.94 peak_rss_mib=30.80 cpu_seconds=0.28
profile=openstack resource_type=network.network page_size=100 concurrency=8
```

Every response was HTTP 200 and remained one server-side bounded page. The
tail and throughput reflect the single-host disposable OpenStack control-plane
limits; they are diagnostic observations, not a claim of a multi-host or
customer-scale service-level objective. Repeat the same test on each
production topology before setting a release latency budget.

## Validation

```text
./tests/performance-bounded.sh                                  PASS (p50 12.41 ms, p95 19.16 ms, p99 20.93 ms; 563.89 rps, peak RSS 28.44 MiB, CPU 1.67 s)
tests/performance-openstack.sh                                  PASS (100 requests, p50 1130.17 ms, p95 1351.19 ms, p99 1432.28 ms; 6.94 rps, peak RSS 30.80 MiB, CPU 0.28 s; Kolla 2025.1)
./tests/pilot-soak.sh                                           PASS (800 requests, 2 Tenant + 2 Operator replicas)
pnpm build                                                       PASS
pnpm typecheck                                                   PASS
pnpm lint                                                        PASS
pnpm format:check                                                PASS
cargo check --workspace --all-targets --all-features             PASS
cargo clippy --workspace --all-targets --all-features -- -D warnings PASS
cargo test --workspace --all-features                            PASS
```
