# Contributing to Araf

Araf is issue-driven and specification-first. Human contributors and coding agents follow the same architectural and acceptance rules.

## Before starting

1. Choose one GitHub issue from the roadmap or create a narrowly scoped issue.
2. Read `AGENTS.md`, the matching implementation prompt under `prompts/`, and the linked architecture/ADR documents.
3. Confirm the issue has explicit acceptance criteria, tests, non-goals, and upstream O3K dependencies.
4. Discuss public contracts, trust-boundary changes, or exceptions to the generic resource model before implementation.

## Pull requests

- Keep one logical roadmap issue per PR unless the issue explicitly requires a coordinated change.
- Link the issue and matching implementation prompt.
- Add tests before or with implementation.
- Update docs/ADRs when behavior or architecture changes.
- Record material AI/coding-agent assistance in the PR template.
- Do not include non-public code, internal documents, credentials, production data, or generated artifacts without committed source contracts.
- Do not work around missing O3K capabilities by making Araf authoritative; record the upstream gap instead.

## Required checks

The exact commands are established by M0 and kept in `docs/engineering/quality-gates.md`. Every PR must run the applicable frontend, Rust, contract, end-to-end, accessibility, and security gates.

## Review expectations

Architecture, authentication, authorization, session handling, BFF trust boundaries, cross-tenant scope, CSP, executable extensions, provider-destructive operations, and changes to the manifest/resource contract require explicit human maintainer review.

## Licensing

Unless explicitly marked otherwise, contributions intentionally submitted for inclusion are accepted under Apache-2.0 as described by section 5 of the license. Contributors must have the right to submit their work.
