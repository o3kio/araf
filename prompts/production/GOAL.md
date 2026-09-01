# Goal — Araf Production Ready

Drive the production roadmap through P1-P4 until Araf can legitimately conclude **GO — PRODUCTION READY**.

Do not optimize for closing issues. Optimize for a stable cloud-console product that can be operated safely in production.

The target state is:

- coherent Tenant and Operator product UX;
- browser E2E and protected-main governance;
- HTTPS/BFF security with no browser-held reusable cloud credentials;
- no production fixture fallback or placeholder functionality;
- fully real supported O3K tenant/operator journeys;
- a first-class supported OpenStack backend profile behind the same Araf product/runtime;
- truthful canonical O3K Operations and explicitly derived OpenStack CompatibilityOperations;
- server-bounded scale behavior;
- observable, HA/resilient BFF deployment;
- reproducible secured release artifacts;
- tested upgrade/rollback and support runbooks;
- final real-environment acceptance and pilot/soak evidence.

P1, P2, P3 and P4 are hard gates. Do not continue breadth when an earlier gate exposes architectural/security defects.

The final evidence must compare shipped behavior with `docs/production/definition-of-done.md` and end in exactly one verdict:

**GO — PRODUCTION READY**

**GO WITH BOUNDED DEVIATIONS**

**NO-GO — NOT PRODUCTION READY**

Never declare GO with unresolved isolation/authentication/credential/resource-truth blockers or unresolved HIGH/BLOCKER findings in the supported production profile.