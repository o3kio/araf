# Araf security model and threat baseline

## 1. Security objective

Araf is a privileged cloud control-plane client. Compromise can lead to resource destruction, credential abuse, cross-tenant exposure or infrastructure impact. Security boundaries therefore take precedence over frontend convenience.

## 2. Trust boundaries

Primary boundaries:

1. Browser -> Tenant BFF
2. Browser -> Operator BFF
3. BFF -> O3K API
4. Tenant identity/scope -> other tenant scopes
5. Tenant Console -> Operator Console
6. Declarative service metadata -> trusted UI runtime
7. Future custom/third-party extension -> parent console
8. Future VM console session -> normal cloud API session

Tenant and Operator browser sessions are separate security contexts.

## 3. Authentication/session model

Production target:

- OIDC authorization-code flow through a confidential Rust BFF client,
- BFF retains OAuth/O3K tokens server-side,
- browser receives an opaque secure session cookie,
- cookie is `Secure`, `HttpOnly` and appropriately `SameSite`/host-scoped,
- explicit CSRF defense for state-changing browser requests,
- session rotation/expiry/logout behavior documented and tested.

Never persist access/refresh tokens in browser storage.

## 4. BFF constraints

The BFF:

- has a strict allowlist of upstream O3K destinations/routes,
- is not a generic forward proxy,
- performs server-side session lookup,
- propagates only required auth/scope context,
- enforces request/body limits,
- preserves correlation IDs,
- redacts secrets from logs,
- does not downgrade upstream authorization failures.

Tenant and Operator BFFs may share internal crates but not runtime session namespaces or privileged route exposure.

## 5. Authorization and isolation

Frontend visibility is not authorization.

Every authoritative request must be authorization checked by O3K/BFF according to documented responsibility. Araf must have negative tests for:

- guessed resource ID across projects,
- stale project switch,
- manipulated project/region path/query,
- hidden action invoked directly,
- operator route attempted with tenant session,
- descriptor requesting unauthorized action.

## 6. XSS/content injection

Treat resource names, metadata, descriptions, filenames, error details and service descriptors as untrusted.

Requirements:

- React escaping by default,
- no raw HTML rendering without an explicit audited sanitizer/use case,
- strict CSP,
- no `unsafe-eval`,
- no arbitrary runtime CDN scripts,
- no descriptor-supplied JavaScript,
- URLs validated/allowlisted by purpose.

## 7. CSRF

Cookie-authenticated mutation endpoints require robust CSRF protection. SameSite is defense-in-depth, not the sole protection.

State-changing GET routes are forbidden.

## 8. Service metadata supply chain

Manifest/descriptor content can influence privileged UI behavior and therefore must be treated as a supply-chain input.

MVP requirements:

- schema validation,
- version checks,
- rejection of unknown executable fields,
- deterministic rendering,
- safe string/URL handling.

Signed package/provenance enforcement may land after MVP, but the descriptor model must remain compatible with it.

## 9. Operator risk

Operator Console should support stronger deployment controls than Tenant Console, including private management-plane exposure if desired by deployers.

High-risk actions should be architected for future step-up authentication/approval, but MVP must not invent bypassable client-side step-up semantics if upstream identity support is absent.

## 10. Sensitive data and logging

Never log:

- access/refresh tokens,
- session cookie contents,
- provider credentials,
- private keys,
- secret values,
- authorization headers.

Structured logs should include safe correlation, route, status, scope identifiers when policy permits.

## 11. CSP/security headers

Production deployment must define and test at minimum:

- strict Content-Security-Policy,
- frame-ancestors protection,
- nosniff/content-type behavior,
- Referrer-Policy,
- appropriate Permissions-Policy,
- HSTS when deployed on HTTPS production origins.

Exact policy must be generated/tested against the final asset model rather than copied blindly.

## 12. Future graphical console boundary

Graphical/serial VM console is not MVP. When added, it must use a separate short-lived, single-resource, audited ticket/session boundary and must not expose reusable O3K credentials to the console client.

## 13. Security release evidence

MVP release requires documented evidence for:

- browser token absence,
- tenant/operator separation,
- CSRF tests,
- CSP tests,
- cross-scope negative tests,
- descriptor injection tests,
- dependency/security scan results,
- secret/log redaction checks.
