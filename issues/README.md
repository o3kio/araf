# Roadmap issue index

The live implementation roadmap is tracked by GitHub epic **#2**.

| Phase | Issue | Purpose |
|---|---:|---|
| EPIC | #2 | Araf MVP — O3K next-generation cloud console |
| M0 | #3 | Repository bootstrap and deterministic quality gates |
| M1 | #4 | Araf design system foundation |
| M2 | #5 | Tenant/Operator shells and explicit scope context |
| M3 | #6 | Rust BFF boundary and deterministic O3K contract harness |
| M4 | #7 | Generic resource list/detail/relationship runtime |
| M5 | #8 | Schema-driven create/edit/action flows |
| M6 | #9 | First-class Operations UX and prototype gate |
| M7 | #10 | Real O3K tenant core-service integration |
| M8 | #11 | Tenant governance, access, quota and audit |
| M9 | #12 | Operator platform, regions and provider health |
| M10 | #13 | ServiceManifest catalog and capability-driven integration |
| M11 | #14 | Usage, quota and bounded cost awareness |
| M12 | #15 | Production authentication and security hardening |
| M13 | #16 | Accessibility, i18n, density and cross-browser closure |
| M14 | #17 | End-to-end MVP acceptance and release candidate |

The Markdown files in this directory preserve the planning source bodies. The **live GitHub issues** are the tracking source of truth for status, dependency discussion and implementation evidence; the **files under `prompts/`** are the versioned implementation instructions.

Do not create duplicate roadmap issues. `scripts/create-issues.sh` is idempotent and skips an issue when its exact title already exists.
