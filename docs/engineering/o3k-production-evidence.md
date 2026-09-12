# Araf P2.8 native O3K production evidence

## Identity

- Issue: #51
- Base main: `48e539f837e4fa222138d78791f298d8b7baec99`
- Branch: `p2-8-o3k-production-gate`
- Araf implementation commit: recorded by `tests/p2-8-production-gate.sh`
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

## Current host attempt

The host contains KVM/libvirt tooling and the O3K disposable TestLab scripts,
but no running production deployment, trusted TLS ingress, standards-based
IdP, or configured real provider. The available P12/P2 process scripts start
`O3K_PROVIDER=fake` and use loopback HTTP with explicit test-profile settings.
Those scripts remain valid deterministic/process tests for their respective
issues, but are intentionally rejected by the P2.8 wrapper. No production
journey, credential-custody proof, cross-project isolation run, or operator
production run is claimed from them.

The wrapper was exercised without a deployment harness and failed closed
because `ARAF_P2_8_HARNESS` was not configured. No fixture data was rendered
and no credentials were emitted.

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

`NO-GO`

