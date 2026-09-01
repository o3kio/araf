# `main` protection

The `main` branch is protected through GitHub branch protection. Normal changes
must use a pull request with at least one approving review. Stale approvals are
dismissed, conversations must be resolved, and the branch must be up to date
before merging.

The required checks are the exact GitHub Actions job names:

- `frontend (format, lint, typecheck, test, build)`
- `rust (fmt, clippy, check, test)`
- `browser E2E (Chromium)`

Force-pushes and branch deletion are disabled. Administrator enforcement is
intentionally not enabled so an administrator can perform an emergency bypass;
such a bypass is exceptional, must be auditable in the GitHub audit log, and
must be followed by a normal pull request and complete required checks.

The configuration can be verified with:

```sh
gh api repos/o3kio/araf/branches/main/protection
```
