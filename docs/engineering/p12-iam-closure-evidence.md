# P12-IAM closure evidence

Evidence is recorded against protected repository tips:

- O3K Rust: `e47a81861f1f2c5dac125d017366a959dca39d1b` (`P12-IAM.8`)
- Araf: `01b234ba228b0dccfbbb794819933f5d48f97a31` (`P12-IAM.8`)

## Slice status

P12-IAM.0 through P12-IAM.7 are accepted and their evidence remains linked
from the O3K issue history. P12-IAM.8 implementation is merged in both
repositories and the required real Araf BFF-to-real-O3K browser journey has
passed.

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
- `O3K_P12_7_AFTER_HOOK=tests/p12-iam-8-real-araf-process.sh bash tests/p12-iam-7-real-idp.sh` — PASS; real Keycloak browser login, Araf callback/session, O3K scope discovery, scope selection, context/resource request, token-custody check, and logout
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

The real-process hook provides the required end-to-end proof through a real
Araf BFF process and browser-like network flow against a live O3K HTTP process
using the pinned Keycloak provider. The deterministic callback test remains
useful unit evidence but is not used as the real-host claim.

## Restart and key rotation

O3K restart, SQLite/PostgreSQL persistence, validator rejection, and provider
key/issuer/audience failure evidence are covered by the P12-IAM.7 and prior
identity gates. Araf currently uses an in-memory session store: restart logs
out sessions, and multi-replica session sharing is not claimed.

## Findings and classifications

- **ACCEPTED BOUNDED DEVIATION:** Araf session storage is in-memory and only
  single-instance evidence is claimed; a durable shared store is required for
  HA production claims.
- **ACCEPTED BOUNDED DEVIATION:** Refresh-token renewal is not implemented;
  external token expiry terminates the Araf session.
- **No known unresolved HIGH finding** in the merged implementation. O3K-hosted
  identity is not advertised by the supported profile.

## Final verdict

P12-IAM aggregate verdict: PASS
Araf production identity unblocked: YES
