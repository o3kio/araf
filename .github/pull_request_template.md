## Issue

Closes #

Implementation prompt: `prompts/...`

## Scope

Describe the bounded change and explicitly list deferred/non-goal work.

## Architecture

- [ ] Follows `AGENTS.md` and accepted ADRs.
- [ ] Does not make Araf authoritative for missing O3K semantics.
- [ ] Preserves Tenant/Operator trust boundaries.
- [ ] Keeps provider implementation details out of ordinary tenant models.
- [ ] Uses the generic resource/schema runtime where applicable, or documents an approved exception.

## Security and accessibility

Describe authentication/authorization/session/CSP/CSRF/scope implications and accessibility impact.

## Validation

List commands/tests executed and their result. Include contract, Playwright, accessibility, or security evidence when applicable.

## O3K dependencies / gaps

List authoritative upstream contracts used and any missing O3K capability recorded in `docs/engineering/upstream-gaps.md`.

## AI / coding-agent assistance

Record material coding-agent or LLM assistance and the prompt/issue used. Do not include private chain-of-thought or credentials.

## Final evidence

- [ ] Acceptance criteria are satisfied.
- [ ] Relevant docs/ADRs updated.
- [ ] No tests/lints/security gates were bypassed without documented approval.
- [ ] Deferred items are explicit.
