# Roadmap issue index

## MVP roadmap — complete

The completed architecture/MVP roadmap is tracked by GitHub epic **#2**.

| Phase | Issue | Purpose |
|---|---:|---|
| EPIC | #2 | Araf MVP — O3K next-generation cloud console |
| M0 | #3 | Repository bootstrap and deterministic quality gates |
| M1 | #4 | Araf design system foundation |
| M2 | #5 | Tenant/Operator shells and explicit scope context |
| M3 | #6 | Rust BFF boundary and deterministic O3K contract harness |
| M4 | #7 | Generic resource list/detail/relationship runtime |
| M5 | #8 | Schema-driven create/edit/action flows |
| M6 | #9 | First-class Operations UX and prototype gate |
| M7 | #10 | Real O3K tenant core-service integration |
| M8 | #11 | Tenant governance, access, quota and audit |
| M9 | #12 | Operator platform, regions and provider health |
| M10 | #13 | ServiceManifest catalog and capability-driven integration |
| M11 | #14 | Usage, quota and bounded cost awareness |
| M12 | #15 | Production authentication and security hardening |
| M13 | #16 | Accessibility, i18n, density and cross-browser closure |
| M14 | #17 | End-to-end MVP acceptance and release candidate |

## Production maturity roadmap — active

The active production program is tracked by GitHub epic **#42**. P1.1 was completed by #34 / PR #35.

| Phase | Issue | Purpose |
|---|---:|---|
| EPIC | #42 | Araf Production Maturity — P1 to P4 |
| P1.2 | #38 | Browser E2E and visual shell regression CI |
| P1.3 | #39 | Protect `main` with required CI/review policy |
| P1.4 | #40 | Production deployment security and fail-closed mode |
| P1.5 | #43 | Production UX completeness / remove placeholders |
| P2.1 | #44 | Production OIDC/BFF session/AuthContext |
| P2.2 | #45 | Real O3K scope/service/resource/schema discovery |
| P2.3 | #46 | Native O3K tenant core API closure |
| P2.4 | #47 | Canonical O3K Operations Center |
| P2.5 | #48 | Real governance/IAM/quota/audit |
| P2.6 | #49 | Real Operator health/regions/capacity |
| P2.7 | #50 | Real metering/usage/cost |
| P2.8 | #51 | Native O3K production E2E gate |
| P3.1 | #52 | CloudBackend architecture and ADR |
| P3.2 | #53 | Keystone identity/scope/service catalog |
| P3.3 | #54 | Nova Compute + Glance Image adapter |
| P3.4 | #55 | Neutron Networking adapter |
| P3.5 | #56 | Cinder Block Storage adapter |
| P3.6 | #57 | OpenStack CompatibilityOperations/reconciliation |
| P3.7 | #58 | Optional OpenStack Object Storage profile |
| P3.8 | #59 | OpenStack governance/quotas/capability mapping |
| P3.9 | #60 | OpenStack production support-profile E2E gate |
| P4.1 | #61 | Production observability and correlation |
| P4.2 | #62 | Performance, scale and resource budgets |
| P4.3 | #63 | HA, resilience and session durability |
| P4.4 | #64 | Security and software-supply-chain release gate |
| P4.5 | #65 | Packaging/deployment/config/upgrade/rollback |
| P4.6 | #66 | Supportability, docs and incident runbooks |
| P4.7 | #67 | RC, pilot/soak and v1.0 production decision |

Production design is under `docs/production/`; one implementation prompt per phase is under `prompts/production/`.

The **live GitHub issues** are the tracking source of truth for status, dependency discussion and implementation evidence. The **versioned prompt files** are the implementation instructions.

Do not create duplicate roadmap issues. The legacy `scripts/create-issues.sh` only owns the M0-M14 roadmap and must not be used to recreate the production backlog.