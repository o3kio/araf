import { describe, expect, it } from "vitest";
import { createSchemaValidator, validateFormData } from "./index.js";

const serverCreateSchema = {
  type: "object",
  required: ["name", "regionId", "projectId"],
  properties: {
    name: { type: "string", minLength: 1 },
    regionId: { enum: ["eu-west", "us-east", "ap-south"] },
    projectId: { enum: ["project-1", "project-2", "project-3", "project-4", "project-5"] },
    bootVolumeSizeGb: { type: "number", minimum: 10 },
  },
};

describe("createSchemaValidator", () => {
  it("returns valid for an object that matches the schema", () => {
    const validator = createSchemaValidator(serverCreateSchema);
    const result = validator.validate({
      name: "web-01",
      regionId: "eu-west",
      projectId: "project-1",
      bootVolumeSizeGb: 50,
    });

    expect(result.valid).toBe(true);
  });

  it("reports a missing required field", () => {
    const validator = createSchemaValidator(serverCreateSchema);
    const result = validator.validate({ regionId: "eu-west", projectId: "project-1" });

    expect(result.valid).toBe(false);
    if (result.valid) throw new Error("expected invalid result");
    expect(result.errors.some((e) => e.keyword === "required" && e.message.includes("name"))).toBe(
      true,
    );
  });

  it("reports a wrong type", () => {
    const validator = createSchemaValidator(serverCreateSchema);
    const result = validator.validate({
      name: "web-01",
      regionId: "eu-west",
      projectId: "project-1",
      bootVolumeSizeGb: "big",
    });

    expect(result.valid).toBe(false);
    if (result.valid) throw new Error("expected invalid result");
    expect(result.errors.some((e) => e.keyword === "type")).toBe(true);
  });

  it("reports an enum violation", () => {
    const validator = createSchemaValidator(serverCreateSchema);
    const result = validator.validate({
      name: "web-01",
      regionId: "mars-north",
      projectId: "project-1",
    });

    expect(result.valid).toBe(false);
    if (result.valid) throw new Error("expected invalid result");
    expect(result.errors.some((e) => e.keyword === "enum")).toBe(true);
  });

  it("reports a numeric minimum violation", () => {
    const validator = createSchemaValidator(serverCreateSchema);
    const result = validator.validate({
      name: "web-01",
      regionId: "eu-west",
      projectId: "project-1",
      bootVolumeSizeGb: 5,
    });

    expect(result.valid).toBe(false);
    if (result.valid) throw new Error("expected invalid result");
    expect(result.errors.some((e) => e.keyword === "minimum")).toBe(true);
  });

  it("rejects a schema containing a remote $ref", () => {
    const schema = {
      type: "object",
      properties: {
        name: { $ref: "https://example.com/schema.json#/definitions/name" },
      },
    };
    const validator = createSchemaValidator(schema);
    const result = validator.validate({ name: "anything" });

    expect(result.valid).toBe(false);
    if (result.valid) throw new Error("expected invalid result");
    expect(result.errors[0]?.keyword).toBe("unsupportedSchemaFeature");
    expect(result.errors[0]?.message).toContain("remote $ref");
  });

  it("rejects a schema containing discriminator", () => {
    const schema = {
      type: "object",
      discriminator: { propertyName: "kind" },
    };
    const validator = createSchemaValidator(schema);
    const result = validator.validate({ kind: "a" });

    expect(result.valid).toBe(false);
    if (result.valid) throw new Error("expected invalid result");
    expect(result.errors[0]?.keyword).toBe("unsupportedSchemaFeature");
  });

  it("allows local $ref fragments", () => {
    const schema = {
      type: "object",
      properties: {
        name: { $ref: "#/$defs/name" },
      },
      $defs: {
        name: { type: "string", minLength: 1 },
      },
    };
    const validator = createSchemaValidator(schema);
    const valid = validator.validate({ name: "ok" });
    const invalid = validator.validate({ name: "" });

    expect(valid.valid).toBe(true);
    expect(invalid.valid).toBe(false);
  });
});

describe("validateFormData", () => {
  it("returns an empty array for valid data", () => {
    const errors = validateFormData(serverCreateSchema, {
      name: "web-01",
      regionId: "eu-west",
      projectId: "project-1",
    });
    expect(errors).toHaveLength(0);
  });

  it("returns errors for invalid data", () => {
    const errors = validateFormData(serverCreateSchema, {
      name: "",
      regionId: "eu-west",
      projectId: "project-1",
    });
    expect(errors.length).toBeGreaterThan(0);
  });
});

// Regression: O3K's native resource contracts publish JSON Schema 2020-12
// (e.g. the compute.server create spec). Ajv's draft-07 default rejects the
// declared dialect, which blocked the tenant-console create form against a
// real O3K deployment (PP.4 / o3kio/o3k#973).
const o3kComputeServerCreateSpec2020 = {
  $schema: "https://json-schema.org/draft/2020-12/schema",
  type: "object",
  required: ["name", "image_id", "flavor_id", "network_ids"],
  properties: {
    name: { type: "string" },
    image_id: { type: "string" },
    flavor_id: { type: "string", format: "uuid" },
    network_ids: { type: "array", items: { type: "string" } },
    key_name: { type: "string" },
    ssh_public_key: { type: "string" },
  },
  additionalProperties: false,
};

describe("createSchemaValidator (JSON Schema 2020-12)", () => {
  it("compiles an O3K-style 2020-12 create schema", () => {
    const validator = createSchemaValidator(o3kComputeServerCreateSpec2020);
    const result = validator.validate({
      name: "pp4-native",
      image_id: "image-1",
      flavor_id: "00000000-0000-0000-0000-000000000001",
      network_ids: ["net-1"],
    });
    expect(result.valid).toBe(true);
    if (!result.valid) throw new Error(JSON.stringify(result.errors));
  });

  it("reports validation errors for a 2020-12 schema", () => {
    const errors = validateFormData(o3kComputeServerCreateSpec2020, { name: "only-name" });
    expect(errors.length).toBeGreaterThan(0);
    expect(errors.some((e) => e.keyword === "required")).toBe(true);
  });
});
