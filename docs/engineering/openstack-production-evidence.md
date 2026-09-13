# OpenStack production support-profile evidence

## Target reference

The target OpenStack release for the supported Kolla deployment is **2025.1 or
later**, using matching Kolla-Ansible tooling and service images. This document
records a current-candidate P3.9 run against OpenStack 2025.1 and retains the
older 2024.2 result as historical context. The profile is certified only for
the capabilities and topology described below.

## Verdict

`GO — OPENSTACK PROFILE SUPPORTED (SINGLE-HOST 2025.1 PROFILE)`

The unchanged fail-closed gate passed against a real Kolla-Ansible 2025.1
all-in-one deployment and production-profile Tenant and Operator BFFs. The
deployment harness created and destroyed real Nova, Glance, Neutron and
Cinder resources and wrote the redacted artifact at
`/tmp/araf-p4-openstack-2025/evidence5/redacted-run.txt`.

The recorded run had 30 tenant capabilities, real image/flavor/network/subnet/volume/server IDs, authoritative ACTIVE/SHUTOFF transitions, stop/start/reboot, invalid server input, quota reads, deletion, and 25 persisted compatibility operations.

## Deployment

| Item | Observed value |
|---|---|
| Host | Ubuntu 24.04.4 LTS, kernel 6.8.0-139-generic, 16 vCPU, 62 GiB RAM |
| Araf implementation source under test | `24a8b691a7c447ce001271519713d5b322757eb8` |
| Virtualization | `/dev/kvm` present; disposable Nova profile uses KVM/libvirt |
| Kolla-Ansible | 19.7.0; OpenStack 2025.1, Ubuntu Noble images |
| Core services | Keystone, Nova, Glance, Neutron, Cinder, Placement (plus Heat) |
| Service images | `quay.io/openstack/kolla/*:2025.1-ubuntu-noble` |
| Araf ingress | Local CA-backed HTTPS: Tenant 8445, Operator 8446, Keystone proxy 9444 |
| Network topology | Existing management `eth0` preserved; Neutron external `ens19` isolated; no public floating-IP requirement |
| Storage | Dedicated 15 GiB loopback image and `cinder-volumes` LVM VG; no host disks used |
| Object Storage | Not deployed and correctly capability-hidden (`OPENSTACK_OBJECT_STORAGE_URL` unset) |

The deployment used the compatible `openstack.kolla` collection from the
Kolla-Ansible 19.7.0 environment with matching 2025.1 service images. No
cross-version image substitution was used.

## Security and isolation evidence

Keystone credentials are supplied only to the BFF process; the browser receives opaque Araf cookies and never receives an OpenStack token. The harness uses the explicit local CA (`--cacert`), never `--insecure`.

Two disposable Keystone projects were created. A Tenant-B-owned network was looked up by direct ID while the Araf session was scoped to the admin/Tenant-A project and returned 404. The adapter sends Nova `project_id`, Glance `owner`, and Neutron project filters and rejects mismatched direct-ID mappings. Compatibility operations are filtered by selected project.

## Async, failure and restart evidence

Nova state was polled from subsequent resource reads; compatibility operation records are correlation/journal artifacts and never replace provider state. The harness submitted a deliberately invalid image/flavor server request and asserted a real 4xx/5xx response. The tenant BFF was restarted with the same durable journal, returned healthy, and the subsequent real harness observed the pre-existing journal plus new lifecycle operations (25 operations in the recorded run).

## Adapter and capability boundary

The OpenStack adapter translates Keystone/Nova/Glance/Neutron/Cinder wire shapes into provider-neutral descriptors and resources. Unsupported optional services/actions are omitted from the capability map. Server-side pagination is bounded (`limit <= 100`); Neutron's lack of offset is explicitly handled while retaining the browser inventory bound.
Tenant requests are bound to the configured/session project; arbitrary project selection and inventory remain restricted to the separate operator surface. Supported sort fields are forwarded to the provider, and Neutron pages advance with marker cursors.

## Artifacts and reproducibility

- Gate result: `/tmp/araf-p4-openstack-2025/evidence5/harness.success`
- Redacted run: `/tmp/araf-p4-openstack-2025/evidence5/redacted-run.txt`
- Deployment-owned harness: `/root/p3-9-openstack-run/harness.sh`
- Durable tenant journal: `/root/p3-9-openstack-run/tenant-journal.jsonl`

No passwords, tokens, private keys, or `clouds.yaml` credentials are committed.

## Deferred / non-goals

Swift/Object Storage, public floating-IP exposure, volume attachment UX, and provider-specific React pages remain outside this profile. The compatibility adapter remains distinct from the authoritative O3K control plane; no O3K endpoint or cloud semantic is invented.
