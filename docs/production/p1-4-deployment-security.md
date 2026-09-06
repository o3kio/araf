# P1.4 production deployment security

The supported production topology is illustrated by
[`nginx-production.conf.example`](../../deploy/nginx-production.conf.example)
and is not tied to nginx; an equivalent ingress/load balancer is valid.

The topology is:

```text
Browser --HTTPS--> TLS ingress/load balancer --HTTP or HTTPS--> console static files
                                             \--> /api --> the matching Tenant or Operator BFF --HTTPS/HTTP--> O3K
```

TLS terminates at the browser-facing ingress. The ingress owns the certificate
and emits HSTS (`Strict-Transport-Security: max-age=31536000; includeSubDomains`)
only on its HTTPS virtual host. The BFF is not exposed directly to browsers.
Internal ingress-to-BFF HTTP is permitted only inside the private deployment
network; the O3K hop uses the configured `O3K_URL` and may be HTTP only when the
private O3K boundary explicitly provides equivalent network protection.

The ingress must overwrite, rather than append to, `Host`, `X-Forwarded-Proto`,
`X-Forwarded-Host` and `X-Forwarded-For`. Direct BFF access is blocked by the
network policy. Araf does not make security decisions from forwarded headers
received from arbitrary clients. The browser-visible origin is configured
explicitly through `ARAF_PUBLIC_URL` and `ARAF_TRUSTED_ORIGINS`.

## Required production configuration

Both BFF processes require:

```text
ARAF_RUNTIME_PROFILE=production
ARAF_UPSTREAM_ADAPTER=o3k
O3K_URL=https://o3k.internal.example/o3k
ARAF_PUBLIC_URL=https://console.example
ARAF_TRUSTED_ORIGINS=https://console.example
ARAF_TENANT_OIDC_CLIENT_ID=...
ARAF_TENANT_OIDC_CLIENT_SECRET=...
ARAF_TENANT_OIDC_ISSUER_URL=https://identity.example/realms/cloud
ARAF_TENANT_OIDC_REDIRECT_URI=https://console.example/login/callback
```

The Operator process uses the corresponding `ARAF_OPERATOR_OIDC_*` values and
its own browser origin. Secrets are injected by the deployment secret manager,
never committed or logged. Missing, empty, malformed, insecure production URLs,
unknown adapters and fixture mode fail before the listener starts. There is no
production localhost/default adapter.

Development and tests must explicitly set `ARAF_RUNTIME_PROFILE=development`
or `test` and `ARAF_UPSTREAM_ADAPTER=fixture`; fixture sessions and resources
are unavailable in the production profile.

The BFF exposes `/healthz` for liveness. Readiness is the successful process
startup/configuration validation boundary; upstream capability failures are
returned as explicit errors by the O3K adapter and are never replaced by
fixture data.

Production sessions use separate `araf_tenant_session` and
`araf_operator_session` opaque cookies: `Secure`, `HttpOnly`, `SameSite=Lax`,
`Path=/`, and one-day expiry. The readable CSRF cookie contains only a CSRF
nonce; OIDC/O3K tokens remain server-side. Mutating requests require the
server-stored nonce in `X-CSRF-Token`. CORS allows only configured trusted
origins.

The effective BFF CSP is:

```text
default-src 'self'; script-src 'self' 'strict-dynamic'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self' ws: wss:; frame-ancestors 'none'; base-uri 'self'; block-all-mixed-content
```

`style-src 'unsafe-inline'` remains because the current UI library emits
runtime styles; no inline scripts, evaluation, remote scripts or analytics are
allowed. The static ingress adds the same non-HSTS security headers, while HSTS
belongs only to its HTTPS boundary.
