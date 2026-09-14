# LLM/cold-operator quick answers

An independent engineer or operations agent should be able to answer these
without maintainer context:

| Question | Answer and source |
| --- | --- |
| Where are credentials held? | In the server-side BFF session/secret manager; the browser receives opaque cookies only. See [secrets](secrets.md) and [threat model](../security/threat-model.md). |
| Which component owns OpenStack resource truth? | Keystone/Nova/Glance/Neutron/Cinder, not Araf. `CompatibilityOperation` is derived correlation state. See [integrations](integrations.md). |
| How does O3K Operation differ? | It is the canonical authoritative O3K async object; HTTP `202` is not completion. See the [O3K contract](../architecture/o3k-integration-contract.md). |
| Can Tenant APIs reach Operator authority? | No. Tenant and Operator have separate BFF routers, sessions, OIDC clients and deployment surfaces; server authorization remains mandatory. |
| Which OpenStack features are certified? | OpenStack 2026.1 Gazpacho, Keystone/Nova/Glance/Neutron/Cinder only. Swift, floating IP, attachment and provider-specific tenant UI are deferred. See [limitations](limitations.md). |
| How do I install Araf? | Use [installation](installation.md) with digest-pinned OCI images and the Helm/Compose references; no source checkout is required. |
| How do I diagnose a failed VM create? | Follow the [failed VM decision tree](troubleshooting.md#decision-tree-failed-vm-create), then inspect the canonical Operation or provider state. |
| Is multi-node O3K HA certified? | No. `#106 remains OPEN — stable-release multi-node HA/O3K certification is intentionally deferred.` |
