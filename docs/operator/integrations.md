# Backend and IaC integration boundary

## End-to-end request paths

```text
Browser
  +--> Tenant Console --> Tenant BFF --> CloudBackend --> O3K
  |                                  \--> OpenStackBackend --> Keystone/Nova/Glance/Neutron/Cinder
  +--> Operator Console -> Operator BFF -> privileged backend APIs
```

The browser never calls O3K, Keystone or an OpenStack service directly. The
matching BFF owns the browser session, injects server-side credentials and
enforces scope/authorization. Tenant and Operator BFFs share implementation
packages only; they have separate routes, sessions, OIDC clients and
deployment boundaries. Capability discovery is a server response used to hide
or disable unavailable actions; it is not authorization.

## O3K

`O3kBackend` consumes the deployed O3K native contract. O3K is authoritative
for AuthContext/scope, resource identity/lifecycle, quota/governance/audit,
diagnostics/metering and canonical Operations. An accepted asynchronous
request is followed through its Operation; Araf never promotes HTTP `202` to
success. Consult the [O3K contract](../architecture/o3k-integration-contract.md)
for exact schemas and the [limitations](limitations.md) for release status.

## OpenStack

`OpenStackBackend` translates the certified Keystone v3, Nova v2.1, Glance v2,
Neutron v2 and Cinder v3 profile. Those services remain authoritative for
resource state. `CompatibilityOperation` is durable derived correlation and
reconciliation state; it is not an OpenStack authority and cannot override a
subsequent provider read. `unknownOutcome` requires reconciliation rather than
blind replay. Swift, floating IP, attachment and provider-specific tenant UI
are outside the certified profile.

## Terraform/OpenTofu

Terraform and OpenTofu are infrastructure clients, not Araf execution engines.
They manage infrastructure directly through their configured O3K/OpenStack
provider. Araf observes authoritative backend state through its normal
discovery/list/show paths where that resource is supported; it does not
silently import state, generate plans, run Terraform, or become a provider
proxy. Resources created outside Araf may therefore appear after backend
discovery and pagination converge, subject to scope, capability and provider
visibility. Drift correction remains the responsibility of Terraform/OpenTofu
or the cloud backend.

For O3K, use the O3K provider/API contract. For OpenStack, use the supported
OpenStack provider/profile. Do not infer that an Araf tenant action exists just
because a Terraform resource exists, or vice versa.
