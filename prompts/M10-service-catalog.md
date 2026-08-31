# M10 implementation prompt — ServiceManifest catalog and capability-driven service integration

Implement **M10** after the generic runtime and representative real integration are understood.

## Read first

- `AGENTS.md`
- `docs/architecture/resource-runtime.md`
- `docs/architecture/o3k-integration-contract.md`
- ADR 0003
- current O3K ServiceManifest/service registry contracts.

## Goal

Prove that installing/enabling a normal O3K service can make it appear in Araf without changing application-shell navigation or creating bespoke list/details pages.

## Required implementation

1. Inspect the actual current ServiceManifest/service-registry contract; do not assume planning field names are authoritative.
2. Implement service discovery/adaptation into Araf's generic runtime.
3. Build Tenant Service Catalog view showing only installed + entitled/capable services.
4. Build Operator Installed Services/catalog view with safe version/status metadata available upstream.
5. Map resource descriptors/actions/capabilities into generic list/detail/form/action primitives.
6. Add descriptor schema/version validation and fail closed on unsupported dangerous fields.
7. Demonstrate adding a new fixture/test service without changing application-shell source code.
8. Demonstrate a supported real service where upstream metadata is sufficient; if not sufficient, record exact upstream descriptor gaps.
9. Keep custom executable modules out of the default runtime.
10. Document version compatibility rules between Araf runtime metadata and O3K service metadata.

## Acceptance

- discovered services appear dynamically based on server data and entitlement,
- a new normal fixture service requires no shell/navigation change,
- descriptor validation rejects executable/unsafe content,
- action capabilities come from authoritative context,
- gaps between current ServiceManifest and desired presentation metadata are explicit rather than hidden in service-specific code.

Branch suggestion: `m10-service-catalog-runtime`.
