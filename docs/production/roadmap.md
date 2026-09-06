# Production maturity roadmap

## Starting state

The architecture MVP M0-M14 is complete. P1.1 productionized the initial shell visual foundation.

O3K P12-IAM is now complete and the original Araf identity blocker is resolved. Real Keycloak -> Araf BFF -> O3K federation, scope selection, tenant resource access and logout evidence passed. The remaining work is Araf product/production convergence rather than a missing upstream IAM contract.

Current important repository findings:

- P1.2 remains open: branch protection names `browser E2E (Chromium)`, but the current Araf CI workflow does not emit a browser job;
- P1.3 is partially configured: `main` protection exists, but final acceptance waits on the actual browser check being produced and verified on a real PR;
- P1.4 remains open: current `BffConfig::from_env()` can silently select `Fixture` when `ARAF_UPSTREAM_ADAPTER` is absent or unrecognized, which is not acceptable for production;
- P2.1 is no longer upstream-blocked, but still requires provider-neutral OIDC discovery, explicit production fail-closed configuration and real Operator BFF process evidence.

## Dependency graph

```text
P1.2 Browser/E2E CI ----+
P1.3 Main protection ---+--> P1 gate
P1.4 Deployment security+
P1.5 Product completeness+
                         |
                         v
P2.1 OIDC/session/AuthContext closure
P2.2 Scope + service/schema discovery
P2.3 Tenant core API closure
P2.4 Operations Center
P2.5 Governance/IAM/quota/audit
P2.6 Operator health/capacity
P2.7 Metering/usage/cost
                         |
                         +--> P2.8 Real O3K production E2E gate
                                  |
                                  v
P3.1 CloudBackend architecture/ADR
P3.2 Keystone
P3.3 Nova + Glance
P3.4 Neutron
P3.5 Cinder
P3.6 OpenStack compatibility Operations
P3.7 Optional Object Storage profile
P3.8 Quota/governance/capabilities
                         |
                         +--> P3.9 Cross-backend parity E2E gate
                                  |
                                  v
P4.1 Observability
P4.2 Performance/scale
P4.3 HA/resilience/session durability
P4.4 Security/supply-chain release gate
P4.5 Packaging/deployment/upgrade compatibility
P4.6 Supportability/runbooks/docs
                         |
                         +--> P4.7 RC, pilot/soak and v1.0 decision
```

## Immediate execution order

1. Close P1.2 by making browser E2E an actual CI-produced required check.
2. Close P1.3 by validating repository protection against the real produced checks.
3. Close P1.4 and P1.5 so production mode cannot silently use fixtures/placeholders.
4. Close P2.1 against the completed P12-IAM baseline; do not reopen O3K IAM without a reproducible regression.
5. Continue P2.2-P2.7, then run the hard P2.8 integrated production gate.
6. Begin OpenStack production claims only through P3 and its P3.9 gate.
7. Earn final product readiness through P4.7.

## Gate rules

- Do not start broad P2 work while production mode can silently use fixtures.
- P12-IAM PASS means Araf should consume the authoritative O3K IAM contract; it does not waive Araf production configuration, generic OIDC-client or real Operator-BFF evidence requirements.
- Do not start P3 service adapters before P3.1 freezes the backend boundary and compatibility-Operation semantics.
- Do not call OpenStack support complete until P3.9 runs the same critical tenant journeys against a real supported OpenStack deployment.
- Do not call Araf production-ready until P4.7 passes against at least one real O3K environment. OpenStack can be a separately supported backend profile, but if advertised as production-supported it must also pass P3.9 and applicable P4 gates.

## Explicitly deferred beyond v1 core

Unless separately promoted into the release scope:

- multi-backend aggregation in one browser session,
- cross-cloud live migration,
- Octavia load balancer UI,
- Designate DNS UI,
- managed Kubernetes/database product-specific UX,
- billing/invoicing/accounting,
- marketplace,
- arbitrary runtime JavaScript plugins,
- VM graphical console gateway,
- AI/autonomous administration.

The architecture must remain compatible with these without making them v1 blockers.