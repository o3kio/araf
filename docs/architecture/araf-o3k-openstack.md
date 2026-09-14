# Araf, O3K and OpenStack architecture

Araf is one next-generation cloud-console product with two supported backend families:

- **O3K**, which is Araf's native semantic model and richest integration;
- **OpenStack**, supported through a bounded server-side compatibility backend.

Araf replaces the *role* of a traditional cloud dashboard such as Horizon; it is not a Horizon fork and does not implement a Horizon compatibility protocol. The browser-facing product stays provider-neutral while backend-specific authority and wire protocols remain behind server-side boundaries.

![Araf, O3K and OpenStack architecture](araf-o3k-openstack-architecture.webp)

## Architecture in one sentence

**Araf provides one Tenant/Operator user experience, the BFFs enforce the browser security boundary, `CloudBackend` selects O3K or OpenStack server-side behavior, and each cloud remains authoritative for its own resource state.**

## 1. User experience and trust surfaces

Araf has one shared product/runtime with two distinct security surfaces:

- **Tenant Console / Tenant BFF** for self-service cloud consumption;
- **Operator Console / Operator BFF** for operator and administrative workflows.

They may share frontend packages and generic resource/runtime code, but they do not collapse their browser sessions, OIDC clients, or trust boundaries.

The normal React product must not branch into separate O3K and OpenStack applications. Backend provenance and provider-specific details may be exposed in bounded operator/diagnostic views where required, but the common tenant vocabulary remains resource-oriented: Virtual Machine, Network, Image, Volume, Project, Operation, Usage, Quota, and capability-driven services.

## 2. Server-side backend boundary

The browser never talks directly to O3K control-plane credentials or to Keystone/Nova/Neutron/Glance/Cinder tokens. Those credentials remain behind the BFF.

The central application boundary is:

```text
React generic resource/runtime
          |
     Tenant/Operator BFF
          |
       CloudBackend
       /          \
 O3kBackend   OpenStackBackend
     |              |
 O3K native     supported OpenStack
 API            service APIs
```

`CloudBackend` is not allowed to become a least-common-denominator cloud model that degrades O3K semantics. Capabilities are explicit and unsupported backend behavior is hidden or rejected, never simulated.

## 3. O3K backend

`O3kBackend` consumes the O3K native `/o3k/v1` API family.

O3K remains authoritative for:

- IAM/AuthContext and authorization;
- resource identity, ownership, desired state and lifecycle;
- canonical Operations and Relationships;
- topology, Regions, Availability Zones and failure domains;
- placement/scheduling and capacity semantics;
- quota, policy, audit and metering;
- service-registry/catalog semantics.

O3K's internal architecture is a shared Cloud Kernel plus native domains and typed southbound execution boundaries. It is **not** implemented internally as Nova + Neutron + Cinder + Keystone.

Infrastructure mutations that require provider execution cross versioned typed provider contracts over gRPC + mTLS to components such as `o3k-compute`, `o3k-network`, and `o3k-storage`.

### O3K's separate OpenStack compatibility path

O3K also exposes selected OpenStack-compatible northbound contracts for existing OpenStack clients, SDKs, Terraform/OpenTofu, migration, and explicitly proven external OpenStack services.

That compatibility path is independent of Araf's O3K-native backend:

```text
OpenStack CLI / SDK / Terraform / OpenTofu
                    |
       O3K OpenStack-compatible APIs
                    |
             same O3K Cloud Kernel
```

Compatibility does not redefine O3K's internal resource model.

## 4. OpenStack backend

`OpenStackBackend` operates a supported real OpenStack cloud through the BFF. The first certified production support profile covers the Araf journeys backed by:

- Keystone v3;
- Nova v2.1;
- Glance v2;
- Neutron v2;
- Cinder v3;
- capability and quota discovery required by the supported journeys;
- Araf `CompatibilityOperation` correlation/reconciliation for asynchronous UX.

Object storage is optional and capability-driven. Swift or an explicitly configured S3-compatible endpoint is not assumed to exist.

OpenStack remains authoritative for OpenStack-backed resources. Araf must not manufacture cloud truth. A `CompatibilityOperation` is correlation state only: it derives progress/result from authoritative OpenStack resource/service state and must reconcile after restart.

## 5. Current support status

As of 2026-09-13:

- the **O3K backend** has passed Araf's P2.8 native production-convergence gate;
- the bounded **OpenStack backend profile** has historical real-environment P3.9 evidence against OpenStack 2024.2 (Dalmatian) and a supplemental single-host run using 2025.1 service images;
- that 2025.1 run used Kolla-Ansible 19.7.0, which is Dalmatian-series tooling rather than the matching 20.x Epoxy toolchain, so it is compatibility evidence rather than a matched-toolchain certification;
- the current production reference target is **OpenStack 2026.1 Gazpacho (SLURP)** with matching Kolla-Ansible 22.x tooling and 2026.1 service images; a clean P3.9 run against that exact series is required before 2026.1 is advertised as the current certified OpenStack release;
- Araf does not claim automatic compatibility with `2026.1 or later`; each later series requires its own production gate;
- multi-backend aggregation in one running console is **not required** for the first production release;
- optional OpenStack Object Storage, public floating-IP exposure, volume-attachment UX, and provider-specific React pages remain outside the certified P3.9 profile unless separately proven.

The OpenStack evidence is recorded in [`../engineering/openstack-production-evidence.md`](../engineering/openstack-production-evidence.md).

## 6. Edge-to-datacenter truth on the O3K side

The diagram intentionally distinguishes product direction from evidence.

O3K targets the same product semantics from edge/office deployments through server-room/rack/cage environments to datacenter scale. Today, the bounded edge profile has real execution evidence; larger rack/cage/datacenter-scale placement, networking, database/control-plane and failure-domain behavior require their own measured evidence.

Araf should therefore present the same product model as O3K scales, while capability discovery controls what a particular deployment can honestly expose.

## 7. Optional external OpenStack services around O3K

O3K's OpenStack-compatible APIs may also become an ecosystem boundary for independently hosted services such as Octavia, Designate, Barbican or Manila.

Those services are **not implied by endpoint names**. Each integration requires explicit dependency-contract discovery and real conformance evidence before being advertised as supported.

## 8. Architectural invariants

1. Araf is a console, not a second cloud control plane.
2. Browser JavaScript never owns raw cloud backend credentials/tokens.
3. Tenant and Operator trust surfaces remain separate.
4. O3K remains authoritative for O3K cloud semantics.
5. OpenStack remains authoritative for OpenStack resource state.
6. Backend-specific code stays behind server-side backend boundaries rather than branching the React product.
7. Unsupported capabilities are absent or disabled, never faked.
8. O3K canonical Operations remain authoritative on O3K; OpenStack `CompatibilityOperation` objects are derived correlation state only.
9. O3K's OpenStack-compatible northbound APIs are a separate compatibility/ecosystem path, not Araf's native O3K integration path.
10. Support and scale claims remain bounded by recorded evidence.

## Related architecture and evidence

- [`overview.md`](overview.md) — Araf product/trust-surface architecture
- [`backend-abstraction.md`](backend-abstraction.md) — backend abstraction design
- [`../adr/0005-cloud-backend-openstack-compatibility.md`](../adr/0005-cloud-backend-openstack-compatibility.md) — accepted CloudBackend/OpenStack boundary
- [`o3k-integration-contract.md`](o3k-integration-contract.md) — O3K authority and integration rules
- [`../production/openstack-support.md`](../production/openstack-support.md) — supported OpenStack profile
- [`../engineering/openstack-production-evidence.md`](../engineering/openstack-production-evidence.md) — P3.9 evidence
