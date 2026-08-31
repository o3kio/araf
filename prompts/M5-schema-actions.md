# M5 implementation prompt — schema-driven create/edit/action flows

Implement **M5** after M4.

## Read first

- `AGENTS.md`
- `docs/architecture/resource-runtime.md`
- `docs/product/information-architecture.md`
- `docs/security/threat-model.md`

## Goal

Prove that ordinary cloud mutations can be rendered from JSON Schema plus bounded presentation metadata without turning descriptors into executable programs.

## Required implementation

1. Implement `packages/schema-runtime` using JSON Schema 2020-12 validation with a reviewed validator such as Ajv.
2. Keep validation/data schema separate from Araf presentation metadata.
3. Support MVP form needs:
   - text/number/boolean,
   - enum/select,
   - required/optional,
   - bounded arrays only where needed,
   - sections,
   - basic vs Advanced grouping,
   - field help/error text,
   - safe conditional visibility based on local form values/capabilities.
4. Implement generic action rendering from stable action IDs, required effective capability and risk class.
5. Implement destructive/disruptive confirmations using authoritative resource/relationship data when available.
6. Action submission must return/transition to a canonical Operation contract.
7. Prove at least one complete schema-generated Create flow and one non-create action.
8. Server fixture must independently validate input and capability; client validation is UX only.
9. Reject descriptor features that attempt to inject JavaScript, raw HTML or arbitrary network URLs.
10. Add schema compatibility/version tests.

## UX requirements

The default create form must be short and safe. Advanced provider-neutral fields are progressively disclosed rather than placed on the first screen.

## Non-goals

- complex visual topology editors,
- service-specific form code unless an explicit generic-runtime limitation is documented,
- preflight API implementation; leave a clean future hook for authoritative preflight results.

## Acceptance

- create flow is generated from schema/presentation metadata,
- server rejects invalid/unauthorized submissions,
- action produces an Operation,
- Advanced grouping works without provider-specific leakage,
- no arbitrary executable descriptor capability exists,
- keyboard/form error behavior is accessible.

Branch suggestion: `m5-schema-actions`.
