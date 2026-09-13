# OpenStack production support-profile evidence

## Verdict

`GO — OPENSTACK PROFILE SUPPORTED`

The unchanged fail-closed gate (`tests/p3-9-openstack-production-gate.sh`) passed against a real Kolla-Ansible all-in-one deployment and production Tenant and Operator BFFs. The deployment harness created and destroyed real Nova, Glance, Neutron and Cinder resources and wrote the redacted artifact at `target/p3-9-openstack-gate/redacted-run.txt`.

The recorded run had 30 tenant capabilities, real image/flavor/network/subnet/volume/server IDs, authoritative ACTIVE/SHUTOFF transitions, stop/start/reboot, invalid server input, quota reads, deletion, and 25 persisted compatibility operations.

## Deployment

| Item | Observed value |
|---|---|
| Host | Ubuntu 24.04.4 LTS, kernel 6.8.0-139-generic, 16 vCPU, 62 GiB RAM |
| Virtualization | `/dev/kvm` present; disposable Nova profile uses KVM/libvirt |
| Kolla-Ansible | 19.7.0; OpenStack 2024.2 (Dalmatian), Ubuntu Noble images |
| Core services | Keystone, Nova, Glance, Neutron, Cinder, Placement (plus Heat) |
| Service images | `quay.io/openstack/kolla/*:2024.2-ubuntu-noble` |
| Araf ingress | Local CA-backed HTTPS: Tenant 8445, Operator 8446, Keystone proxy 9444 |
| Network topology | Existing management `eth0` preserved; Neutron external `ens19` isolated; no public floating-IP requirement |
| Storage | Dedicated 15 GiB loopback image and `cinder-volumes` LVM VG; no host disks used |
| Object Storage | Not deployed and correctly capability-hidden (`OPENSTACK_OBJECT_STORAGE_URL` unset) |

The stable/2024.2 Kolla collection branch was retired upstream; the deployment used the compatible `openstack.kolla` collection from stable/2025.1 while retaining the requested 2024.2 service images. This packaging deviation is recorded and does not alter the API compatibility target.

## Security and isolation evidence

Keystone credentials are supplied only to the BFF process; the browser receives opaque Araf cookies and never receives an OpenStack token. The harness uses the explicit local CA (`--cacert`), never `--insecure`.

Two disposable Keystone projects were created. A Tenant-B-owned network was looked up by direct ID while the Araf session was scoped to the admin/Tenant-A project and returned 404. The adapter sends Nova `project_id`, Glance `owner`, and Neutron project filters and rejects mismatched direct-ID mappings. Compatibility operations are filtered by selected project.

## Async, failure and restart evidence

Nova state was polled from subsequent resource reads; compatibility operation records are correlation/journal artifacts and never replace provider state. The harness submitted a deliberately invalid image/flavor server request and asserted a real 4xx/5xx response. The tenant BFF was restarted with the same durable journal, returned healthy, and the subsequent real harness observed the pre-existing journal plus new lifecycle operations (25 operations in the recorded run).

## Adapter and capability boundary

The OpenStack adapter translates Keystone/Nova/Glance/Neutron/Cinder wire shapes into provider-neutral descriptors and resources. Unsupported optional services/actions are omitted from the capability map. Server-side pagination is bounded (`limit <= 100`); Neutron's lack of offset is explicitly handled while retaining the browser inventory bound.

## Artifacts and reproducibility

- Gate result: `target/p3-9-openstack-gate/result.env`
- Harness marker: `target/p3-9-openstack-gate/harness.success`
- Redacted run: `target/p3-9-openstack-gate/redacted-run.txt`
- Deployment-owned harness: `/root/p3-9-openstack-run/harness.sh`
- Durable tenant journal: `/root/p3-9-openstack-run/tenant-journal.jsonl`

No passwords, tokens, private keys, or `clouds.yaml` credentials are committed.

## Deferred / non-goals

Swift/Object Storage, public floating-IP exposure, volume attachment UX, and provider-specific React pages remain outside this profile. The compatibility adapter remains distinct from the authoritative O3K control plane; no O3K endpoint or cloud semantic is invented.
