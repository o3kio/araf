# Protected `main` governance

This document records the GitHub configuration verified for the Araf
repository's `main` branch on 2026-09-06. It describes the live policy, not a
desired future configuration.

## Rule

The repository uses the GitHub classic branch-protection rule for `main`.
There are no GitHub rulesets currently configured for this repository.

`main` is protected and normal changes require a pull request. The required
status contexts are exactly:

- `frontend (format, lint, typecheck, test, build)`
- `rust (fmt, clippy, check, test)`
- `browser E2E (Chromium)`

The required checks must pass on the current branch tip (`strict: true`), so a
branch that is behind `main` must be updated before it can merge.

## Reviews and conversations

At least one approving review is required. Stale approvals are dismissed when
the pull request changes. Code-owner approval and approval from the last
push are not additionally required. Conversation resolution is required
before merge.

## Branch integrity and merge methods

- Force pushes to `main` are disabled.
- Deletion of `main` is disabled.
- Direct ordinary pushes do not satisfy the pull-request requirement.
- Merge commits, squash merges and rebase merges are enabled; automatic merge
  is disabled.

Administrator enforcement is disabled in the live GitHub configuration. An
administrator can therefore use an emergency bypass, but that is exceptional,
must be explicit and auditable, and is not the routine development path.
Routine changes use a reviewed, green, up-to-date pull request.

## Evidence and limitations

The configuration and required contexts were verified through the GitHub API.
The three contexts were emitted and passed by CI on the P1.2 merge and its
post-merge `main` run. A normal fully-green reviewed merge requires an
approver other than the pull-request author; self-approval is rejected by
GitHub. That reviewer-dependent proof must be completed by a maintainer with
an independent account when the repository's normal development workflow is
exercised.
