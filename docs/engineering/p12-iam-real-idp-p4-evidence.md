# P4 real external-IdP process evidence

## Run

- Date: 2026-09-13
- Araf source: `f0743c2537fd463e6065b3f2095a84c527f999d4`
- O3K source: `74b992a997315cbbc4bb5ffecdc002a2ff16217f`
- Harness: O3K `tests/p12-iam-7-real-idp.sh` with the Araf P12-IAM.8 process hook
- Identity provider: disposable Keycloak realm created by the harness
- O3K provider: fake provider (the process and HTTP contract were real)
- Transport: disposable local HTTP topology; no production TLS claim

## Redacted result

The harness passed IAM.7 and IAM.8. The real browser-like flow completed:

1. Keycloak authorization-code login;
2. Araf callback acceptance;
3. opaque HttpOnly session and CSRF cookie capture;
4. authenticated session check;
5. O3K federated scope discovery and project selection;
6. context and resource requests through the Araf Tenant BFF;
7. browser-cookie/token-custody scan; and
8. CSRF-protected logout.

The direct O3K scope probe returned project `project-a` with HTTP 200, and the
Araf scope projection returned the same canonical project with HTTP 200. No
access, refresh, or ID token was present in the browser cookie jar or session
headers.

This evidence closes the current-process external-IdP/session integration
check. It does not establish production HTTPS termination, a supported
OpenStack deployment, multi-host rolling failover, or trusted CI provenance.
