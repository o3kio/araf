# M3 implementation prompt — Rust BFF boundary and deterministic O3K contract harness

Implement **M3** after M0; it may proceed in parallel with M1/M2 where dependencies permit.

## Read first

- `AGENTS.md`
- `docs/architecture/o3k-integration-contract.md`
- `docs/security/threat-model.md`
- ADR 0002
- `docs/engineering/quality-gates.md`

## Goal

Establish the server/client boundary that the prototype can use with deterministic fixtures and the MVP can later bind to real O3K without rewriting frontend semantics.

## Required implementation

1. Implement shared BFF core primitives for:
   - correlation/request IDs,
   - structured JSON errors/Problem Details passthrough shape,
   - strict route/upstream abstraction,
   - request/body limits,
   - safe structured logging/redaction hooks.
2. Keep Tenant BFF and Operator BFF as separate binaries/route registries.
3. Define a typed frontend client contract for prototype needs:
   - current context/capabilities fixture,
   - resource descriptor discovery fixture,
   - paginated resource collection fixture,
   - resource detail fixture,
   - action submission returning an Operation,
   - Operation list/detail.
4. Implement a **deterministic fixture adapter** behind the BFF. It must be explicitly configured and impossible to mistake for a production O3K adapter.
5. Fixture collections must support server-bounded pagination/filtering and a total >=100,000 without returning all records.
6. Add contract tests proving frontend client/BFF fixture agreement.
7. Add a placeholder production adapter trait/interface with no fabricated O3K routes.
8. Create `docs/engineering/upstream-gaps.md` and document any production contract needed but not yet confirmed upstream.

## Security constraints

- do not implement browser bearer-token storage,
- do not build a generic URL proxy,
- do not make fixture capabilities client-only; fixture server routes must enforce them too,
- tenant BFF cannot expose operator fixture routes.

## Non-goals

- real OIDC (M12 finalizes production auth; an earlier issue may establish it if needed for real integration),
- real O3K write integration,
- SSE if the fixture can begin with bounded polling; keep the Operation source abstraction stream-ready.

## Acceptance

- both BFFs expose independent fixture APIs,
- 100k-total collection proof remains bounded per request,
- action returns Operation rather than fake synchronous success,
- cross-scope/route negative fixture tests exist,
- frontend API contract is typed,
- no undocumented O3K endpoint has been invented.

Branch suggestion: `m3-bff-contract-harness`.
