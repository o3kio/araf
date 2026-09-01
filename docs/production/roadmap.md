# Production maturity roadmap

## Starting state

The architecture MVP M0-M14 is complete. P1.1 productionized the initial shell visual foundation. Existing open work P1.2-P1.4 remains part of this program.

## Dependency graph

```text
P1.2 Browser/E2E CI ----+
P1.3 Main protection ---+--> P1 gate
P1.4 Deployment security+
P1.5 Product completeness+
                         |
                         v
P2.1 OIDC/session/AuthContext
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

## Gate rules

- Do not start broad P2 work while production mode can silently use fixtures.
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