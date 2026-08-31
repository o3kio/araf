# Implementation prompts

Use one prompt per GitHub issue. Do not combine phases unless the issue explicitly says so.

## Execution rules

For every prompt:

1. Start from current protected `main`.
2. Read `AGENTS.md` and all documents named by the prompt.
3. Inspect the repository before proposing changes; do not assume files/tooling already exist.
4. Implement only the requested phase.
5. If a required O3K production contract does not exist, document an upstream gap rather than inventing one.
6. Keep fixture/demo adapters isolated from production code paths.
7. Run all relevant quality gates.
8. Open a focused PR; do not merge unless separately instructed.
9. Finish with the report format required by `AGENTS.md`.

## Phase gates

- M0-M6: Prototype architecture gate.
- M7-M14: MVP integration and production-readiness gate.

M12/M13 own final security/accessibility evidence, but every earlier phase must implement applicable security/accessibility requirements rather than deferring them.
