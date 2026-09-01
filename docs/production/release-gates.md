# Production release gates

## P1 — Product Foundation

Exit only when browser E2E is a required signal, normal changes cannot bypass protected-main policy, production HTTPS/session/CSP/fail-closed behavior is documented and tested, and production navigation contains no unfinished advertised features.

## P2 — Native O3K Production Convergence

Exit only when a real O3K environment can complete the defined Tenant and Operator acceptance journeys with production identity and **zero silent fixture fallback**. Missing optional O3K capabilities must be surfaced as unavailable rather than emulated as authoritative Araf state.

## P3 — OpenStack Compatibility

Exit only when the accepted OpenStack support profile completes the common tenant journeys through the same Araf resource UX. OpenStack service APIs may differ internally, but React code must not become service/backend conditional. Compatibility Operations must be derived/reconciled from OpenStack state and clearly separated from canonical O3K Operations internally.

## P4 — Industrial Production Release

Exit only when observability, performance/scale, HA/resilience, release security, deployment/upgrade, supportability and pilot/soak evidence are complete.

## Merge discipline

Each phase is one focused issue/PR unless the issue explicitly allows sub-PRs. A phase must publish exact validation evidence. Architecture/security changes require explicit review and ADR updates where applicable.