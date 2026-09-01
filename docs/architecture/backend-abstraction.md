# Backend abstraction design target

## Purpose

Araf is the native O3K console, but it should also be able to operate a supported OpenStack cloud without duplicating the React product or exposing Nova/Neutron/Cinder/Glance as the tenant mental model.

## Boundary

```text
React generic resource/runtime
          |
      Araf BFF API
          |
    CloudBackend contract
      /             \
 O3kBackend     OpenStackBackend
      |             |
 O3K native      Keystone/Nova/Glance/
 API             Neutron/Cinder/Swift*
```

`CloudBackend` is a server-side/application boundary. It must not become a least-common-denominator domain model that degrades O3K.

## Rules

1. O3K remains the primary/native semantics.
2. Backend capabilities are explicit. Unsupported OpenStack features are unavailable, not simulated.
3. The browser never talks directly to Keystone/Nova/etc.
4. Backend credentials/tokens remain server-side.
5. Resource identity includes backend provenance internally so IDs cannot collide or cross scopes.
6. Authorization remains enforced by the backend; Araf capability discovery is UX guidance only.
7. O3K canonical Operations remain authoritative when using O3K.
8. OpenStack has no equivalent universal Operation object. Araf may create a **CompatibilityOperation** to correlate an accepted request and derive progress/result from authoritative OpenStack resource/task state. It is never authoritative for resource state and must reconcile after restart.
9. Backend-specific fields may be exposed only through bounded advanced/operator projections, not scattered through generic tenant components.
10. Multi-backend aggregation is not required for the first production release.

## P3.1 requirement

Before implementation, P3.1 must convert this design into an accepted ADR with concrete Rust/TypeScript boundaries, error/capability contracts, identity/scope model, compatibility-operation persistence/reconciliation rules and negative security tests.