# Security evidence

This file documents the security features implemented in Araf and the evidence that they are effective.

## Authentication

- **Browser token absence**: Access tokens, refresh tokens, and O3K credentials are never stored in `localStorage`, `sessionStorage`, IndexedDB, or browser-readable cookies. The browser receives only an opaque HttpOnly Secure session cookie (prefixed `araf_tenant_session` or `araf_operator_session`).
- **OIDC authorization-code flow**: The confidential Rust BFF exchanges authorization codes at the OIDC provider token endpoint. Tokens remain server-side in the `SessionStore`.
- **Fixture mode**: When OIDC is not configured, the fixture adapter creates sessions with synthetic identity. No real tokens are issued or exchanged.

## Tenant/Operator isolation

- **Separate applications**: ADR 0001 mandates separate Tenant and Operator console applications with separate BFF processes, OIDC clients, cookie namespaces, and deployment boundaries.
- **Route separation**: The tenant BFF router (`tenant_api_router`) does not mount operator routes. Negative tests (`tenant_bff_does_not_expose_operator_endpoints`, `operator_resource_types_reject_tenant_session`) confirm 404 on operator paths from tenant sessions.
- **Session namespace isolation**: Session cookie name is surface-specific (`araf_tenant_session` vs `araf_operator_session`).

## CSRF protection

- **Double-submit cookie pattern**: State-changing HTTP methods (POST, PUT, DELETE, PATCH) require an `X-CSRF-Token` header matching the token stored in the server-side session.
- **SameSite=Lax**: Session cookies are set with `SameSite=Lax` as defense-in-depth.
- **Negative tests**: Tests confirm missing or wrong CSRF tokens return `403 Forbidden`.

## Content Security Policy

- **Strict CSP**: `default-src 'self'; script-src 'self' 'strict-dynamic' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self' ws: wss:; frame-ancestors 'none'; base-uri 'self'; block-all-mixed-content;`
- **No `unsafe-eval`**: The CSP does not include `unsafe-eval`, preventing arbitrary code execution from evaluated strings.
- **No remote scripts**: No external CDN or arbitrary third-party script origins are allowed.

## Security headers applied

| Header | Value |
|---|---|
| `Content-Security-Policy` | See CSP section above |
| `X-Frame-Options` | `DENY` |
| `X-Content-Type-Options` | `nosniff` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | `camera=(), microphone=(), geolocation=()` |

## Session lifecycle

- **Session TTL**: Default 24 hours (`DEFAULT_SESSION_TTL`).
- **Rotation**: `SessionStore::rotate` invalidates the old token and issues a new one (called on privilege escalation).
- **Logout**: Destroys the server-side session and clears the cookie with `Max-Age=0`.
- **Expiry**: Expired sessions are rejected by `SessionStore::validate` and cleaned up by `reap_expired`.

## Descriptor validation

- **Backend**: `descriptor_validation.rs` scans all service discovery responses for executable or unsafe content (event-handler script keys, `eval`/`exec` keys, dangerous URL schemes). Applied to `/api/v1/services`, `/api/v1/services/catalog`, `/api/v1/operator/services/installed`, and `/api/v1/operator/services/resource-types` before reaching the browser.
- **Frontend**: `packages/resources/src/descriptor.ts` performs additional validation for defense-in-depth.

## Cross-scope denial

- Negative tests prove that cross-project resource access (`cross_project_access_is_rejected`), stale/incorrect project IDs, and tenant→operator route access are rejected with `403` or `404`.

## Log redaction

- Structured log spans capture method, URI, request id, and correlation id only. Sensitive headers (Authorization, Cookie, CSRF token) are never included in structured fields. A middleware layer (`redact_sensitive_logs`) is in place to enforce this.

## Upstream gaps

See `docs/engineering/upstream-gaps.md` entries:
- M3-O3K-001 through M3-O3K-011: Session, tokens, capabilities, errors.
- M12-O3K-001: Session storage HA strategy (production deployments should replace in-memory store with Redis).

## Security scan evidence

(Dependency/SBOM scanning is not yet integrated into CI; see deferred scope.)
