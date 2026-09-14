# Canonical support limitations

This page is the single release/support limitation list. Other documents
should link here instead of inventing broader claims.

- **`#106 remains OPEN — stable-release multi-node HA/O3K certification is intentionally deferred.`** Same-host/session-primitive evidence is not a multi-node O3K HA certification, and Araf must not advertise it as one.
- Araf consumes O3K contracts; an O3K stable semantic/API release version is
  not claimed by the current P4.5/P4.6 artifacts. The native O3K production
  evidence is a convergence/development profile until the stable release gate
  is completed.
- The certified OpenStack profile is Keystone v3, Nova v2.1, Glance v2,
  Neutron v2 and Cinder v3 as recorded by the matched P3.9 evidence. Support
  is profile/version-specific; do not infer “2026.1 or later” compatibility.
- Swift/Object Storage, public floating-IP workflows, volume attachment
  workflows and provider-specific tenant UI are deferred unless separately
  certified and capability-enabled.
- Araf does not own cloud-resource backup, provider scheduling truth or
  universal OpenStack operation semantics. CompatibilityOperation is derived
  reconciliation state only.

See [production release evidence](../engineering/production-release-evidence.md)
and [upstream gaps](../engineering/upstream-gaps.md) for evidence and open
dependencies. These limitations are release claims, not workarounds.
