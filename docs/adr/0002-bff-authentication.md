# ADR 0002: Rust BFF owns browser authentication tokens

Status: Accepted for prototype/MVP planning

## Context

A cloud console is a high-value XSS target. Persisting OAuth/O3K bearer or refresh tokens in browser storage expands the impact of script compromise and makes privileged operator sessions particularly risky.

## Decision

Use a Rust Backend-for-Frontend as the confidential OIDC client and token holder. Browsers receive only an opaque secure session cookie and CSRF protection.

The BFF may proxy/aggregate approved O3K calls but must not become a generic forward proxy or a second cloud-control authority.

## Consequences

Positive:
- browser JavaScript does not own reusable cloud tokens,
- centralized session/logout/security-header controls,
- clear tenant/operator session separation.

Cost:
- server component required for console deployment,
- session storage/HA strategy required for production,
- CSRF/session lifecycle testing required.
