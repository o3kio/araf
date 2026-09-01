# Production deployment and security boundary

## Configuration modes

`ARAF_ENV` must be one of `development`, `test`, or `production`. Fixture
adapters are available only outside production. In production,
`ARAF_UPSTREAM_ADAPTER=o3k` is mandatory; an omitted, unknown, or fixture
adapter causes the BFF to terminate during startup. This prevents a failed
O3K configuration from silently presenting fixture cloud state.

Production OIDC configuration must also provide `ARAF_OIDC_CLIENT_ID`,
`ARAF_OIDC_CLIENT_SECRET`, `ARAF_OIDC_ISSUER_URL`,
`ARAF_OIDC_REDIRECT_URI` (HTTPS), `ARAF_OIDC_AUTHORIZATION_URL`, and
`ARAF_OIDC_USERINFO_URL`. Provider authorization and userinfo endpoints are
explicit deployment inputs; the BFF does not invent or default them.

The production browser origin is HTTPS. TLS is terminated by the deployment's
ingress or reverse proxy, which must forward `Host`, `X-Forwarded-Proto`, and
the client IP chain. Only a deployment-controlled, allowlisted proxy may set
these headers; the BFF must not be exposed directly to untrusted clients.

The reference proxy should emit HSTS only when HTTPS is enforced for the full
host, for example:

```nginx
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
```

An executable reference is provided in
[`deploy/nginx-production.conf.template`](../../deploy/nginx-production.conf.template).
It redirects HTTP to HTTPS, terminates TLS, sets the security headers, and
forwards only the explicitly supported proxy headers to the private BFF.

The browser-facing API is same-origin under `/api/`. Session cookies must be
opaque, `Secure`, `HttpOnly`, `SameSite=Lax` (or stricter where compatible),
path-scoped, and cleared on logout. CSRF tokens are sent in a request header;
state-changing requests without a valid token are rejected.

The auth callback rejects synthetic fixture codes and missing callback state in
production. The remaining OIDC state/nonce correlation and provider-backed
session middleware are still an upstream integration dependency tracked in
[`docs/engineering/upstream-gaps.md`](../engineering/upstream-gaps.md).

The current MVP BFF still uses fixture session injection and in-memory session
state. Therefore this document records the deployment boundary and fail-closed
adapter behavior, but does not claim production identity/session readiness;
that is a P2 dependency and must be completed before a production verdict.
