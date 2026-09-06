# P12-IAM closure evidence

Evidence is recorded against protected repository tips:

- O3K Rust: `7d28f9ef61fa9a50604a6f242c2b8d23eb920eb4` (`P12-IAM.8`)
- Araf: `3ff5a17f0ee5e42c6ecab1643d4456c415435687` (`P12-IAM.8`)

## Slice status

P12-IAM.0 through P12-IAM.7 are accepted and their evidence remains linked
from the O3K issue history. P12-IAM.8 implementation is merged in both
repositories, but the aggregate is not closed because the required real Araf
BFF-to-real-O3K browser journey has not been executed.

## Real IdP profile

The provider gate uses Keycloak `25.0.6` pinned to digest
`sha256:82c5b7a110456dbd42b86ea572e728878549954cc8bd03cd65410d75328095d2`.
P12-IAM.7 real federation passed against both the SQLite and PostgreSQL
identity paths. The provider is test evidence only; Araf does not depend on
Keycloak-specific types.

## Contract and implementation evidence

- O3K machine-readable contract: `contracts/openapi/native-iam.yaml`
- O3K route: `POST /o3k/v1/identity/scopes`
- O3K implementation: `crates/o3k-native-api`, `bins/o3kd/src/native_adapters/token.rs`
- Araf client and session boundary: `backend/console-bff-core/src/auth.rs`,
  `session.rs`, `middleware.rs`, and `o3k_client.rs`
- Araf uses separate tenant/operator OIDC configuration and opaque cookie
  namespaces. External and native tokens are server-side session fields only.
- CSRF accepts only exact `araf_tenant_session` or
  `araf_operator_session` cookies; the readable CSRF cookie cannot be
  mistaken for a session.

## Executed gates

- `cargo test -p o3k-native-api --all-features` — PASS (35 tests)
- `cargo test -p o3kd --all-features` — PASS
- `cargo clippy -p o3k-native-api -p o3kd --all-targets --all-features -- -D warnings` — PASS
- `bash tests/native-iam-contract.sh` — PASS
- `bash tests/p12-iam-7-real-idp.sh` — PASS
- Araf `cargo test --workspace --all-features` — PASS (20 unit, 61 contract, 7 adapter)
- Araf `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS
- Araf production callback custody test — PASS; opaque cookies contain no
  external or native credential, and the server session contains the tokens.
- Protected O3K CI and Araf CI — PASS before merge.

## Tenant/operator evidence

Software evidence proves separate configuration, cookie names, session state,
CSRF, server-side token custody, session expiry/logout behavior, O3K scope
discovery request shape, selected-session native-token use, cross-scope
negative tests, and operator authorization fail-closed behavior. P12-IAM.7
proves real provider validation, project assignment, rescoping, and system
operator authorization in O3K.

The missing proof is an end-to-end run through a real Araf BFF process and
browser/network trace against a live O3K HTTP process using the real provider.
The current Araf callback test uses a deterministic HTTP test double and is not
represented as real-host evidence.

## Restart and key rotation

O3K restart, SQLite/PostgreSQL persistence, validator rejection, and provider
key/issuer/audience failure evidence are covered by the P12-IAM.7 and prior
identity gates. Araf currently uses an in-memory session store: restart logs
out sessions, and multi-replica session sharing is not claimed.

## Findings and classifications

- **BLOCKER:** Real Araf BFF/browser → real O3K federation journey, including
  real scope selection and a real tenant resource request, is not yet executed.
- **ACCEPTED BOUNDED DEVIATION:** Araf session storage is in-memory and only
  single-instance evidence is claimed; a durable shared store is required for
  HA production claims.
- **ACCEPTED BOUNDED DEVIATION:** Refresh-token renewal is not implemented;
  external token expiry terminates the Araf session.
- **No known unresolved HIGH finding** in the merged implementation. O3K-hosted
  identity is not advertised by the supported profile.

## Final verdict

P12-IAM aggregate verdict: BLOCKED
Araf production identity unblocked: NO
