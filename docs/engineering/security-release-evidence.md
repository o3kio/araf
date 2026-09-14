# P4.4 security and supply-chain gate

The release gate is `tests/security-release-gate.sh` and runs in the required
frontend CI job. It performs deterministic checks for browser credential
persistence, tracked credential patterns, executable UI construction, moving
workflow actions, and release SBOM/provenance configuration. It emits locked
Rust and frontend dependency inventories and a redacted manifest under
`target/security/`.

Runtime security regression coverage remains in the Rust contract suite:

- CSRF and trusted-origin rejection;
- strict CSP/security headers;
- descriptor/schema executable-content rejection;
- tenant/operator and cross-project authorization negatives;
- session cookies and server-side token custody; and
- sensitive log-field redaction.

Container images run as non-root users supplied by the distroless and
nginx-unprivileged base images and expose only the BFF/static-console ports.
Release publication
must attach a digest, the generated SBOM/dependency inventories and CI
provenance attestation. No BLOCKER/HIGH finding is accepted for the advertised
O3K or supported OpenStack profile.

Validation on the development host:

```text
./tests/security-release-gate.sh                  PASS
cargo audit                                       PASS (0 advisories)
cargo deny check licenses bans sources             PASS (policy warnings reviewed)
pnpm audit --prod --audit-level=high               PASS (no known vulnerabilities)
cargo test --workspace --all-features              PASS
pnpm build                                         PASS
```

The tag-triggered `.github/workflows/release-images.yml` is the publication
path. Buildx emits SBOM and maximum provenance metadata, Trivy scans the
published digest for HIGH/CRITICAL vulnerabilities, and
`actions/attest-build-provenance` binds that digest to the GitHub OIDC build
identity. The workflow and scanner image are commit/digest pinned. Local
development signatures or scans are not treated as trusted release evidence;
the release owner must verify the CI attestation before publication. No SLSA
level is claimed.

The exact candidate SHA, scanner versions, and dependency inventory are
captured by the generated `target/security/manifest.json`; generated output is
not committed because it is candidate-specific.
