# Araf prototype and MVP definition

## 1. Objective

Araf must prove that O3K can expose a modern, stable cloud-provider experience without coupling the console to individual infrastructure providers or hand-building every future service screen.

The first delivery therefore validates architecture before breadth.

## 2. Personas

### Tenant user
Creates and operates permitted cloud resources inside a project and region without understanding provider internals.

### Organization/project administrator
Manages project membership, access, quotas and developer credentials within delegated scope without obtaining platform-operator authority.

### Cloud operator
Manages platform health, regions, provider capacity, organizations, service availability and operations through the control plane.

### Support/security operator
Investigates resource/operation/audit evidence under explicit scoped authority. Invisible impersonation is not part of the MVP.

### Service developer
Publishes O3K resource/action descriptors so normal service UX can be generated without modifying the application shell.

## 3. Prototype gate

The prototype is an architecture and UX proof. It may use deterministic contract fixtures upstream of the BFF, but fixture mode must be unmistakably isolated from production adapters.

The prototype is complete when all of the following are demonstrated:

1. Tenant and Operator applications run as separate applications while sharing UI/runtime packages.
2. Tenant shell permanently exposes organization/project and regional context.
3. Operator shell exposes platform-level context without leaking those privileges into the tenant app.
4. At least three representative resource types render from generic resource descriptors using the same list/details primitives.
5. At least one create flow is generated from JSON Schema + Araf UI metadata instead of a hand-written service page.
6. Capability metadata controls visible/enabled actions while the fixture API still enforces the same capability server-side.
7. ✅ **M6** A submitted async action transitions through a canonical Operation object and never reports completion from HTTP acceptance alone.
8. ✅ **M6** Operation list/detail/timeline is usable from both an affected resource and the global Operations view.
9. Resource relationships are rendered generically for at least one composed example.
10. Problem Details/correlation identifiers produce actionable error UX.
11. The same resource total can represent 100,000 records while the browser only handles bounded paginated results.
12. Keyboard-only navigation works through one critical tenant journey.

### Prototype reference journey

Tenant:

1. Select project (`project-1`) and region (`eu-west`) in the tenant shell.
2. Navigate to `/resources/compute.server` to list servers.
3. Open a server detail page at `/resources/compute.server/{id}`.
4. Click **Create** to open `/resources/compute.server/create`.
5. Submit the schema-driven create form.
6. The success screen shows "Request accepted" with the canonical Operation id, state, and correlation id, plus a link to `/operations/{operation.id}`.
7. Open the operation detail page to watch the authoritative event timeline transition `Pending -> Running -> Succeeded/Failed`.
8. Reload `/operations/{operation.id}` — the Operation and its events survive because the BFF persists the canonical Operation, not just the HTTP 202 response.
9. Return to the server detail page and open the **Operations** tab to see operations scoped to that resource, with a link to the global Operations list.

Operator:

1. Open the operator console at `/platform/overview`.
2. Navigate to `/operations` to view the platform-wide operations list with server-bounded pagination and filters.
3. Open a failed operation at `/operations/{id}`.
4. Inspect the structured error section (code, title, detail) and the event timeline to correlate the failure with the affected resource and correlation id.

## 4. MVP gate

The MVP turns the proven architecture into a deployable O3K console.

Required:

- real Rust Tenant and Operator BFFs,
- OIDC authorization-code flow using confidential BFF clients and secure browser sessions,
- separate Tenant/Operator session and deployment boundaries,
- native O3K integration for the representative core resources actually supported by upstream at implementation time,
- canonical O3K Operation integration, preferably with SSE/event-driven updates when the upstream contract supports it,
- organization/project/region context,
- capability-driven authorization UX backed by server enforcement,
- Compute, Network, Volume and Image tenant experiences where upstream contracts support them,
- tenant governance views for project access/quota/audit capabilities that exist upstream,
- operator read/manage flows for organizations/projects/regions/provider health appropriate to upstream support,
- ServiceManifest/resource-descriptor discovery sufficient to prove a new normal service can integrate without shell changes,
- usage/quota presentation and a bounded cost-estimation interface when metering/pricing data exists; no fabricated billing,
- strict CSP/CSRF/session/security baseline,
- WCAG 2.2 AA for critical paths,
- deterministic CI and end-to-end acceptance suite,
- deployable artifacts and documented configuration.

## 5. MVP non-goals

Explicitly deferred:

- invoice generation, tax, payments or accounting,
- marketplace procurement,
- arbitrary remote-JavaScript plugin execution,
- full service-provider reseller hierarchy,
- full branding editor (architecture must remain compatible with BrandProfile configuration),
- OpenStack migration-center implementation,
- graphical/serial VM console gateway,
- Kubernetes/database/DNS/load-balancer product implementations,
- AI agent that executes privileged cloud actions,
- Terraform generation from forms,
- complete provider credential/configuration UI,
- native mobile application.

## 6. Success criteria

The MVP is successful if a prospective enterprise or cloud-provider customer can understand and safely operate a representative O3K environment without learning the internal hypervisor/storage/network implementation, while an operator can diagnose platform work through Operations, resource relationships, scoped identity and provider health.

The architectural success criterion is stronger: adding a simple future O3K service must primarily require descriptors/contracts, not application-shell code.
