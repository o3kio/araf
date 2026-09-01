# OpenStack backend support profile

## Goal

Allow Araf to be deployed as a production cloud console for a supported OpenStack cloud while keeping one Araf product/runtime and preserving O3K-native architecture.

## Core v1 profile

Required:

- Keystone v3 identity, project and service-catalog discovery
- Nova v2.1 server lifecycle and server reads
- Glance v2 image discovery required by server creation
- Neutron v2 network/subnet/port/security-group operations required by supported Araf network journeys
- Cinder v3 volume lifecycle and attachment-related reads required by supported Araf storage journeys
- backend capability discovery
- quotas required for supported services where the deployed OpenStack APIs expose them
- Araf CompatibilityOperation tracking/reconciliation for async UX

Optional capability profile:

- Swift object storage
- S3-compatible object storage endpoint (for example an operator-configured RGW S3 endpoint)

Optional services must not be assumed to exist.

## Authentication

Human credentials must remain behind the BFF. Prefer federated/OIDC-compatible Keystone deployments where available. If legacy username/password login is supported for broad compatibility, it must be an explicit deployment mode, never persist the raw password, and remain subject to the same secure-session boundary.

## Native UX mapping

Araf presents generic terms such as Virtual Machine, Network, Volume, Image, Project and Object Storage. Nova/Neutron/Cinder/Glance/Keystone names belong to adapter/operator diagnostics, not normal tenant navigation.

## Compatibility Operations

OpenStack asynchronous state is service-specific. The adapter must correlate commands with authoritative service/resource status and produce a compatibility view for Araf. It must not claim success from HTTP acceptance alone and must recover/reconcile after BFF restart.

## Non-goals for first supported profile

- Horizon compatibility
- Heat orchestration UI
- Octavia UI
- Designate UI
- Magnum UI
- Ceilometer/Gnocchi requirement
- multi-cloud aggregation
- cross-cloud migration

These can become later capability profiles.