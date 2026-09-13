# OpenStack backend support profile

## Goal

Allow Araf to be deployed as a production cloud console for a supported OpenStack cloud while keeping one Araf product/runtime and preserving O3K-native architecture.

## Target OpenStack release

The target release for the supported Kolla deployment and production evidence is
**OpenStack 2026.1 or later**, using the matching `stable/2026.1` Kolla-Ansible
branch and matching 2026.1 service images. OpenStack 2025.1 remains the minimum
acceptable version for compatibility, but it is not the target reference for
new certification evidence.

The Kolla control plane may run inside a KVM/libvirt virtual machine. If that
deployment also provides Nova compute, `nova_compute_virt_type: kvm` requires
nested KVM with `/dev/kvm` exposed to the guest; otherwise the disposable
functional profile must use `qemu` and must not make native-KVM performance
claims.

Evidence recorded against an older OpenStack release is historical baseline
evidence and does not certify the 2026.1 target.

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

## Deployment configuration

Production startup requires an HTTPS `OPENSTACK_AUTH_URL`, either a
server-side `OPENSTACK_TOKEN` or explicit `OPENSTACK_USERNAME` plus
`OPENSTACK_PASSWORD` credentials, and a durable
`ARAF_OPENSTACK_COMPATIBILITY_JOURNAL` path. Service URLs may be pinned with
`OPENSTACK_*_URL`; when omitted, Keystone's public service catalog is used.
Tokens and passwords are never sent to the browser or persisted in browser
storage.

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
