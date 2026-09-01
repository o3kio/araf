/**
 * Versioned console presentation descriptor for the generic resource runtime.
 *
 * These shapes are intentionally presentation-only. They do not redefine O3K
 * cloud semantics; they mirror the BFF's `ServiceDescriptor`/`ResourceTypeDescriptor`
 * and are validated strictly so that unknown descriptor fields fail visibly in
 * development rather than silently executing behavior.
 */

import type {
  ColumnDescriptor,
  FilterDescriptor,
  DetailsSectionDescriptor,
  RelationshipDescriptor,
  ActionDescriptor,
} from "@araf/api-client";

export type {
  ColumnDescriptor,
  FilterDescriptor,
  DetailsSectionDescriptor,
  RelationshipDescriptor,
  ActionDescriptor,
};

export interface ResourceDescriptor {
  /** Stable resource type identifier, e.g. `compute.server`. */
  readonly id: string;
  /** Singular display name. */
  readonly name: string;
  /** Plural display name. */
  readonly pluralName: string;
  /** Icon token (not an arbitrary URL or script). */
  readonly iconToken: string;
  /** Actions the resource supports in this surface. */
  readonly supportedActions: readonly ActionDescriptor[];
  /** Columns rendered in the collection table. */
  readonly columns: readonly ColumnDescriptor[];
  /** Filters exposed in the collection page. */
  readonly filters: readonly FilterDescriptor[];
  /** Fields the server can sort by. */
  readonly sortableFields: readonly string[];
  /** Sections rendered on the detail page. */
  readonly detailsSections: readonly DetailsSectionDescriptor[];
  /** Relationships to other resource types. */
  readonly relationships: readonly RelationshipDescriptor[];
  /** JSON Schema describing the create payload, if creation is supported. */
  readonly createSchema?: unknown;
  /** Capability required to create this resource type. */
  readonly createCapability: {
    readonly resourceType: string;
    readonly action: string;
  };
}

const ALLOWED_DESCRIPTOR_KEYS = new Set([
  "id",
  "name",
  "pluralName",
  "iconToken",
  "supportedActions",
  "columns",
  "filters",
  "sortableFields",
  "detailsSections",
  "relationships",
  "createSchema",
  "createCapability",
]);

const ALLOWED_COLUMN_KEYS = new Set(["id", "header", "field", "width"]);
const ALLOWED_FILTER_KEYS = new Set(["id", "label", "field", "kind"]);
const ALLOWED_SECTION_KEYS = new Set(["id", "label", "fields"]);
const ALLOWED_RELATIONSHIP_KEYS = new Set([
  "id",
  "targetResourceType",
  "label",
  "sourcePropertyKey",
  "direction",
]);
const ALLOWED_ACTION_KEYS = new Set([
  "id",
  "name",
  "requiresConfirmation",
  "riskClass",
  "requiredCapability",
  "inputSchema",
]);
const ALLOWED_CAPABILITY_KEYS = new Set(["resourceType", "action"]);

const RISK_CLASSES = new Set(["normal", "disruptive", "destructive", "privileged"]);

const EXECUTABLE_STRING_PATTERNS = [/^javascript:/iu, /<script/iu, /^data:text\/html/iu];

const FORBIDDEN_SCHEMA_KEYS = new Set(["$exec", "x-araf-script", "eval"]);
const FORBIDDEN_SCHEMA_STRINGS = new Set(["eval", "Function("]);

function assertPlainObject(
  value: unknown,
  _path: string,
): asserts value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`Invalid descriptor at ${_path}: expected object, got ${typeof value}`);
  }
}

function checkUnknownKeys(
  value: Record<string, unknown>,
  allowed: Set<string>,
  path: string,
): void {
  const unknown = Object.keys(value).filter((key) => !allowed.has(key));
  if (unknown.length > 0) {
    throw new Error(
      `Unsupported descriptor field(s) at ${path}: ${unknown.join(", ")}. ` +
        `This runtime version only supports: ${[...allowed].join(", ")}.`,
    );
  }
}

function validateArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * Recursively scan a descriptor value for executable-looking strings and
 * schema keys that could carry code. Throws as soon as a dangerous value is
 * found. This is defense-in-depth; the BFF also rejects dangerous descriptors.
 */
function rejectExecutableValues(value: unknown, path: string): void {
  if (typeof value === "string") {
    for (const pattern of EXECUTABLE_STRING_PATTERNS) {
      if (pattern.test(value)) {
        throw new Error(`Dangerous descriptor value at ${path}: executable string rejected`);
      }
    }
    return;
  }

  if (Array.isArray(value)) {
    for (const [index, item] of value.entries()) {
      rejectExecutableValues(item, `${path}[${String(index)}]`);
    }
    return;
  }

  if (isPlainObject(value)) {
    for (const [key, child] of Object.entries(value)) {
      if (FORBIDDEN_SCHEMA_KEYS.has(key)) {
        throw new Error(`Dangerous schema key at ${path}.${key}: ${key} is not allowed`);
      }
      rejectExecutableValues(child, `${path}.${key}`);
    }
  }
}

/**
 * Scan a JSON Schema value for dangerous keys/strings. Separated from the
 * general descriptor scan because schemas have their own key allow-list.
 */
function rejectDangerousSchema(schema: unknown, path: string): void {
  if (typeof schema === "string") {
    if (FORBIDDEN_SCHEMA_STRINGS.has(schema)) {
      throw new Error(`Dangerous schema value at ${path}: "${schema}" is not allowed`);
    }
    return;
  }

  if (Array.isArray(schema)) {
    for (const [index, item] of schema.entries()) {
      rejectDangerousSchema(item, `${path}[${String(index)}]`);
    }
    return;
  }

  if (isPlainObject(schema)) {
    for (const [key, value] of Object.entries(schema)) {
      if (FORBIDDEN_SCHEMA_KEYS.has(key)) {
        throw new Error(`Dangerous schema key at ${path}.${key}: ${key} is not allowed`);
      }
      rejectDangerousSchema(value, `${path}.${key}`);
    }
  }
}

function validateCapability(
  value: unknown,
  path: string,
): { readonly resourceType: string; readonly action: string } {
  assertPlainObject(value, path);
  checkUnknownKeys(value, ALLOWED_CAPABILITY_KEYS, path);
  if (typeof value.resourceType !== "string") {
    throw new Error(`${path}.resourceType must be a string`);
  }
  if (typeof value.action !== "string") {
    throw new Error(`${path}.action must be a string`);
  }
  return {
    resourceType: value.resourceType,
    action: value.action,
  };
}

function validateSchemaValue(value: unknown, path: string): void {
  if (value === undefined || value === null) {
    return;
  }
  if (!isPlainObject(value)) {
    throw new Error(`${path} must be an object, null, or undefined`);
  }
  rejectDangerousSchema(value, path);
}

/**
 * Validate a runtime descriptor in development.
 *
 * Throws a clear error for unknown/unsupported fields so that descriptor
 * evolution is explicit and visible. In production this should be a no-op
 * to avoid breaking releases that receive descriptors from a newer BFF.
 */
export function validateDescriptor(descriptor: unknown): asserts descriptor is ResourceDescriptor {
  if (typeof descriptor !== "object" || descriptor === null || Array.isArray(descriptor)) {
    throw new Error("Descriptor must be a plain object");
  }

  const d = descriptor as Record<string, unknown>;
  checkUnknownKeys(d, ALLOWED_DESCRIPTOR_KEYS, "descriptor");
  rejectExecutableValues(descriptor, "descriptor");

  for (const key of ["id", "name", "pluralName", "iconToken"] as const) {
    if (typeof d[key] !== "string") {
      throw new Error(`Descriptor.${key} must be a string`);
    }
  }

  for (const key of [
    "supportedActions",
    "columns",
    "filters",
    "sortableFields",
    "detailsSections",
    "relationships",
  ] as const) {
    if (!validateArray(d[key])) {
      throw new Error(`Descriptor.${key} must be an array`);
    }
  }

  if ("createCapability" in d) {
    validateCapability(d.createCapability, "descriptor.createCapability");
  } else {
    throw new Error("Descriptor.createCapability is required");
  }

  if ("createSchema" in d) {
    validateSchemaValue(d.createSchema, "descriptor.createSchema");
  }

  for (const [index, column] of (d.columns as unknown[]).entries()) {
    assertPlainObject(column, `descriptor.columns[${String(index)}]`);
    checkUnknownKeys(column, ALLOWED_COLUMN_KEYS, `descriptor.columns[${String(index)}]`);
    for (const key of ["id", "header", "field"] as const) {
      if (typeof column[key] !== "string") {
        throw new Error(`descriptor.columns[${String(index)}].${key} must be a string`);
      }
    }
    if (column.width !== undefined && typeof column.width !== "string") {
      throw new Error(`descriptor.columns[${String(index)}].width must be a string`);
    }
  }

  for (const [index, filter] of (d.filters as unknown[]).entries()) {
    assertPlainObject(filter, `descriptor.filters[${String(index)}]`);
    checkUnknownKeys(filter, ALLOWED_FILTER_KEYS, `descriptor.filters[${String(index)}]`);
    for (const key of ["id", "label", "field", "kind"] as const) {
      if (typeof filter[key] !== "string") {
        throw new Error(`descriptor.filters[${String(index)}].${key} must be a string`);
      }
    }
    if (!["text", "select"].includes(filter.kind as string)) {
      throw new Error(`descriptor.filters[${String(index)}].kind must be "text" or "select"`);
    }
  }

  for (const [index, section] of (d.detailsSections as unknown[]).entries()) {
    assertPlainObject(section, `descriptor.detailsSections[${String(index)}]`);
    checkUnknownKeys(section, ALLOWED_SECTION_KEYS, `descriptor.detailsSections[${String(index)}]`);
    for (const key of ["id", "label"] as const) {
      if (typeof section[key] !== "string") {
        throw new Error(`descriptor.detailsSections[${String(index)}].${key} must be a string`);
      }
    }
    if (!validateArray(section.fields)) {
      throw new Error(`descriptor.detailsSections[${String(index)}].fields must be an array`);
    }
    for (const [fieldIndex, field] of section.fields.entries()) {
      if (typeof field !== "string") {
        throw new Error(
          `descriptor.detailsSections[${String(index)}].fields[${String(fieldIndex)}] must be a string`,
        );
      }
    }
  }

  for (const [index, relationship] of (d.relationships as unknown[]).entries()) {
    assertPlainObject(relationship, `descriptor.relationships[${String(index)}]`);
    checkUnknownKeys(
      relationship,
      ALLOWED_RELATIONSHIP_KEYS,
      `descriptor.relationships[${String(index)}]`,
    );
    for (const key of ["id", "targetResourceType", "label", "sourcePropertyKey"] as const) {
      if (typeof relationship[key] !== "string") {
        throw new Error(`descriptor.relationships[${String(index)}].${key} must be a string`);
      }
    }
    if (!["to-one", "to-many"].includes(relationship.direction as string)) {
      throw new Error(
        `descriptor.relationships[${String(index)}].direction must be "to-one" or "to-many"`,
      );
    }
  }

  for (const [index, action] of (d.supportedActions as unknown[]).entries()) {
    assertPlainObject(action, `descriptor.supportedActions[${String(index)}]`);
    checkUnknownKeys(action, ALLOWED_ACTION_KEYS, `descriptor.supportedActions[${String(index)}]`);
    for (const key of ["id", "name"] as const) {
      if (typeof action[key] !== "string") {
        throw new Error(`descriptor.supportedActions[${String(index)}].${key} must be a string`);
      }
    }
    if (typeof action.requiresConfirmation !== "boolean") {
      throw new Error(
        `descriptor.supportedActions[${String(index)}].requiresConfirmation must be a boolean`,
      );
    }
    if (typeof action.riskClass !== "string" || !RISK_CLASSES.has(action.riskClass)) {
      throw new Error(
        `descriptor.supportedActions[${String(index)}].riskClass must be one of: ${[
          ...RISK_CLASSES,
        ].join(", ")}`,
      );
    }
    validateCapability(
      action.requiredCapability,
      `descriptor.supportedActions[${String(index)}].requiredCapability`,
    );
    if ("inputSchema" in action) {
      validateSchemaValue(
        action.inputSchema,
        `descriptor.supportedActions[${String(index)}].inputSchema`,
      );
    }
  }
}
