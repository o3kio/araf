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
const ALLOWED_ACTION_KEYS = new Set(["id", "name", "requiresConfirmation"]);

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
  }
}
