# O3K integration contract

## 1. Authority

Araf consumes O3K. It does not define O3K.

The O3K repository and published native/compatibility contracts remain authoritative for resource identity, scope, lifecycle, authorization, operation semantics and provider behavior.

The planning baseline was cross-checked against the active O3K development line on 2026-08-31, including the P13.5A IaC convergence work and the established provider-neutral mappings for Compute, Network, Volume and related resources. Those details are evidence for the direction, not permission for Araf to freeze undocumented endpoints.

## 2. No invented production APIs

When an Araf feature needs an upstream capability:

1. inspect the current O3K native API/specification,
2. use the documented operation if it exists,
3. if absent, record an explicit upstream dependency/gap,
4. keep fixture behavior behind a fixture adapter,
5. do not create a production BFF endpoint that silently becomes a second cloud-control API.

A BFF endpoint may aggregate presentation data, manage sessions or translate transport shape. It must not invent authoritative resource state or permission decisions.

## 3. Adapter boundary

Frontend packages consume an Araf client contract that is generated or mechanically derived from authoritative schemas where possible.

```text
UI/runtime
   -> typed Araf client
      -> Tenant/Operator BFF
         -> O3K native API
```

Fixture mode uses the same client contract:

```text
UI/runtime
   -> typed Araf client
      -> fixture adapter
```

Fixture and production modes must not be mixed implicitly.

## 4. Representative MVP resources

The MVP prioritizes the native O3K equivalents of:

- Compute Server
- Network
- Endpoint/Port when required by the native model
- Volume
- Image/read-only image selection where supported
- Operation
- Project/scope identity
- Quota/usage/audit capabilities that exist upstream

OpenStack compatibility names such as Nova, Neutron, Cinder and Glance must not become Araf domain nouns.

## 5. Resource identity

The UI must treat resource IDs as opaque stable identifiers. Names are display/search attributes, not identity.

Resource URLs should remain bookmarkable and scope-safe. The BFF/API must reject cross-scope access even if a valid opaque identifier is guessed.

## 6. Pagination/filtering

Collection integration must be server bounded. Araf expects contracts capable of:

- page/cursor bounded retrieval,
- server-side filtering,
- server-side sorting where supported,
- stable total/continuation behavior where the upstream API provides it.

If a current O3K endpoint lacks scalable collection semantics, document the upstream gap; do not solve it by downloading the complete inventory into the browser.

## 7. Operations

For asynchronous mutations, the response path must preserve the canonical O3K Operation identity.

Araf requires enough data to display:

- operation ID,
- action/type,
- state,
- requested scope/resource,
- timestamps,
- structured failure reason/detail,
- related resources/relationships when available,
- correlation/request identifiers where available.

Polling may be used initially if O3K does not yet expose an event/SSE contract. Araf must keep the runtime abstract enough to replace polling with streaming without changing resource pages.

## 8. Problem Details and errors

Use upstream Problem Details/structured errors where available. Preserve:

- stable machine-readable type/code,
- human-safe title/detail,
- correlation identifier,
- operation/resource linkage,
- retryability only when the server explicitly establishes it.

Do not parse human error strings to make authorization or retry decisions.

## 9. Capabilities

The UI may hide or disable actions based on effective capabilities supplied by the authenticated server context/resource response.

Araf must not infer permission from role names alone.

Server authorization is mandatory on every authoritative operation.

## 10. Compatibility and migration

OpenStack API/Terraform compatibility is strategically important to O3K, but Araf remains native-first.

Future migration UX may translate OpenStack inventory into native O3K resources. That future feature must be implemented as an explicit compatibility/migration module and must not force OpenStack service topology into every native screen.

## 11. Upstream dependency log

During implementation, unresolved required upstream gaps should be added to `docs/engineering/upstream-gaps.md` with:

- Araf issue/code,
- desired user capability,
- missing O3K contract,
- why a frontend workaround would be unsafe/incorrect,
- proposed upstream interface direction without pretending it is already approved,
- blocking/non-blocking status.
