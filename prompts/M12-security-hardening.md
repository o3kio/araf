# M12 implementation prompt — production authentication and security evidence

Implement/finalize **M12** after the main MVP surfaces exist. Security requirements in this prompt should have been respected continuously; this phase closes production evidence.

## Read first

- `AGENTS.md`
- `docs/security/threat-model.md`
- ADR 0001 and ADR 0002
- current O3K identity/token/AuthContext contracts.

## Goal

Turn the prototype/real-integration BFF boundary into a defensible production browser security architecture and prove tenant/operator isolation.

## Required implementation

1. Implement OIDC authorization-code login through confidential Tenant/Operator BFF clients using currently supported secure libraries.
2. Keep OAuth/O3K access and refresh tokens server-side.
3. Implement opaque secure session cookies, rotation/expiry/logout and documented session storage strategy.
4. Implement robust CSRF defense for state-changing browser requests.
5. Maintain separate tenant/operator:
   - OIDC clients,
   - cookie names/session namespaces,
   - allowed routes,
   - deployment configuration.
6. Add strict upstream destination allowlisting; verify no open proxy behavior.
7. Define/test production CSP and security headers; no `unsafe-eval` or arbitrary runtime third-party scripts.
8. Add security tests for:
   - cross-project resource ID,
   - manipulated scope,
   - tenant session against operator endpoint,
   - CSRF missing/invalid,
   - malicious resource names/metadata,
   - malicious descriptor content,
   - token/session/log redaction.
9. Generate dependency/SBOM/security scan evidence in CI/release pipeline where tooling is available.
10. Document production configuration and secret requirements without committing secrets.

## Stop conditions

Do not weaken CSP, CSRF or token storage to accommodate an implementation shortcut. If the chosen UI dependency requires unsafe runtime behavior, fix/replace the dependency or document an architectural blocker.

## Acceptance

- browser never receives persistent reusable O3K refresh/access credentials by design,
- tenant/operator isolation is proven by negative tests,
- session fixation/logout/expiry behavior is tested,
- CSRF attacks are rejected,
- strict CSP passes with shipped assets,
- sensitive values are absent from logs/browser storage,
- security evidence is summarized in `docs/engineering/security-evidence.md`.

Branch suggestion: `m12-security-hardening`.
