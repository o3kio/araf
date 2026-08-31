# Generic resource runtime

## 1. Goal

Araf must avoid one React application per cloud service. The generic resource runtime turns authoritative service/resource metadata into consistent lists, details, actions, relationships and operation links.

The runtime is a product differentiator only if it remains strict enough to preserve security and predictable UX.

## 2. Resource descriptor

A representative descriptor should be able to express:

```text
identity
  resource type
  display name/singular/plural
  icon token (not arbitrary URL/script)

scope
  organization/project requirements
  regional/global behavior

collection
  columns
  filters
  sortable fields
  searchable fields
  default views

details
  sections
  fields
  status/lifecycle presentation

actions
  action ID
  HTTP/API operation reference or generated-client operation
  input schema
  risk class
  required effective capability
  async/Operation behavior

relationships
  relationship types
  direction
  display rules

observability
  metrics/meter descriptors when upstream supports them

documentation
  safe documentation references
```

Descriptors are data, not executable code.

## 3. ServiceManifest relationship

The runtime expects a ServiceManifest/service registry to identify installed services and their resource descriptors. Exact upstream serialization is owned by O3K.

Araf may define versioned **console presentation metadata** only where it is clearly presentation-specific and does not redefine cloud semantics.

## 4. JSON Schema + UI metadata

Action/create inputs use JSON Schema for data shape and validation.

Presentation metadata is separate and may define:

- sections,
- ordering,
- widgets,
- help text keys,
- advanced/basic grouping,
- conditional visibility based only on safe form state/capability context.

Do not embed arbitrary JavaScript expressions in descriptors.

## 5. Standard resource page

The generic details shell supports applicable tabs/sections such as:

- Overview
- Configuration
- Relationships
- Metrics
- Operations
- Audit
- Usage & Cost

Tabs appear only when the descriptor and server capabilities support them.

## 6. Standard list behavior

Resource tables support:

- server-side pagination,
- server-side filters,
- sorting,
- saved/local view preferences,
- status display,
- scoped bulk actions only when explicitly supported,
- stable empty/loading/error states.

The runtime must never assume every resource supports delete/edit.

## 7. Actions

Action descriptors have stable IDs and explicit risk classes, for example:

- read/local navigation,
- normal mutation,
- disruptive mutation,
- destructive mutation,
- privileged/operator mutation.

Risk class affects confirmation UX, but authorization is still server-owned.

A destructive action confirmation should show impact/relationships when authoritative data is available. Do not fabricate impact analysis from stale client caches.

## 8. Lifecycle vs health

Araf should visually distinguish lifecycle/provisioning state from health/conditions.

The concrete vocabulary must map to the canonical O3K model. The UI must not create a competing lifecycle enum simply because two services use different legacy strings.

## 9. Custom UI escape hatch

Custom UI is allowed only when a generic descriptor cannot express the interaction without harming usability, such as a future network-topology editor or graphical console.

Custom modules:

- require an explicit issue/ADR,
- consume the same typed API/capability layer,
- cannot bypass resource scope checks,
- cannot import privileged browser credentials,
- cannot redefine generic resource identity/Operation semantics.

## 10. Extension trust levels

1. **Declarative service integration** — default; descriptors/schemas only.
2. **Trusted Araf module** — versioned, reviewed, shipped with the trusted product boundary.
3. **Third-party application** — future; separate/sandboxed trust boundary and no parent session-cookie access.

Arbitrary remote JavaScript loaded into the privileged console is explicitly excluded.

## 11. Prototype proof

M4/M5 must prove the runtime with at least three deliberately different fixture resources. At least one must include relationships and at least one must have a schema-generated mutation that returns an Operation.

A prototype that merely hard-codes Compute, Network and Volume screens does not satisfy this architectural proof.
