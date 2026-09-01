# Production implementation prompts

Use one prompt per production issue. The phase ID in the filename matches the production roadmap.

## Universal execution rules

For every phase:

1. Start from latest clean `main` and read `AGENTS.md`.
2. Read `docs/production/*`, `docs/architecture/backend-abstraction.md`, relevant ADRs, threat model and quality gates.
3. Inspect the current repository and current O3K/OpenStack contracts; never implement from stale assumptions.
4. Preserve Tenant/Operator trust separation and `@araf/ui` ownership.
5. Production mode must fail closed. Fixtures remain test/development-only.
6. Do not add Araf authority for O3K resource/IAM/quota/Operation state.
7. Backend-specific code stays behind backend boundaries.
8. Add positive and negative tests appropriate to the change.
9. Run frontend/Rust/browser gates that the phase touches.
10. Open a focused PR linked to the issue and report exact commands/results, findings and remaining upstream gaps.

## Stop conditions

Stop and report instead of hacking around the problem if the requested production behavior requires an authoritative O3K/OpenStack contract that does not exist or cannot be implemented safely in Araf.

## Final evidence format

Report branch, SHA, PR, issue, implementation, architecture proof, security proof, tests, backend/upstream dependencies, deferred scope and findings using BLOCKER/HIGH/MEDIUM/LOW-NIT.