import { describe, expect, it } from "vitest";
import type { Resource } from "@araf/api-client";
import { getResourceField, formatResourceField } from "./fields";

const resource: Resource = {
  id: "resource-0000000001",
  name: "fixture-server-1",
  resourceType: "compute.server",
  projectId: "project-1",
  regionId: "eu-west",
  status: "ready",
  createdAt: "2024-01-01T00:00:00Z",
  updatedAt: "2024-01-01T00:01:00Z",
  properties: {
    sizeGb: 100,
    attachedServerId: "resource-0000000002",
    nested: { value: "deep" },
  },
};

describe("getResourceField", () => {
  it("extracts top-level fields", () => {
    expect(getResourceField(resource, "id")).toBe("resource-0000000001");
    expect(getResourceField(resource, "name")).toBe("fixture-server-1");
    expect(getResourceField(resource, "status")).toBe("ready");
  });

  it("extracts dotted properties.* fields", () => {
    expect(getResourceField(resource, "properties.sizeGb")).toBe(100);
    expect(getResourceField(resource, "properties.attachedServerId")).toBe("resource-0000000002");
  });

  it("returns undefined for missing fields", () => {
    expect(getResourceField(resource, "properties.missing")).toBeUndefined();
    expect(getResourceField(resource, "unknownField")).toBeUndefined();
  });

  it("returns undefined for empty properties prefix", () => {
    expect(getResourceField(resource, "properties.")).toBeUndefined();
  });
});

describe("formatResourceField", () => {
  it("formats strings as-is", () => {
    expect(formatResourceField("hello")).toBe("hello");
  });

  it("formats numbers and booleans", () => {
    expect(formatResourceField(42)).toBe("42");
    expect(formatResourceField(true)).toBe("true");
  });

  it("returns em-dash for null or undefined", () => {
    expect(formatResourceField(null)).toBe("—");
    expect(formatResourceField(undefined)).toBe("—");
  });

  it("falls back to JSON for objects", () => {
    expect(formatResourceField({ value: "deep" })).toBe('{"value":"deep"}');
  });
});
