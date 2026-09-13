# Araf P2.8 native O3K production evidence

## Identity

- Issue: #51
- Base main: `54df38bd6a117c945fb68221729427f9b4275236`
- Branch: deployment-owned evidence run (harness outside product path)
- Araf implementation commit: `54df38bd6a117c945fb68221729427f9b4275236`
- O3K convergence: `21fe687c387a04f107b6e87fac04060b1c28e449` (#907/#928)

## Gate contract

P2.8 is an integrated production-profile gate. The deployment-owned harness
must run the Tenant and Operator BFFs against a real O3K deployment, a real
supported provider, and a standards-based IdP. The harness receives no
browser bearer token and must write only redacted artifacts. A fake O3K
provider, Araf fixture adapter, insecure HTTP exception, or direct handler
invocation is not production evidence.

`tests/p2-8-production-gate.sh` enforces the preflight boundary. It requires
an executable `ARAF_P2_8_HARNESS`, a provider other than `fake`/`fixture`, and
HTTPS URLs for O3K, both BFFs, and OIDC discovery. It passes only the
non-secret endpoint metadata to the harness and records a redacted result
under `ARAF_P2_8_EVIDENCE_DIR`.
The harness must also create a non-symlink `harness.success` marker containing
exactly `P2_8_HARNESS_PASS=1`; without that redacted attestation the wrapper
cannot emit a successful result.

## Deployment-owned production run

The host run used a real O3K `o3kd` at convergence commit
`21fe687c387a04f107b6e87fac04060b1c28e449`, the supported `agent` provider
(`o3k-compute` 0.4.0-alpha.1), Keycloak 25.0.6, and nginx 1.24.0 TLS ingress.
Tenant and Operator BFFs both ran Araf commit
`54df38bd6a117c945fb68221729427f9b4275236` in the production profile. HTTPS
endpoints were `https://127.0.0.1:8444` (O3K), `:8445` (Tenant BFF), `:8446`
(Operator BFF), and `:8443/realms/araf-p28/.well-known/openid-configuration`
(OIDC); the deployment CA was trusted by the harness.

The executable deployment harness was
`/tmp/araf-p2-8-deployment-final/harness.py`. It received only the wrapper's
sanitized endpoint metadata; credentials stayed in a mode-700 deployment
secret file and were never written to evidence. Redacted artifacts are under
`/tmp/araf-p2-8-evidence-final-1789293594` (deployment-local, not committed):
`harness.redacted.json`, `harness.success`, and `result.env`.

The unchanged `tests/p2-8-production-gate.sh` returned `GO`. Its 71 assertions
covered OIDC login/callback/logout, canonical identity and scope selection,
Compute and Network lifecycle/Operations/reload convergence, native Image
upload lifecycle, truthful Volume `not_ready`, unsupported-action hiding,
Project-A/Project-B list/show/delete/Operation isolation, Operator BFF
separation, secure HttpOnly/SameSite cookies, CSRF/CSP/security headers,
credential-free BFF payloads, bounded collections, a real stale-port provider
failure ending in a structured failed Operation, and process-level absence of
fixture/fake configuration.

## Required deployment evidence

The deployment run must attach redacted, immutable artifacts proving:

- tenant OIDC discovery, callback, scope selection, `/identity/me`, logout;
- Compute, Network, Volume and Image journeys supported by discovery;
- canonical Operation IDs and post-reload resource convergence;
- a real provider failure with a failed Operation and structured correlation;
- Project-A to Project-B denial for list, show, mutation and Operation reads;
- separate Operator BFF/system context and tenant-to-operator denial;
- HTTPS proxy, secure HttpOnly cookies, CSRF and CSP checks;
- browser/network/storage/log scans with no reusable IdP or O3K credential;
- bounded resource and Operation pagination;
- production configuration fail-closed checks and no fixture/placeholder path.

The artifact must identify exact O3K, provider, IdP, BFF and TLS versions and
must not include access tokens, refresh tokens, client secrets or session
identifiers.

## Scope boundary

This workstream adds no O3K API and does not implement the Operations Center,
governance, diagnostics or metering UX. O3K remains semantic authority;
Araf's presentation metadata remains non-authoritative. Any future contract
problem must first be classified as an Araf defect, an incorrect assumption,
or an adaptation to the frozen O3K contract; only a reproducible violation of
that contract can be escalated upstream.

## Verdict

`GO — O3K PRODUCTION CONVERGED`
