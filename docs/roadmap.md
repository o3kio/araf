# Araf prototype and MVP roadmap

## 1. Delivery philosophy

Build architectural risk first, feature breadth later.

The highest-risk assumptions are:

1. two secure console surfaces can share one maintainable platform,
2. a generic resource runtime can produce genuinely good cloud UX,
3. schema-driven mutations can remain usable without service-specific forms,
4. canonical O3K Operations can drive truthful async UX,
5. the BFF can preserve O3K authority without becoming a shadow control plane,
6. upstream O3K contracts can support scalable resource browsing and capability-aware actions.

The prototype is designed to invalidate bad assumptions cheaply.

## 2. Dependency graph

```text
M0 Bootstrap
 |
 +--> M1 UI foundation --------+
 |                             |
 +--> M3 BFF/contract harness -+--> M4 Generic resource runtime
                               |          |
M1 --> M2 Shells/context ------+          +--> M5 Schema actions
                                          |
                                          +--> M6 Operations

                         ===== PROTOTYPE GATE =====

M3/M4/M5/M6 --> M7 Tenant core integration
M2/M3/M4/M6 --> M8 Tenant governance
M2/M3/M4/M6 --> M9 Operator platform
M4/M5       --> M10 Service catalog/runtime
M7/M8/M9   --> M11 Usage/quota/cost
M3..M11    --> M12 Security hardening
M1..M11    --> M13 Accessibility/i18n/density
M7..M13    --> M14 MVP acceptance/release
```

M12/M13 are not meant to be postponed conceptually until the end. Their issues own final evidence, while security/accessibility requirements are implemented continuously in preceding work.

## 3. Prototype milestone: M0-M6

### Outcome
A convincing architecture prototype using deterministic fixture contracts.

### Demo

Tenant:
- scope switch,
- generic list/detail,
- schema-generated create,
- accepted request creates canonical Operation,
- operation progress/completion,
- resource relationship view.

Operator:
- platform/region/provider fixture health,
- project lookup,
- failed operation investigation.

### Exit decision
Proceed to MVP only if the generic runtime is at least as usable as hand-written resource pages for the representative flows. If it is not, refine the descriptor/presentation boundary before adding services.

## 4. MVP milestone: M7-M14

### Outcome
Deployable Araf console backed by real supported O3K native contracts for a bounded representative cloud-service set.

The MVP must prove product credibility, not service completeness.

### Core tenant service priority
1. Compute Server
2. Network
3. Volume
4. Image/read-only image selection

Additional service implementations wait until the generic runtime is proven.

## 5. Parallel upstream O3K work

Araf implementation may expose upstream requirements. Examples could include missing pagination, capabilities, Operation fields, audit reads or usage contracts.

These become explicit upstream dependencies. Araf must not hide them using frontend-only semantics.

## 6. Post-MVP roadmap

After MVP evidence, priorities can include:

- richer ServiceManifest/service-package lifecycle,
- Offerings and service plans,
- BrandProfile / white-label provider mode,
- managed database/Kubernetes/DNS/load-balancer services,
- OpenStack migration center,
- preflight/impact API and UX,
- signed OCI-distributed service packages,
- separate VM console gateway,
- richer metering/pricing/billing export,
- sandboxed third-party application extension model,
- agent-readable action contracts and safe AI-assisted explanation/generation.

These are deliberately not prerequisites for a credible first MVP.

## 7. Failure conditions

The roadmap must stop and revisit architecture if any of these become true:

- Tenant UI requires direct provider-specific APIs.
- Operator privileges are exposed through the tenant BFF/session.
- Every new service requires custom navigation/list/details code.
- Resource completion is inferred from browser state rather than O3K Operations.
- Upstream API gaps are hidden with a second durable state store in Araf.
- Browser authentication requires persistent bearer/refresh tokens.
- Generic descriptors need arbitrary JavaScript to express ordinary service UX.
