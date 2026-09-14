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

## Manual release security checklist

The automated gate is supplemented by a release-owner review of the deployed
candidate. Each item must be recorded as pass, fail, or not applicable before
publication:

- verify TLS termination, certificate coverage, HSTS, and secure cookie flags on
  both Tenant and Operator origins;
- exercise login, callback, logout, expiry, CSRF failure, and cross-origin
  requests through the production ingress;
- attempt tenant-to-tenant and tenant-to-operator access using guessed project
  and resource identifiers;
- submit oversized bodies, malformed JSON, invalid descriptor/schema content,
  unsafe URLs, redirect targets, and unexpected methods;
- inspect browser, BFF, ingress, and upstream logs for tokens, cookies, keys,
  credentials, or sensitive request bodies;
- verify the published image digest, SBOM, and keyless provenance attestation
  match the release commit; and
- verify BFF and console containers run without root and have only the
  filesystem/network access required by their deployment profile.

The current candidate has no unresolved BLOCKER or HIGH finding for the
advertised O3K or supported OpenStack surfaces. The deferred multi-node O3K
deployment certification, including the delete `409`/`202` contract question,
is tracked in #106 and is not represented as a P4.4 security claim.
