# @araf/schema-runtime

JSON Schema 2020-12 validation wrapper used by the Araf generic resource
runtime for schema-driven create and action forms.

## API

- `createSchemaValidator(schema)` — compiles a schema and returns a synchronous
  `{ validate(data) }` object.
- `validateFormData(schema, data)` — convenience helper returning a flat array
  of `ValidationError` objects.
- `ValidationError` — `{ instancePath, schemaPath, message, keyword }`.

## M5 limitations

To avoid arbitrary remote code execution or network requests, the validator
rejects schemas that contain:

- `$ref` pointing to non-local URLs (anything not starting with `#`).
- `discriminator`.
- `$dynamicRef`, `$dynamicAnchor`, `$recursiveRef`, `$recursiveAnchor`.

Schemas with these features fail every validation with a single
`unsupportedSchemaFeature` error. No remote schema fetching is performed.
