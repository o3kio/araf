# Araf production maturity program

Araf M0-M14 proved the architecture and produced a credible MVP. This program owns the work required to turn that MVP into a production-grade cloud console.

The program has four release gates:

1. **P1 — Product Foundation**: browser-level CI, repository governance, production deployment/security, and removal of unfinished product surfaces.
2. **P2 — Native O3K Production Convergence**: real identity, scope, service discovery, tenant/operator resources, Operations, governance and metering with no production fixture fallback.
3. **P3 — OpenStack Backend Compatibility**: a first-class backend adapter that maps a supported OpenStack profile into the same Araf resource UX without leaking OpenStack service concepts into the React application.
4. **P4 — Industrial Production Release**: observability, scale, HA/resilience, release security, deployment/upgrade engineering, supportability and final pilot/soak evidence.

## Non-negotiable product boundaries

- O3K remains authoritative for O3K cloud semantics.
- OpenStack remains authoritative for OpenStack resource state; Araf compatibility state may correlate requests but never becomes resource truth.
- Tenant and Operator consoles remain separate trust surfaces.
- Browser-held reusable cloud tokens are forbidden.
- Production mode fails closed; fixtures are development/test only.
- Backend-specific code stays behind a backend boundary; the React generic resource runtime does not branch on `openstack` versus `o3k`.
- Unsupported backend capabilities are hidden or reported unavailable, never faked.
- A `202` or accepted OpenStack request is not success.
- `v1.0` is earned by evidence, not by issue count.

See `roadmap.md`, `definition-of-done.md`, `release-gates.md`, `openstack-support.md`, and `../architecture/backend-abstraction.md`.