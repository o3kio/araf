# OpenStack production support-profile evidence

## Target reference

The current OpenStack certification target is **OpenStack 2026.1 Gazpacho
(SLURP)**, deployed with matching `stable/2026.1` Kolla-Ansible 22.x tooling
and matching 2026.1 service images. OpenStack support is version-specific;
Araf does not claim automatic support for `2026.1 or later` without separate
P3.9 evidence for each later series.

As of 2026-09-13, 2026.1 is the current maintained stable SLURP release.
OpenStack 2026.2 Hibiscus is still a development series. Existing 2024.2 and
2025.1 results are retained below as historical/transitional compatibility
evidence only.

## Current target verdict

`NO-GO — OPENSTACK 2026.1 TARGET NOT YET CERTIFIED`

A clean P3.9 run using matching Kolla-Ansible 22.x tooling and 2026.1 service
images is required before the repository may advertise 2026.1 as the current
certified OpenStack release.

## Supplemental 2025.1 single-host run

A later single-host run exercised 2025.1 service images through the unchanged
fail-closed gate and production-profile Tenant and Operator BFFs. The deployment
harness created and destroyed real Nova, Glance, Neutron and Cinder resources
and wrote a redacted local artifact at
`/tmp/araf-p4-openstack-2025/evidence5/redacted-run.txt`.

The recorded run had 30 tenant capabilities, real image/flavor/network/subnet/volume/server IDs, authoritative ACTIVE/SHUTOFF transitions, stop/start/reboot, invalid server input, quota reads, deletion, and 25 persisted compatibility operations.

This is useful compatibility evidence, but it is **not** a matched-toolchain
2025.1 certification: the run used Kolla-Ansible 19.7.0, which belongs to the
OpenStack 2024.2 Dalmatian Kolla series, with 2025.1 service images. Official
OpenStack 2025.1 Epoxy Kolla-Ansible is the 20.x series. The earlier wording
that described this pairing as matching 2025.1 tooling was incorrect.

## Deployment observed in the supplemental run

| Item | Observed value |
|---|---|
| Host | Ubuntu 24.04.4 LTS, kernel 6.8.0-139-generic, 16 vCPU, 62 GiB RAM |
| Araf implementation source under test | `24a8b691a7c447ce001271519713d5b322757eb8` |
| Virtualization | `/dev/kvm` present; disposable Nova profile uses KVM/libvirt |
| Kolla-Ansible | 19.7.0 (Dalmatian-series tooling) |
| OpenStack service images | `quay.io/openstack/kolla/*:2025.1-ubuntu-noble` |
| Core services | Keystone, Nova, Glance, Neutron, Cinder, Placement (plus Heat) |
| Araf ingress | Local CA-backed HTTPS: Tenant 8445, Operator 8446, Keystone proxy 9444 |
| Network topology | Existing management `eth0` preserved; Neutron external `ens19` isolated; no public floating-IP requirement |
| Storage | Dedicated 15 GiB loopback image and `cinder-volumes` LVM VG; no host disks used |
| Object Storage | Not deployed and correctly capability-hidden (`OPENSTACK_OBJECT_STORAGE_URL` unset) |

## Security and isolation evidence

Keystone credentials are supplied only to the BFF process; the browser receives opaque Araf cookies and never receives an OpenStack token. The harness uses the explicit local CA (`--cacert`), never `--insecure`.

Two disposable Keystone projects were created. A Tenant-B-owned network was looked up by direct ID while the Araf session was scoped to the admin/Tenant-A project and returned 404. The adapter sends Nova `project_id`, Glance `owner`, and Neutron project filters and rejects mismatched direct-ID mappings. Compatibility operations are filtered by selected project.

## Async, failure and restart evidence

Nova state was polled from subsequent resource reads; compatibility operation records are correlation/journal artifacts and never replace provider state. The harness submitted a deliberately invalid image/flavor server request and asserted a real 4xx/5xx response. The tenant BFF was restarted with the same durable journal, returned healthy, and the subsequent real harness observed the pre-existing journal plus new lifecycle operations (25 operations in the recorded run).

## Adapter and capability boundary

The OpenStack adapter translates Keystone/Nova/Glance/Neutron/Cinder wire shapes into provider-neutral descriptors and resources. Unsupported optional services/actions are omitted from the capability map. Server-side pagination is bounded (`limit <= 100`); Neutron's lack of offset is explicitly handled while retaining the browser inventory bound.
Tenant requests are bound to the configured/session project; arbitrary project selection and inventory remain restricted to the separate operator surface. Supported sort fields are forwarded to the provider, and Neutron pages advance with marker cursors.

## Artifacts and reproducibility

The 2025.1 supplemental artifacts were generated on the development host:

- Local harness marker: `/tmp/araf-p4-openstack-2025/evidence5/harness.success`
- Local redacted run: `/tmp/araf-p4-openstack-2025/evidence5/redacted-run.txt`
- Deployment-owned harness: `/root/p3-9-openstack-run/harness.sh`
- Durable tenant journal: `/root/p3-9-openstack-run/tenant-journal.jsonl`

Paths under `/tmp` are **not durable release artifacts**. They are useful local
evidence references only and must not be treated as immutable certification
evidence after the host/environment is removed. A 2026.1 certification run
should publish a durable redacted artifact through the repository/CI evidence
mechanism.

No passwords, tokens, private keys, or `clouds.yaml` credentials are committed.

## Historical 2024.2 baseline

Araf previously passed the P3.9 support-profile gate against a real Kolla-Ansible
OpenStack 2024.2 (Dalmatian) environment. That result remains valuable
historical compatibility evidence but does not certify the current 2026.1
reference release.

## Required 2026.1 certification closure

Before changing the current target verdict to GO:

1. deploy OpenStack 2026.1 Gazpacho using matching Kolla-Ansible 22.x tooling;
2. use matching 2026.1 service images;
3. run the unchanged P3.9 gate through production-profile Tenant and Operator BFFs;
4. exercise the documented Keystone/Nova/Glance/Neutron/Cinder profile, failure path, isolation, restart/reconciliation and quota journeys;
5. publish durable redacted evidence tied to the exact Araf HEAD and exact OpenStack/Kolla versions;
6. only then record `GO — OPENSTACK PROFILE SUPPORTED` for the 2026.1 target.

## Deferred / non-goals

Swift/Object Storage, public floating-IP exposure, volume attachment UX, and provider-specific React pages remain outside this profile. The compatibility adapter remains distinct from the authoritative O3K control plane; no O3K endpoint or cloud semantic is invented.
