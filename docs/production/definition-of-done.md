# Definition of Production Ready

Araf is production-ready only when all applicable conditions below are proven.

## Product

- Tenant and Operator consoles have no production-visible placeholder routes.
- Navigation is capability/service driven and only advertises usable functions.
- Loading, empty, degraded, forbidden, not-found and structured-error states are deliberate and consistent.
- Critical journeys are keyboard accessible and meet the WCAG 2.2 AA project target.
- Comfortable and compact density remain usable.

## Security

- HTTPS is mandatory in the production reference topology.
- Tenant and Operator sessions use separate BFF trust surfaces and secure cookies.
- No reusable cloud token is readable by browser JavaScript.
- CSRF, CSP, origin, trusted-proxy and forwarded-header behavior are tested.
- Production mode never silently falls back to fixtures.
- Cross-project and tenant-to-operator negative tests pass.
- No unresolved BLOCKER/HIGH security finding is accepted for shipped functionality.

## O3K backend

- Real OIDC/session/AuthContext works.
- Real project/scope/region and service/resource/schema discovery works.
- Compute, Network, Volume and Image MVP journeys use authoritative O3K APIs.
- All supported mutations resolve through canonical O3K Operations.
- Operations collection/detail is real, bounded and scope safe.
- Supported IAM/governance/quota/audit/operator/metering surfaces are real or explicitly unavailable; no fixture data appears in production.

## OpenStack backend when advertised as supported

- Keystone, Nova, Glance, Neutron and Cinder supported-profile flows work against a real deployment.
- Backend-specific code is behind the backend adapter boundary.
- Araf compatibility Operations are explicitly derived from OpenStack truth and cannot override resource state.
- Unsupported optional services are capability-hidden.
- The common Araf E2E journey suite passes against OpenStack.

## Reliability and operations

- BFFs are observable with metrics, logs, traces/correlation and health/readiness signals.
- Multiple replicas and rolling restart behavior are tested.
- Session durability/failure behavior is documented and proven.
- Upstream outages, timeouts and partial failures have bounded behavior.
- Large resource populations remain server-bounded; browser and BFF resource use is measured.

## Release engineering

- Reproducible OCI images and supported deployment artifacts exist.
- Configuration is versioned/validated and secrets are externalized.
- Upgrade and rollback procedures are tested.
- SBOM, dependency/license/security scanning and artifact provenance/signing are enforced as defined by the release policy.
- Operator, security, backup/recovery and incident runbooks are current.
- A release candidate passes the production acceptance matrix and pilot/soak gate.

## Final verdict

The final production evidence must end in exactly one of:

- **GO — PRODUCTION READY**
- **GO WITH BOUNDED DEVIATIONS**
- **NO-GO — NOT PRODUCTION READY**

A bounded deviation must be explicit, non-security-critical, non-isolation-critical, and outside the advertised supported profile.