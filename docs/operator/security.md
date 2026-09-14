# Secure deployment guide

## Request and trust flow

```text
Browser --HTTPS--> ingress/TLS --> matching static console --> matching BFF
                                                   --> CloudBackend --> O3K or OpenStack
```

Tenant and Operator are separate applications, BFF processes, OIDC clients,
cookie names and deployment trust surfaces. The Operator origin may be private
to a management network. A Tenant session cannot reach Operator routes.

## Araf-owned controls

- Confidential OIDC authorization-code + S256 PKCE; tokens stay server-side.
- Opaque `HttpOnly; Secure; SameSite=Lax` surface-specific session cookies.
- Server-side session lookup, expiry, rotation and revocation.
- Double-submit CSRF nonce (`X-CSRF-Token`) on state-changing methods.
- Strict upstream route allowlisting; the BFF is not a generic proxy.
- CSP, frame denial, nosniff, referrer and permissions headers.
- Descriptor validation, request/body limits, scope checks and structured
  secret-free correlation logs.
- Production startup rejection for fixture mode, insecure URLs, missing key,
  missing durable path or missing backend credentials.

## Deployment-owned controls

The ingress/Kubernetes/environment must provide HTTPS and HSTS, overwrite
forwarded headers, block direct BFF exposure, enforce network policy, protect
the secret manager, encrypt persistent volumes, restrict the runtime UID,
drop capabilities, disable service-account token mounting, and provide
backups/monitoring. Do not trust arbitrary client-supplied forwarded headers.

Validate both origins after every ingress change: certificate coverage,
`Content-Security-Policy`, `X-Frame-Options: DENY`, `nosniff`,
`Referrer-Policy`, `Permissions-Policy`, HSTS and cookie flags. See the
[threat model](../security/threat-model.md) for the complete security
contract and [security evidence](../engineering/security-evidence.md) for
tests; this guide does not replace either.
