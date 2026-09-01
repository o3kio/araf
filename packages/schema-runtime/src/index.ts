/**
 * @araf/schema-runtime — JSON Schema 2020-12 validation wrapper.
 *
 * Uses Ajv for local, synchronous schema validation. Schemas that rely on
 * remote references or dynamic/discriminator features are rejected at compile
 * time rather than fetched from the network, because M5 must not allow
 * arbitrary remote code or URLs.
 */

import Ajv from "ajv";
import addFormats from "ajv-formats";

export interface ValidationError {
  readonly instancePath: string;
  readonly schemaPath: string;
  readonly message: string;
  readonly keyword: string;
  readonly params?: Record<string, unknown>;
}

export interface ValidResult {
  readonly valid: true;
}

export interface InvalidResult {
  readonly valid: false;
  readonly errors: readonly ValidationError[];
}

export type ValidationResult = ValidResult | InvalidResult;

export interface SchemaValidator {
  readonly validate: (data: unknown) => ValidationResult;
}

const FORBIDDEN_SCHEMA_KEYWORDS = new Set([
  "$dynamicRef",
  "$dynamicAnchor",
  "$recursiveRef",
  "$recursiveAnchor",
  "discriminator",
]);

const REMOTE_REF_PROTOCOLS = ["http:", "https:"];

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isNonLocalRef(value: unknown): boolean {
  if (typeof value !== "string") return false;
  if (value.startsWith("#")) return false;
  for (const protocol of REMOTE_REF_PROTOCOLS) {
    if (value.startsWith(protocol)) return true;
  }
  return false;
}

/**
 * Recursively scan a schema for features that M5 refuses to support:
 * remote `$ref` URLs and dynamic/discriminator keywords.
 *
 * Returns the first problem found, or `undefined` if the schema is acceptable.
 */
function findUnsupportedSchemaFeature(
  schema: unknown,
  path: string,
): { path: string; feature: string } | undefined {
  if (Array.isArray(schema)) {
    for (const [index, item] of schema.entries()) {
      const found = findUnsupportedSchemaFeature(item, `${path}[${String(index)}]`);
      if (found) return found;
    }
    return undefined;
  }

  if (!isPlainObject(schema)) {
    return undefined;
  }

  for (const [key, value] of Object.entries(schema)) {
    if (FORBIDDEN_SCHEMA_KEYWORDS.has(key)) {
      return { path: `${path}.${key}`, feature: key };
    }
    if (key === "$ref" && isNonLocalRef(value)) {
      return { path: `${path}.$ref`, feature: "remote $ref" };
    }
    if (isPlainObject(value) || Array.isArray(value)) {
      const found = findUnsupportedSchemaFeature(value, `${path}.${key}`);
      if (found) return found;
    }
  }

  return undefined;
}

function formatErrors(errors: Ajv["errors"]): ValidationError[] {
  if (!errors) return [];
  return errors.map((error) => ({
    instancePath: error.instancePath,
    schemaPath: error.schemaPath,
    message: error.message ?? "invalid value",
    keyword: error.keyword,
    params: isPlainObject(error.params) ? error.params : undefined,
  }));
}

/**
 * Create a synchronous, local JSON Schema 2020-12 validator.
 *
 * Limitations (M5):
 * - Schemas containing `$ref` to non-local URLs are rejected.
 * - `discriminator`, `$dynamicRef`, `$dynamicAnchor`, `$recursiveRef`, and
 *   `$recursiveAnchor` are rejected.
 * - No remote schema fetching is performed.
 */
export function createSchemaValidator(schema: unknown): SchemaValidator {
  const unsupported = findUnsupportedSchemaFeature(schema, "schema");
  if (unsupported) {
    return {
      validate: () => ({
        valid: false,
        errors: [
          {
            instancePath: "",
            schemaPath: unsupported.path,
            message: `Unsupported schema feature: ${unsupported.feature}. Remote references and dynamic keywords are disabled.`,
            keyword: "unsupportedSchemaFeature",
          },
        ],
      }),
    };
  }

  const ajv = new Ajv({ strict: true, allErrors: true });
  ajv.addKeyword("x-araf");
  addFormats(ajv);

  let validateFn: ReturnType<typeof ajv.compile>;
  try {
    validateFn = ajv.compile(schema as object);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return {
      validate: () => ({
        valid: false,
        errors: [
          {
            instancePath: "",
            schemaPath: "",
            message: `Schema compilation failed: ${message}`,
            keyword: "schemaCompilation",
          },
        ],
      }),
    };
  }

  return {
    validate: (data: unknown): ValidationResult => {
      const valid = validateFn(data);
      if (valid) {
        return { valid: true };
      }
      return { valid: false, errors: formatErrors(validateFn.errors) };
    },
  };
}

/**
 * Convenience helper that validates form data against a schema and returns
 * any validation errors. Empty array means the data is valid.
 */
export function validateFormData(schema: unknown, data: unknown): ValidationError[] {
  const result = createSchemaValidator(schema).validate(data);
  return result.valid ? [] : [...result.errors];
}
