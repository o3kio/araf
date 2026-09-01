import { describe, expect, it } from "vitest";
import { validateDescriptor, type ResourceDescriptor } from "./descriptor";

const validDescriptor: ResourceDescriptor = {
  id: "compute.server",
  name: "Server",
  pluralName: "Servers",
  iconToken: "server",
  supportedActions: [{ id: "start", name: "Start", requiresConfirmation: false }],
  columns: [{ id: "name", header: "Name", field: "name" }],
  filters: [{ id: "project", label: "Project", field: "projectId", kind: "select" }],
  sortableFields: ["name", "status"],
  detailsSections: [{ id: "overview", label: "Overview", fields: ["id", "name"] }],
  relationships: [
    {
      id: "attachedServer",
      targetResourceType: "compute.server",
      label: "Attached Server",
      sourcePropertyKey: "attachedServerId",
      direction: "to-one",
    },
  ],
};

describe("validateDescriptor", () => {
  it("accepts a valid descriptor", () => {
    expect(() => {
      validateDescriptor(validDescriptor);
    }).not.toThrow();
  });

  it("rejects a non-object descriptor", () => {
    expect(() => {
      validateDescriptor(null);
    }).toThrow(/plain object/);
    expect(() => {
      validateDescriptor("server");
    }).toThrow(/plain object/);
  });

  it("rejects unknown top-level fields", () => {
    expect(() => {
      validateDescriptor({ ...validDescriptor, script: "alert(1)" });
    }).toThrow(/Unsupported descriptor field\(s\).*script/);
  });

  it("rejects unknown column fields", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        columns: [{ id: "name", header: "Name", field: "name", render: "dangerous" }],
      });
    }).toThrow(/Unsupported descriptor field\(s\).*render/);
  });

  it("rejects unknown filter kind", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        filters: [{ id: "project", label: "Project", field: "projectId", kind: "sql" }],
      });
    }).toThrow(/kind must be "text" or "select"/);
  });

  it("accepts to-many relationships and rejects invalid directions", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        relationships: [
          {
            id: "attachedServer",
            targetResourceType: "compute.server",
            label: "Attached Server",
            sourcePropertyKey: "attachedServerId",
            direction: "to-many",
          },
        ],
      });
    }).not.toThrow();

    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        relationships: [
          {
            id: "attachedServer",
            targetResourceType: "compute.server",
            label: "Attached Server",
            sourcePropertyKey: "attachedServerId",
            direction: "has-many",
          },
        ],
      });
    }).toThrow(/direction must be "to-one" or "to-many"/);
  });

  it("rejects missing required string fields", () => {
    expect(() => {
      validateDescriptor({ ...validDescriptor, id: 123 });
    }).toThrow(/Descriptor\.id must be a string/);
  });

  it("rejects non-array array fields", () => {
    expect(() => {
      validateDescriptor({ ...validDescriptor, columns: "name" });
    }).toThrow(/Descriptor\.columns must be an array/);
  });
});
