# OpenStack 2026.1 P3.9 redacted evidence

This artifact records the matched-target compatibility run executed on
2026-09-14. It contains only opaque resource identifiers and aggregate
results; cookies, tokens, passwords, and `clouds.yaml` are intentionally not
included.

| Field | Value |
| --- | --- |
| Araf source under test | `40db777b06f49d3a329a8e3813cb5686ac0758d9` |
| OpenStack target | 2026.1 Gazpacho (SLURP) |
| Kolla-Ansible | 22.2.0 (2026.1 series) |
| Service image family | `quay.io/openstack/kolla/*:2026.1-ubuntu-noble` |
| Harness profile | `kolla-2026.1-clean` |
| Transport | CA-validated HTTPS for Tenant, Operator, Keystone and OIDC |

## Result

The unchanged deployment-owned P3.9 harness passed (`ARAF_P3_9_HARNESS_PASS=1`).
It exercised production-profile Tenant and Operator BFFs with real Keystone,
Nova, Glance, Neutron and Cinder APIs:

- 30 discovered capabilities and server-side bounded collections;
- Tenant project scope and direct-ID isolation (cross-project lookup returned
  404);
- real network, subnet, volume and server create/read/delete operations;
- Nova asynchronous create, stop, start and reboot transitions;
- invalid server input with a failed compatibility operation;
- quota reads and 18 persisted compatibility operations;
- cleanup of all harness-created resources.

Opaque IDs recorded by the harness were image `29778c83-ea66-4004-b661-
eae889cabd86`, flavor `1`, network `f72cffb0-fdf4-451e-a3b9-04795037a624`,
subnet `27c0678f-d45f-46e5-9dd2-c858af07056c`, volume
`be7504b3-79d3-40a7-9f1e-8a7695fdbef9`, and server
`f00ce351-d92c-442b-99f3-4a6f842c6d4a`. These identifiers are retained only to
make the run auditable against the disposable deployment logs.

The deployment used an internal-only disposable Neutron bridge and
`ENABLE_EXT_NET=0`; no public network, floating IP, Swift/Object Storage, or
volume attachment capability is claimed. The 2025.1 image run remains
supplemental cross-series compatibility evidence, not a 2025.1 matched
certification.

## Certification interpretation

This closes the matched OpenStack 2026.1 P3.9 evidence requirement for the
documented Keystone/Nova/Glance/Neutron/Cinder profile. It does not by itself
establish the full P4 production-release verdict: multi-host HA and rolling
upgrade, trusted release provenance, representative pilot/soak, and other
release gates remain tracked in
[`production-release-evidence.md`](production-release-evidence.md).
