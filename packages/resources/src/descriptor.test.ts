import { describe, expect, it } from "vitest";
import { validateDescriptor, type ResourceDescriptor } from "./descriptor";

const validDescriptor: ResourceDescriptor = {
  id: "compute.server",
  name: "Server",
  pluralName: "Servers",
  iconToken: "server",
  createCapability: { resourceType: "compute.server", action: "create" },
  createSchema: {
    type: "object",
    required: ["name", "regionId", "projectId"],
    properties: {
      name: { type: "string", minLength: 1 },
      regionId: { enum: ["eu-west", "us-east", "ap-south"] },
      projectId: { enum: ["project-1"] },
    },
  },
  supportedActions: [
    {
      id: "start",
      name: "Start",
      requiresConfirmation: false,
      riskClass: "normal",
      requiredCapability: { resourceType: "compute.server", action: "start" },
    },
    {
      id: "delete",
      name: "Delete",
      requiresConfirmation: true,
      riskClass: "destructive",
      requiredCapability: { resourceType: "compute.server", action: "delete" },
    },
  ],
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

  it("rejects a missing createCapability", () => {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const { createCapability: _unused, ...withoutCapability } = validDescriptor;
    expect(() => {
      validateDescriptor(withoutCapability);
    }).toThrow(/createCapability is required/);
  });

  it("validates createCapability shape", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        createCapability: { resourceType: "compute.server" },
      });
    }).toThrow(/createCapability\.action must be a string/);
  });

  it("validates action riskClass enum", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        supportedActions: [
          {
            id: "start",
            name: "Start",
            requiresConfirmation: false,
            riskClass: "unknown",
            requiredCapability: { resourceType: "compute.server", action: "start" },
          },
        ],
      });
    }).toThrow(/riskClass must be one of/);
  });

  it("validates action requiredCapability shape", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        supportedActions: [
          {
            ...validDescriptor.supportedActions[0],
            requiredCapability: { action: "start" },
          },
        ],
      });
    }).toThrow(/requiredCapability\.resourceType must be a string/);
  });

  it("rejects executable-looking string values", () => {
    expect(() => {
      validateDescriptor({ ...validDescriptor, name: "javascript:alert(1)" });
    }).toThrow(/Dangerous descriptor value.*executable string rejected/);

    expect(() => {
      validateDescriptor({ ...validDescriptor, name: "<script>alert(1)</script>" });
    }).toThrow(/Dangerous descriptor value.*executable string rejected/);

    expect(() => {
      validateDescriptor({ ...validDescriptor, name: "data:text/html,<script>alert(1)</script>" });
    }).toThrow(/Dangerous descriptor value.*executable string rejected/);
  });

  it("rejects dangerous schema keys in createSchema", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        createSchema: { type: "object", properties: { name: { "x-araf-script": "alert(1)" } } },
      });
    }).toThrow(/Dangerous schema key.*x-araf-script is not allowed/);
  });

  it("rejects dangerous string values in schemas", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        createSchema: { type: "object", properties: { name: { const: "eval" } } },
      });
    }).toThrow(/Dangerous schema value.*"eval" is not allowed/);
  });

  it("accepts inputSchema on actions", () => {
    expect(() => {
      validateDescriptor({
        ...validDescriptor,
        supportedActions: [
          {
            id: "attach",
            name: "Attach",
            requiresConfirmation: false,
            riskClass: "normal",
            requiredCapability: { resourceType: "storage.volume", action: "attach" },
            inputSchema: {
              type: "object",
              required: ["serverId"],
              properties: { serverId: { type: "string", minLength: 1 } },
            },
          },
        ],
      });
    }).not.toThrow();
  });
});
