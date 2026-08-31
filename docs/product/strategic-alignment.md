# Strategic alignment with O3K

## 1. Product thesis

Araf must make O3K feel like a coherent cloud operating system rather than a set of infrastructure controllers. The UI is successful when a tenant consumes stable cloud resources while provider implementation can evolve independently underneath.

## 2. O3K goals -> Araf consequences

| O3K strategic goal | Araf design consequence | MVP treatment |
|---|---|---|
| OpenStack upgrade/migration path | Native UX remains O3K; compatibility/migration is an explicit future module | Do not expose Nova/Neutron/Cinder/Glance as native nouns |
| OpenStack Terraform compatibility | Console actions must map to authoritative APIs rather than privileged UI-only behavior | Enforce API parity architecture |
| Native O3K automation | Typed contracts and stable resource/action IDs | Build typed client + descriptor runtime |
| Provider-neutral compute | Tenant sees server/offering; operator may see provider | Prove with core Compute integration |
| Multiple storage/network providers | Tenant UX remains resource-centric; provider health belongs to Operator Console | Prove resource/provider separation |
| Control-plane/data-plane isolation | No provider shell/kubectl/SSH shortcuts in Araf | Hard invariant |
| External controllers | Araf consumes normalized O3K observation/Operation state | Do not talk directly to controller APIs |
| Service extensibility | Manifest/descriptor-driven navigation, resources and actions | M10 architectural proof |
| Durable Operations | Accepted work remains visible/truthful across reload/failure | M6 core differentiator |
| Relationships/composition | Generic relationship/impact navigation | Prototype proof; richer composed products post-MVP |
| Metering | Usage/quota is a horizontal cloud primitive | M11 bounded MVP |
| Tenant self-service | Progressive disclosure, permanent scope, simple core journeys | Core MVP |
| Enterprise/service-provider adoption | Tenant/operator separation, delegated governance, future BrandProfile/plan compatibility | Core boundary now; full white-label post-MVP |
| Industrial acceptance | Security, accessibility, audit, supportability, scalable lists, deterministic CI | Release gates |

## 3. Competitor/reference lessons preserved in the design

Araf deliberately combines lessons rather than cloning a single console:

- **AWS/Cloudscape:** consistent resource-management patterns and mature enterprise UI primitives.
- **Azure:** authorization scope must be visible as part of resource context, not hidden IAM machinery.
- **Google Cloud / STACKIT:** project context and shallow resource hierarchy reduce dangerous ambiguity.
- **STACKIT:** portal/API/CLI/automation parity, scoped IAM and cost/usage awareness are cloud-provider fundamentals.
- **Virtuozzo:** deployable cloud platforms need a real self-service/operator separation and service-provider-compatible architecture.
- **CloudStack:** capabilities should influence what the UI can present instead of hard-coded role menus.
- **Rancher/OpenShift:** runtime extensibility is powerful but executable browser plugins enlarge the trusted computing base; Araf therefore defaults to declarative integration.
- **Horizon/Skyline:** modernizing presentation alone does not create a next-generation Cloud OS if the UI remains tightly bound to legacy service topology.

These references are design benchmarks, not compatibility targets.

## 4. Three deployment businesses, one architecture

Araf must be able to serve:

### Enterprise private cloud
Focus: internal organizations/projects, delegated IAM, quotas, operator health and audit.

### Sovereign/regional public cloud
Focus: self-service, regional context, usage/cost, scalable tenant operations and automation parity.

### MSP/service provider
Focus: customer isolation, delegated administration, future branding/service plans/entitlements and safe operator separation.

The applications and shared runtime remain the same. Configuration/capabilities decide which product features are exposed.

## 5. Strategic differentiators to protect

The MVP should preserve a path toward differentiators that are difficult to retrofit later:

1. manifest-first service UX,
2. provider-neutral resource model,
3. durable Operation-first truth,
4. visible resource relationships,
5. separate tenant/operator trust surfaces,
6. API parity rather than console magic,
7. explicit compatibility/migration layer,
8. machine-readable actions suitable for future tooling/agents.

Visual novelty is not a strategic differentiator. Predictability, explainability and safe extensibility are.

## 6. Post-MVP architecture hooks, not MVP scope

The MVP architecture should remain compatible with, but not implement in full:

- BrandProfile/white-label configuration,
- Offerings/service plans/entitlements,
- OpenStack migration center,
- preflight/impact planning API,
- signed service packages/OCI distribution,
- sandboxed third-party applications,
- dedicated VM console gateway,
- managed Kubernetes/database/DNS/load-balancer products,
- AI-assisted explanation or Terraform/CLI generation.

Do not add generic frameworks for these future features before a concrete requirement appears. Preserve boundaries, not speculative complexity.
