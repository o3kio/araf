# Araf O3K PP.4 convergence evidence

This document records the release-preparation audit for the Araf successor
branch. It does not certify a public release or O3K PP.4 acceptance.

## Audit baseline

- Protected `main`: `2c16bd00e14194a310bde07a990b06a4b7714e41`.
- PR #118 head reviewed: `dccddfd5130c6d6ae4d9dde3259301a04f3485b3`; its CI was green. Its schema fix is carried in this successor as `dca09e4`; the PR remains subject to the repository's required review policy.
- PR #115 head reviewed: `475a89ff81bbb13c37ce38877dc007a8130e01ca`; closed as superseded after selective extraction. Its duplicate GitHub Release authority and stale release-version file were rejected.
- PR #114 established the image/bundle publication split (merge `98e55c2495f924910b5f49290bb721e4f5ee8171`).
- PR #116 established version-derived prerelease policy (merge `2c16bd00e14194a310bde07a990b06a4b7714e41`).
- Existing GitHub Release: `v1.0.0-rc.12`.
- Existing GHCR candidate identity: `v1.0.0-rc.13` is already present for the three canonical images. It is not reused or mutated; the next candidate is deliberately determined only immediately before a future publication.

## Gap analysis before implementation

| Severity | Gap | Resolution or remaining gate |
| --- | --- | --- |
| BLOCKER | Browser mutations did not send the readable `araf_csrf` double-submit token. | API client now reads the exact cookie and sends `X-CSRF-Token`; missing tokens fail closed; browser tests observe the real header without request repair. |
| BLOCKER | No active compatible O3K deployment exists for the real adapter/browser smoke. | Remains an external gate; no O3K claim or Araf candidate publication is made here. |
| HIGH | `/version` could be rewritten by runtime environment; release construction could reuse a candidate identity. | Build-bound identity, release-only validation, OCI/GitHub Release preflight, and digest/source manifest added. |
| HIGH | #115 proposed a second GitHub Release authority. | Rejected; `release-images` owns OCI build/attest and `release-publish` alone owns GitHub Release creation after `workflow_run`. |
| HIGH | Schema runtime selected Ajv draft-07 for O3K JSON Schema 2020-12. | Explicit 2020-12/legacy dialect selection and deterministic unknown-dialect rejection carried from #118. |
| MEDIUM | Release deployment lacked a machine-readable source/digest/backend compatibility tuple. | Added schema-validated release manifest and CI/deployment validation. |
| MEDIUM | Release Compose hardening and runtime identity overrides were inconsistent. | Added selected resource/security limits and removed Helm runtime identity overrides. |
| LOW | Publication notes could reference a missing versioned release note. | `release-publish` now fails closed unless `docs/releases/<version>.md` exists. |

## Verification on successor branch

Branch `fix/pp4-araf-final-convergence`, commit `6536ef3`, PR #119:

- `pnpm format:check`, lint, typecheck, unit/component tests and build pass.
- Rust fmt, clippy, check and workspace tests pass.
- Chromium browser suite: 17 tests pass; schema create/action requests carry the production client's CSRF header.
- Release manifest, release publication guard, release Compose/Helm validation, security, docs, support-bundle and package rollback gates pass.
- Release Compose has no build context, uses exact digest references, separate session volumes/surface routes, read-only roots, dropped capabilities, no-new-privileges, bounded resources and restart policy.
- Production profile requires an explicit backend adapter and has `fixture_mode_allowed: false`; fixture mode is not a release fallback.

## Publication boundary

No tag, OCI candidate, SBOM/provenance bundle or GitHub Release is created from
this unmerged branch. A future publication must choose the next unused version,
create release notes, tag the final source, run the image preflight, resolve
all three canonical OCI image digests (four logical runtime components),
assemble the manifest, and then repeat public artifact verification. Real O3K
adapter/browser smoke remains required before calling the candidate PP.4-ready.
