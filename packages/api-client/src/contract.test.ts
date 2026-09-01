import { spawn, type ChildProcess } from "node:child_process";
import { createServer } from "node:net";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, beforeAll, afterAll } from "vitest";
import { createArafClient, ArafApiError } from "./index.js";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const bffBinary = resolve(repoRoot, "backend/target/debug/tenant-bff");

function getFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address !== null ? address.port : 0;
      server.close((error) => {
        if (error) {
          reject(error);
        } else {
          resolve(port);
        }
      });
    });
    server.on("error", reject);
  });
}

function startTenantBff(port: number): Promise<ChildProcess> {
  return new Promise((resolve, reject) => {
    const child = spawn(bffBinary, [], {
      cwd: repoRoot,
      env: {
        ...process.env,
        ARAF_TENANT_BFF_PORT: String(port),
        RUST_LOG: "info",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    let resolved = false;
    const buffer: string[] = [];

    function handleData(data: Buffer): void {
      const text = data.toString();
      buffer.push(text);
      if (!resolved && text.includes("tenant-bff listening")) {
        resolved = true;
        resolve(child);
      }
    }

    child.stdout.on("data", handleData);
    child.stderr.on("data", handleData);

    child.on("error", (error) => {
      if (!resolved) {
        reject(error);
      }
    });

    child.on("exit", (code) => {
      if (!resolved) {
        reject(
          new Error(
            `tenant-bff exited before listening (code ${String(code)}). Output:\n${buffer.join("")}`,
          ),
        );
      }
    });

    setTimeout(() => {
      if (!resolved) {
        child.kill("SIGTERM");
        reject(new Error(`timed out waiting for tenant-bff to start. Output:\n${buffer.join("")}`));
      }
    }, 10_000);
  });
}

describe("tenant-bff fixture contract", () => {
  let port: number;
  let bff: ChildProcess | undefined;

  beforeAll(async () => {
    port = await getFreePort();
    bff = await startTenantBff(port);
  }, 15_000);

  afterAll(() => {
    if (bff && !bff.killed) {
      bff.kill("SIGTERM");
      setTimeout(() => {
        if (bff && !bff.killed) {
          bff.kill("SIGKILL");
        }
      }, 2_000);
    }
  });

  it("returns a healthy healthz response", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const health = await client.healthz();

    expect(health.status).toBe("ok");
    expect(health.service).toBe("tenant-bff");
  });

  it("agrees with the fixture on the resource collection total", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const collection = await client.listResources("compute.server", {
      page: 0,
      pageSize: 5,
    });

    expect(collection.total).toBe(100_000);
    expect(collection.items).toHaveLength(5);
    expect(collection.hasMore).toBe(true);
    expect(collection.items[0]?.resourceType).toBe("compute.server");
  });

  it("accepts a resource action and returns a Pending Operation", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const operation = await client.submitAction("compute.server", "resource-0000000001", {
      actionId: "start",
    });

    expect(operation.action).toBe("start");
    expect(operation.state).toBe("pending");
    expect(operation.resourceId).toBe("resource-0000000001");
    expect(operation.resourceType).toBe("compute.server");
  });

  it("surfaces ProblemDetails for an unsupported resource type", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);

    await expect(
      client.listResources("unsupported.type", { page: 0, pageSize: 1 }),
    ).rejects.toBeInstanceOf(ArafApiError);

    try {
      await client.listResources("unsupported.type", { page: 0, pageSize: 1 });
      expect.fail("expected an error");
    } catch (error) {
      const apiError = error as ArafApiError;
      expect(apiError.status).toBe(404);
      expect(apiError.problem.type).toBe("https://araf.o3k.io/errors/404");
      expect(apiError.problem.status).toBe(404);
    }
  });

  it("returns descriptors with M5 action and create schema fields", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const services = await client.listServices();
    const compute = services
      .find((s) => s.id === "compute")
      ?.resourceTypes.find((rt) => rt.id === "compute.server");

    expect(compute).toBeDefined();
    expect(compute?.createCapability).toEqual({
      resourceType: "compute.server",
      action: "create",
    });
    expect(compute?.createSchema).toBeDefined();

    const startAction = compute?.supportedActions.find((a) => a.id === "start");
    expect(startAction).toBeDefined();
    expect(startAction?.riskClass).toBe("normal");
    expect(startAction?.requiredCapability).toEqual({
      resourceType: "compute.server",
      action: "start",
    });

    const deleteAction = compute?.supportedActions.find((a) => a.id === "delete");
    expect(deleteAction?.riskClass).toBe("destructive");
  });

  it("accepts a create resource request and returns a Pending Operation", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const operation = await client.createResource("compute.server", {
      name: "created-server",
      regionId: "eu-west",
      projectId: "project-1",
      bootVolumeSizeGb: 25,
    });

    expect(operation.action).toBe("create");
    expect(operation.state).toBe("pending");
    expect(operation.resourceType).toBe("compute.server");
    expect(operation.correlationId).toBeDefined();
    expect(operation.events).toBeDefined();
    expect(operation.events.length).toBeGreaterThanOrEqual(1);
    expect(operation.events[0]?.state).toBe("pending");
  });

  it("lists operations and supports filtering by resource type and state", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);

    const all = await client.listOperations({ page: 0, pageSize: 5 });
    expect(all.items.length).toBeGreaterThan(0);
    expect(all.total).toBeGreaterThanOrEqual(all.items.length);

    const filtered = await client.listOperations({
      page: 0,
      pageSize: 5,
      resourceType: "compute.server",
      state: "pending",
    });
    expect(filtered.items.every((op) => op.resourceType === "compute.server")).toBe(true);
    expect(filtered.items.every((op) => op.state === "pending")).toBe(true);
  });

  it("returns an Operation timeline with authoritative events", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const operation = await client.submitAction("compute.server", "resource-0000000001", {
      actionId: "start",
    });

    const fetched = await client.getOperation(operation.id);
    expect(fetched.events).toBeDefined();
    expect(fetched.events.length).toBeGreaterThanOrEqual(1);
    expect(fetched.events[0]?.state).toBe("pending");
    expect(fetched.events[0]?.correlationId).toBeDefined();
  });

  it("retrieves a created operation after reload", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const operation = await client.createResource("compute.server", {
      name: "reload-survival-server",
      regionId: "eu-west",
      projectId: "project-1",
      bootVolumeSizeGb: 25,
    });

    const reloaded = await client.getOperation(operation.id);
    expect(reloaded.id).toBe(operation.id);
    expect(reloaded.action).toBe("create");
    expect(reloaded.resourceType).toBe("compute.server");
    expect(reloaded.events.length).toBeGreaterThanOrEqual(1);
  });
});
