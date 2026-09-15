# Araf operator documentation

This is the starting point for operating a released Araf deployment. It is
written for a cold operator: use the published OCI image digests, the Helm
chart or Compose reference, and the procedures linked here. Source checkout
and maintainer chat are not prerequisites.

## Start here

1. [Architecture and authority](../architecture/araf-o3k-openstack.md) — trust
   boundaries, BFFs, backend authority and the integration flows.
2. [Installation](installation.md) — prerequisites, digest verification,
   Helm/Compose, ingress, TLS, persistence and probes.
3. [Configuration reference](configuration.md) — supported production
   settings, defaults, scope and restart requirements.
4. [Secrets and rotation](secrets.md) — custody, injection and safe rotation.
5. [Security deployment](security.md) — controls owned by Araf versus the
   ingress, Kubernetes and secret manager.
6. [Observability](observability.md) — health, metrics, logs and correlation.
7. [Troubleshooting](troubleshooting.md) — executable decision trees and
   O3K/OpenStack incident runbooks.
8. [Upgrade and rollback](upgrade-rollback.md) — artifact-only rollout flow.
9. [Recovery and continuity](recovery.md) — durable Araf state, IdP outage
   and backend outage behavior.
10. [Support bundle](support-bundle.md) — safe evidence collection and
    redaction proof.
11. [Support limitations](limitations.md) — the canonical capability and
    release-claim boundary.
12. [Backend and IaC integration](integrations.md) — O3K/OpenStack flows and
    the Terraform/OpenTofu boundary.
13. [LLM/cold-operator quick answers](llm-navigation.md) — terminology and
    navigation checks for an independent operations agent.

## Authority map

Documentation explains and links; it does not redefine contracts. When a
semantic matters, consult the authoritative source:

| Question | Authority |
| --- | --- |
| Araf trust surfaces and BFF responsibilities | [Architecture overview](../architecture/overview.md), [ADR 0001](../adr/0001-console-security-surfaces.md), [ADR 0002](../adr/0002-bff-authentication.md) |
| O3K identity, scope, resources, capabilities and Operations | [O3K integration contract](../architecture/o3k-integration-contract.md) and the deployed O3K API/specification |
| OpenStack translation and CompatibilityOperation rules | [CloudBackend ADR](../adr/0005-cloud-backend-openstack-compatibility.md) and [backend abstraction](../architecture/backend-abstraction.md) |
| Release artifacts and deployment shape | [P4.5 packaging evidence](../engineering/packaging-evidence.md), [`deploy/helm/araf`](../../deploy/helm/araf/), [`deploy/docker-compose.release.yml`](../../deploy/docker-compose.release.yml) |
| Security requirements | [Threat model](../security/threat-model.md) and [security evidence](../engineering/security-evidence.md) |

## Non-negotiable boundary

The browser talks only to the matching BFF. Araf is not authoritative for
cloud state: O3K owns native O3K semantics and OpenStack services own
OpenStack state. A `202 Accepted` response is not completion. O3K mutations
must be followed through the canonical O3K Operation; OpenStack mutations use
an explicitly derived CompatibilityOperation and a subsequent authoritative
resource read.

`#106 remains OPEN — stable-release multi-node HA/O3K certification is intentionally deferred.`
