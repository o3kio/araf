# Araf architecture overview

## 1. Product boundary

Araf is the human-facing control-plane client for O3K. It is not a second control plane and must not own authoritative cloud state.

The stable UX model is based on:

- Scope
- Resource
- Relationship
- Capability
- Action
- Operation
- Service
- Policy/Quota
- Meter/Event

Provider implementation is deliberately outside the normal tenant mental model.

## 2. Deployment surfaces

Araf has one shared platform and two security surfaces:

```text
                 shared Araf packages
                        |
          +-------------+-------------+
          |                           |
   Tenant Console                Operator Console
   public/self-service           management surface
          |                           |
   Tenant BFF                    Operator BFF
          |                           |
          +-------------+-------------+
                        |
                   O3K Native API
```

The applications may live in one repository and share most UI/runtime code. They do not share browser sessions, OIDC clients or deployment trust boundaries.

## 3. Organizational and regional scope

Target conceptual hierarchy:

```text
Provider
  -> Account
      -> Organization
          -> Project
              -> Resources
```

An optional one-level grouping/folder may be introduced later if product evidence requires it. Arbitrarily recursive organizational trees are intentionally avoided.

Region and Availability Zone are orthogonal dimensions:

```text
Region -> Availability Zone
```

A resource has organizational scope and, when applicable, regional scope. Global resources must not pretend to be region-scoped.

The exact canonical upstream nouns remain controlled by O3K. Araf adapters must map to documented upstream scope contracts rather than force this conceptual model onto incompatible APIs.

## 4. Service, Provider and Offering are separate

### Service
The capability consumed by the customer, such as Compute or Block Storage.

### Provider
The implementation/authority realizing the resource, such as a compute or storage controller.

### Offering
The customer-facing product configuration/policy, such as a general-purpose VM size or encrypted premium volume.

A tenant chooses an Offering. Placement/provider selection is an operator/control-plane concern unless the service intentionally exposes a meaningful choice.

## 5. Generic resource runtime

Normal service UX must be generated from resource/action metadata:

```text
ServiceManifest
  -> ResourceDescriptor
      -> list columns and filters
      -> details fields/sections
      -> actions + input schemas
      -> required capabilities
      -> relationships
      -> metrics/meters when available
```

The target is that routine services require little or no custom frontend code.

## 6. Operations as first-class state

Async actions are represented by canonical O3K Operation objects.

Araf distinguishes:

- request accepted,
- operation running,
- operation succeeded,
- operation failed/cancelled according to the upstream model.

The frontend must never infer successful completion merely from an accepted HTTP response.

Operations must be reachable globally and from related resources. Correlation identifiers and structured failure information are retained.

## 7. Relationships

Resource relationships are first-class and generic. They support:

- dependency visibility,
- composed-product drill-down,
- impact explanation before destructive actions,
- support diagnosis.

The relationship graph is not permission bypass: every related resource fetch remains scope/authorization checked.

## 8. API parity

Araf may aggregate APIs through its BFF but must not create a privileged shadow cloud API.

If an action exists only in the console, that is an architecture defect unless it is strictly local presentation/session behavior.

The desired authority model is:

```text
                O3K API
           /       |       \
      Console     CLI     Terraform/SDK
```

## 9. Frontend stack

Baseline:

- TypeScript, strict mode
- React
- Vite
- React Router
- TanStack Query for server state
- JSON Schema 2020-12 + Ajv-compatible validation
- generated API clients/types where upstream specifications permit
- O3K-owned `@o3k/ui` abstraction
- underlying enterprise component primitives isolated inside `packages/ui`
- Vitest + Testing Library
- Playwright for critical flows

Do not introduce Next.js without a new ADR demonstrating a concrete requirement that Vite/client applications cannot satisfy.

## 10. BFF stack

Rust BFF services are console-specific security/application gateways. Expected responsibilities:

- confidential OIDC client behavior,
- secure server-side token handling,
- opaque browser session management,
- CSRF defense,
- strict upstream allowlisting/routing,
- auth-context propagation,
- API aggregation only where presentation needs justify it,
- event/SSE forwarding where supported,
- correlation and normalized transport errors without changing authoritative semantics.

The BFF is not allowed to manufacture cloud authorization or durable resource truth.

## 11. Repository target layout

```text
apps/
  tenant-console/
  operator-console/
packages/
  ui/
  shell/
  api-client/
  resources/
  schema-runtime/
  operations/
  fixtures/
backend/
  console-bff-core/
  tenant-bff/
  operator-bff/
contracts/
  fixtures/
docs/
prompts/
```

The exact workspace tooling is established in M0, but the security/package boundaries must remain.
