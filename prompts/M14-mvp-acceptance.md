# M14 implementation prompt — end-to-end MVP acceptance and release candidate

Implement **M14** only after M7-M13 are substantially complete.

## Read first

- all accepted ADRs,
- `docs/product/mvp-prototype.md`,
- `docs/roadmap.md`,
- `docs/engineering/quality-gates.md`,
- prototype/security/accessibility evidence,
- `docs/engineering/upstream-gaps.md`.

## Goal

Produce evidence that Araf is a coherent deployable O3K MVP rather than a set of partially connected frontend features.

## Required implementation/review

1. Build a deterministic MVP acceptance environment using real supported O3K APIs wherever available and explicitly marked fixtures only for non-production demo gaps.
2. Execute and automate representative tenant journey:
   - authenticate,
   - select project/region,
   - browse core resources,
   - create/mutate one supported resource,
   - follow canonical Operation to completion/failure,
   - inspect resource relationships,
   - inspect quota/usage/audit where supported.
3. Execute operator journey:
   - authenticate on separate operator surface,
   - inspect region/provider/service health,
   - find organization/project,
   - investigate a failed operation/correlation trail,
   - verify tenant session cannot reproduce operator access.
4. Prove service extensibility with one descriptor-discovered service that required no shell change.
5. Run all CI/security/accessibility/cross-browser gates.
6. Review bundle boundaries and ensure tenant/operator apps are separately deployable.
7. Create production deployment/configuration documentation including BFF/OIDC/session requirements.
8. Create `docs/engineering/mvp-evidence.md` with PASS/BLOCKED/FAIL for every MVP criterion.
9. Classify every upstream O3K dependency as blocking or post-MVP and do not label the MVP fully complete if a blocker compromises truthful production behavior.
10. Remove accidental prototype shortcuts, dead fixtures in production paths and debug logging.

## Final review questions

- Can a tenant operate O3K without knowing provider internals?
- Can an operator diagnose O3K without data-plane shell access?
- Does every async mutation tell the truth through Operation state?
- Can a new ordinary service integrate without application-shell edits?
- Are tenant/operator trust boundaries independently defensible?
- Is any cloud semantic authoritative only in Araf? If yes, the MVP is not ready.

## Acceptance

All applicable MVP criteria are PASS, or explicitly BLOCKED with an honest release decision. Do not convert blockers into "known issues" merely to declare success.

Branch suggestion: `m14-mvp-acceptance`.
