# P4.4 security and supply-chain gate

The release gate is `tests/security-release-gate.sh` and runs in the required
frontend CI job. It performs deterministic checks for browser credential
persistence and committed private-key/bearer-token patterns, emits locked Rust
and frontend dependency inventories under `target/security/`, and writes the
release license/provenance policy used by artifact publication.

Runtime security regression coverage remains in the Rust contract suite:

- CSRF and trusted-origin rejection;
- strict CSP/security headers;
- descriptor/schema executable-content rejection;
- tenant/operator and cross-project authorization negatives;
- session cookies and server-side token custody; and
- sensitive log-field redaction.

Container images run as non-root users supplied by the distroless/nginx base
images and expose only the BFF/static-console ports. Release publication
must attach a digest, the generated SBOM/dependency inventories and CI
provenance attestation. No BLOCKER/HIGH finding is accepted for the advertised
O3K or supported OpenStack profile.

Validation on the development host:

```text
./tests/security-release-gate.sh                 PASS
cargo audit                                      PASS (0 advisories)
cargo deny check licenses bans sources            PASS (warnings only for duplicate crates/workspace resolver)
pnpm audit --prod                                PASS (no known vulnerabilities)
cargo test --workspace --all-features             PASS
pnpm build                                        PASS
```

Pinned local artifact evidence (development host, 2026-09-13) was generated
with Syft `v1.18.1` (CycloneDX) and Trivy `v0.58.2` using the local OCI
registry. Syft reported 94 BFF, 72 Tenant console and 72 Operator console
components. Trivy reported zero HIGH/CRITICAL findings for both frontend
images, and zero HIGH/CRITICAL findings for the distroless BFF image. The
candidate images are also signed and SBOM-attested in the local registry with
an ephemeral development cosign key; verification is recorded in
`target/security/cosign-verify.txt`. A release still requires the CI-owned
trusted keyless provenance attestation before publication.
