# Goal — Araf Production Ready

Drive the production roadmap through P1-P4 until Araf can legitimately conclude **GO — PRODUCTION READY**.

Do not optimize for closing issues. Optimize for a stable cloud-console product that can be operated safely in production.

## Current authoritative baseline

P12-IAM is complete and must no longer be treated as an upstream blocker:

- O3K P12-IAM aggregate verdict: PASS;
- O3K production identity contract merge: `e47a81861f1f2c5dac125d017366a959dca39d1b`;
- Araf P12-IAM integration merge: `01b234ba228b0dccfbbb794819933f5d48f97a31`;
- Araf closure evidence: `fd790f5ba9a6841d8a14be7b89e3d302a3135182`;
- real Keycloak -> Araf BFF -> O3K federation, scope selection, tenant resource access, token-custody checks and logout passed.

Do not reopen O3K P12-IAM unless a reproducible regression against its merged contract is found.

The remaining work is Araf product/production convergence.

## Immediate execution order

1. **#38 / P1.2** — make browser E2E an actual CI-produced merge signal and close the current mismatch where branch protection names `browser E2E (Chromium)` but `.github/workflows/ci.yml` does not emit that job.
2. **#39 / P1.3** — verify repository protection end-to-end after #38: normal PRs must require the actual frontend, Rust and browser checks; force-push/deletion and normal direct development must remain blocked.
3. **#40 / P1.4** — make production configuration explicit and fail closed. Production must never fall back to fixtures because `ARAF_UPSTREAM_ADAPTER` is missing or misspelled. Prove HTTPS/proxy/cookie/CSP/CSRF behavior.
4. **#43 / P1.5** — remove or capability-hide every placeholder, fixture-only or dead production route and standardize loading/empty/degraded/forbidden/not-found/error UX.
5. **#44 / P2.1** — close Araf-side production identity convergence using the now-authoritative P12-IAM contract. Replace provider-specific OIDC endpoint assumptions with standards-based discovery, prove a real Operator BFF process journey, preserve Tenant/Operator isolation, and keep reusable IdP/O3K credentials server-side.
6. Continue **#45 -> #50**, then run the hard **#51 / P2.8** real O3K production gate.
7. Only after P2 is proven, complete P3 OpenStack support and P4 industrial release gates according to #42.

## Target state

- coherent Tenant and Operator product UX;
- browser E2E and protected-main governance that are actually enforced;
- HTTPS/BFF security with no browser-held reusable cloud credentials;
- explicit production configuration with zero silent fixture fallback;
- provider-neutral standards-based OIDC client behavior in Araf;
- fully real supported O3K tenant/operator journeys;
- a first-class supported OpenStack backend profile behind the same Araf product/runtime;
- truthful canonical O3K Operations and explicitly derived OpenStack CompatibilityOperations;
- server-bounded scale behavior;
- observable, HA/resilient BFF deployment with durable/shared session semantics when HA is claimed;
- reproducible secured release artifacts;
- tested upgrade/rollback and support runbooks;
- final real-environment acceptance and pilot/soak evidence.

## Non-negotiable rules

- O3K remains authoritative for O3K identity, scope, authorization, resource lifecycle and canonical Operations.
- Araf owns browser OIDC redirects/callbacks, BFF sessions, CSRF and browser-facing security; it must not become a second IAM authority.
- Production mode must fail startup or clearly fail the requested capability when required production configuration/contracts are absent; it must never substitute fixture behavior.
- Tenant and Operator BFFs remain separate trust/session surfaces.
- Backend-specific implementation stays behind backend boundaries; React must not branch throughout the product on backend type.
- Do not claim generic OIDC interoperability from Keycloak-only evidence until standards-based discovery/client behavior is implemented and validated.
- Do not turn Araf's current in-memory session store into an HA claim; durable/shared session work belongs to P4.3 unless promoted earlier by a production requirement.

P1, P2, P3 and P4 are hard gates. Do not continue breadth when an earlier gate exposes architectural/security defects.

The final evidence must compare shipped behavior with `docs/production/definition-of-done.md` and end in exactly one verdict:

**GO — PRODUCTION READY**

**GO WITH BOUNDED DEVIATIONS**

**NO-GO — NOT PRODUCTION READY**

Never declare GO with unresolved isolation/authentication/credential/resource-truth blockers or unresolved HIGH/BLOCKER findings in the supported production profile.