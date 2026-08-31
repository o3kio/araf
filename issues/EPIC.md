# [EPIC] Araf MVP — O3K next-generation cloud console

## Objective

Deliver a production-credible prototype and MVP of Araf as the human interface to the O3K Cloud Operating System. The console must prove provider-neutral resource UX, truthful Operation-driven async behavior, manifest-driven service extensibility, separate tenant/operator trust surfaces and production-grade security/accessibility without becoming a second O3K control plane.

## Strategic outcomes

- Enterprise private-cloud self-service.
- Regional/sovereign cloud-provider UX.
- Service-provider/MSP-compatible architecture without product forks.
- Native O3K resource model with OpenStack compatibility isolated from native UX.
- Portal/API/CLI/Terraform parity.
- Provider-neutral tenant experience; operator-only provider visibility.
- Extensible service catalog driven by ServiceManifest/resource descriptors.
- Durable O3K Operations as first-class UI objects.
- Usage/quota awareness without premature billing/invoicing scope.

## Prototype gate — M0-M6

- [ ] M0 Repository bootstrap and deterministic quality gates
- [ ] M1 Araf design system foundation
- [ ] M2 Tenant/Operator shells and explicit scope context
- [ ] M3 Rust BFF boundary and deterministic O3K contract harness
- [ ] M4 Generic resource list/detail/relationship runtime
- [ ] M5 Schema-driven create/edit/action flows
- [ ] M6 First-class Operations UX and prototype GO/NO-GO evidence

The MVP integration phase does not begin until M6 concludes that the generic runtime produces acceptable cloud UX.

## MVP gate — M7-M14

- [ ] M7 Real O3K tenant core-service integration
- [ ] M8 Tenant governance, access, quota and audit
- [ ] M9 Operator platform, regions and provider health
- [ ] M10 ServiceManifest catalog and capability-driven integration
- [ ] M11 Usage, quota and bounded cost awareness
- [ ] M12 Production authentication and security hardening
- [ ] M13 Accessibility, i18n, density and cross-browser closure
- [ ] M14 End-to-end MVP acceptance and release candidate

## MVP non-goals

Full billing/invoicing, marketplace execution, arbitrary remote-JavaScript plugins, full reseller hierarchy, migration-center implementation, graphical VM console, Kubernetes/database product implementations, AI-controlled administration, Terraform generation and native mobile applications.

## Source of truth

Read `docs/product/mvp-prototype.md`, `docs/architecture/overview.md`, `docs/architecture/o3k-integration-contract.md`, `docs/security/threat-model.md`, `docs/engineering/quality-gates.md`, `docs/roadmap.md` and `AGENTS.md`.

## Completion rule

Do not close this epic because pages exist. Close only when `docs/engineering/mvp-evidence.md` from M14 demonstrates that the release criteria are PASS, or when remaining BLOCKED items have been explicitly judged non-blocking without compromising truthful production behavior or the frozen trust boundaries.
